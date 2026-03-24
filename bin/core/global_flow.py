from __future__ import annotations
from pathlib import Path
import time
import subprocess
import shutil
import re
import json
from datetime import datetime
from xml.etree import ElementTree as ET
from .adb_utils import adb_reboot, adb_shell_getprop, kill_adb_server
from . import adb_utils as adb_state
from . import downloader
from .constants import FLASH_XML_DLAGENT, FLASH_XML_ROOT, IMAGE_DIR, READBACK_DIR, TOOLS_DIR, PLATFORM_TOOLS_DIR, LKDTBO_DIR, LKDTBO_MODEL_TO_ZIP
from .flash_spft import launch_spft_gui, run_firmware_upgrade
from .i18n import get_string
from .port_scan import wait_for_preloader
from .proinfo_country import wait_and_patch_proinfo
from .firmware_guard import validate_firmware_image
from .scatter import disable_lk_dtbo_partitions, prepare_platform_scatter, apply_country_plan_to_proinfo, backup_platform_scatter_to_logs
from .utils import clear_console, log, wait_for_device, _write_log_line, run_adb, run_cmd

_SETTINGS_PATH = Path(__file__).resolve().parent / 'lang' / 'settings.json'

def _load_settings() -> dict:
    try:
        if _SETTINGS_PATH.is_file():
            data = json.loads(_SETTINGS_PATH.read_text(encoding='utf-8', errors='ignore'))
            if isinstance(data, dict):
                return data
    except Exception:
        pass
    return {}

def _country_code_feature_enabled() -> bool:
    data = _load_settings()
    v = data.get('country_code_feature')
    if isinstance(v, bool):
        return v
    if isinstance(v, int):
        return bool(v)
    if isinstance(v, str):
        t = v.strip().lower()
        if t in ('1','true','yes','y','on'):
            return True
        if t in ('0','false','no','n','off'):
            return False
    return True

 
def _cleanup_before_flow() -> None:
    for path in IMAGE_DIR.glob('*_Android_scatter.xml'):
        try:
            path.unlink()
        except OSError:
            pass
    if READBACK_DIR.is_dir():
        for path in READBACK_DIR.glob('proinfo*'):
            try:
                path.unlink()
            except OSError:
                pass
    history = TOOLS_DIR / 'history.ini'
    if history.exists():
        try:
            history.unlink()
        except OSError:
            pass

    for name in ('lk.img', 'dtbo.img'):
        p = IMAGE_DIR / name
        if p.exists():
            try:
                p.unlink()
            except OSError:
                pass


def _delete_history_ini() -> None:
    history_path = TOOLS_DIR / 'history.ini'
    if history_path.is_file():
        history_path.unlink()
from .utils import log_text, log
from .adb_utils import adb_shell_getprop


def _ask_country_change_plan() -> bool:
    code = adb_shell_getprop("ro.product.countrycode").strip()
    if not code:
        code = "UNKNOWN"
    if code == "UNKNOWN":
        log("country.unknown_cable")
        log("country.unknown_bootloop")
    while True:
        prompt = get_string("country.change_plan_prompt").format(code=code)
        base = f"{prompt}"
        print(base, end="")
        raw = input().strip()
        line = f"{base}{raw}"
        _write_log_line(line)
        answer = raw.lower()
        if answer in ("y", "yes"):
            return True
        if answer in ("n", "no"):
            return False
        log("input.retry")

def _cleanup_after_flow(platform: str | None=None) -> None:
    if platform:
        pattern = f'{platform}_Android_scatter*.xml'
    else:
        pattern = '*_Android_scatter.xml'
    for path in IMAGE_DIR.glob(pattern):
        try:
            path.unlink()
        except OSError:
            pass
    if READBACK_DIR.is_dir():
        for path in READBACK_DIR.glob('proinfo*'):
            try:
                path.unlink()
            except OSError:
                pass

def _detect_platform() -> str | None:
    log('flow.detect_platform')
    log('flow.android_version_detecting')
    version = adb_shell_getprop('ro.build.version.release').strip()
    adb_state.LAST_ANDROID_VERSION_RELEASE = version
    v_num = None
    if version:
        m = re.match(r'(\d+)', version)
        if m:
            try:
                v_num = int(m.group(1))
            except Exception:
                v_num = None
    if v_num is not None and v_num <= 14:
        log('flow.android_version_detected', version=version)
    region = adb_shell_getprop('ro.config.zui.region').strip()
    region_upper = region.upper()
    log('flow.keep_data.rom_type', region=region_upper if region_upper else region)
    if region_upper == 'ROW':
        log('flow.prc.rom_row_warn')
        kill_adb_server()
        return None
    if region_upper == 'PRC':
        log('flow.prc.rom_prc_ok')
    if v_num is not None and v_num <= 14:
        if v_num == 14 and region_upper == 'PRC':
            log('ota.restore_start')
            try:
                from .ota_enable_flow import _restore_ota_packages
                _restore_ota_packages()
            except Exception:
                pass
            log('ota.update_hint')
        log('flow.android_version_low')
        kill_adb_server()
        return None
    platform = adb_shell_getprop('ro.vendor.mediatek.platform').strip()
    if not platform or not platform.startswith('MT'):
        log('flow.not_mtk')
        return None
    log('flow.platform', platform=platform)
    return platform

def _log_device_extra_info() -> None:
    hw = adb_shell_getprop('ro.vendor.config.lgsi.hw.version').strip()
    cpu = adb_shell_getprop('ro.vendor.config.lgsi.cpuinfo').strip()
    if not hw:
        hw = '?'
    if not cpu:
        cpu = '?'
    log('flow.device_info_value', hw=hw, cpu=cpu)
    version = adb_state.LAST_ANDROID_VERSION_RELEASE
    if not version:
        version = adb_shell_getprop('ro.build.version.release').strip()
        adb_state.LAST_ANDROID_VERSION_RELEASE = version
    if version:
        log('flow.android_version_detected', version=version)


def _prepare_prc_lkdtbo_files() -> bool:
    log('flow.model_detecting')
    raw_model = adb_shell_getprop('ro.product.model').strip()
    adb_state.LAST_DEVICE_MODEL = raw_model
    model = None
    for key in LKDTBO_MODEL_TO_ZIP.keys():
        if key in raw_model:
            model = key
            break

    for name in ('lk_a', 'lk_b', 'dtbo_a', 'dtbo_b'):
        p = IMAGE_DIR / name
        if p.exists():
            try:
                p.unlink()
            except OSError:
                pass

    if LKDTBO_DIR.exists():
        try:
            shutil.rmtree(LKDTBO_DIR, ignore_errors=True)
        except Exception:
            pass

    if model is not None:
        dl_dir = TOOLS_DIR / 'download files'
        zip_name = LKDTBO_MODEL_TO_ZIP.get(model)
        if zip_name:
            zp = dl_dir / zip_name
            if zp.exists():
                try:
                    zp.unlink()
                except OSError:
                    pass

    if model is None:
        log('flow.model_not_supported')
        kill_adb_server()
        return False

    if model in {'TB365FC', 'TB361FU', 'TB335FC', 'TB336FU'}:
        return True

    if model not in {'TB375FC', 'TB373FU'}:
        log('flow.model_not_supported')
        kill_adb_server()
        return False

    log('flow.lkdtbo_downloading')
    zip_path = downloader.ensure_lkdtbo_zip_for_model(model)
    if zip_path is None or not zip_path.is_file():
        return False
    log('flow.lkdtbo_extracting')
    ok = downloader.extract_lkdtbo_zip(zip_path, LKDTBO_DIR)
    if not ok:
        return False
    LKDTBO_DIR.mkdir(parents=True, exist_ok=True)
    for name in ('lk_a', 'lk_b', 'dtbo_a', 'dtbo_b'):
        src = LKDTBO_DIR / name
        if not src.is_file():
            return False
        dst = IMAGE_DIR / name
        try:
            shutil.copy2(src, dst)
        except Exception:
            return False
    log('flow.lkdtbo_ready')
    return True


def _find_flash_xml() -> Path | None:
    candidates = [FLASH_XML_ROOT, FLASH_XML_DLAGENT]
    for path in candidates:
        if path.is_file():
            return path
    return None

def _check_flash_xml_platform(platform: str) -> bool:
    flash_xml = _find_flash_xml()
    if flash_xml is None:
        log('flow.flash_xml_missing')
        return False
    try:
        text = flash_xml.read_text(encoding='utf-8', errors='ignore')
    except Exception:
        log('flow.flash_xml_read_error')
        return False
    match = re.search(r'<scatter>\.\./(MT\d+)_Android_scatter\.xml</scatter>', text)
    if not match:
        log('flow.flash_xml_read_error')
        return False
    flash_platform = match.group(1)
    if flash_platform != platform:
        log('flow.flash_xml_mismatch', flash_platform=flash_platform, platform=platform)
        return False
    log('flow.flash_xml_ok', platform=platform)
    return True






def _detect_current_ab_slot() -> str | None:
    log("flow.ab_slot.detect")
    commands = [
        [str(PLATFORM_TOOLS_DIR / "fastboot"), "getvar", "current-slot"],
        ["fastboot", "getvar", "current-slot"],
    ]
    output = ""
    for cmd in commands:
        try:
            result = run_cmd(cmd, timeout=10)
        except Exception:
            continue
        text = ((result.stdout or "") + "\n" + (result.stderr or "")).strip()
        if text:
            output = text
            break
    if not output:
        log("flow.ab_slot.skip")
        return None
    slot: str | None = None
    for line in output.splitlines():
        line_lower = line.strip().lower()
        if "current-slot" not in line_lower:
            continue
        m = re.search(r"current-slot[^ab]*([ab])\b", line_lower)
        if m:
            slot = m.group(1)
            break
        for ch in reversed(line_lower):
            if ch in ("a", "b"):
                slot = ch
                break
        if slot:
            break
    if slot not in ("a", "b"):
        log("flow.ab_slot.skip")
        return None
    log("flow.ab_slot.current", slot=slot.upper())
    return slot


def wait_for_fastboot(timeout: int = 60) -> bool:
    start = time.time()
    while time.time() - start < timeout:
        try:
            result = run_cmd([str(PLATFORM_TOOLS_DIR / "fastboot"), "devices"], timeout=5)
            output = ((result.stdout or "") + "\n" + (result.stderr or "")).strip()
            if output:
                return True
        except Exception:
            try:
                result = run_cmd(["fastboot", "devices"], timeout=5)
                output = ((result.stdout or "") + "\n" + (result.stderr or "")).strip()
                if output:
                    return True
            except Exception:
                pass
        time.sleep(3)
    return False



def _force_slot_a_via_adb() -> None:
    for i in range(10):
        ok = False
        try:
            result = run_adb(["shell", "bootctl", "set-active-boot-slot", "0"], capture_output=True)
            if getattr(result, "returncode", 0) == 0:
                ok = True
        except Exception:
            pass
        if ok:
            return
        if i < 9:
            time.sleep(2)


def _switch_ab_slot_fastboot(current_slot: str | None) -> None:
    slot = (current_slot or "").lower()
    assume_ok = slot == "a"
    if slot in ("a", "b"):
        slot_name = slot.upper()
        if assume_ok:
            log("flow.ab_slot.ok", slot=slot_name)
        else:
            log("flow.ab_slot.switch", from_slot=slot_name, to_slot="A")
    else:
        log("flow.ab_slot.switch", from_slot="UNKNOWN", to_slot="A")
    success_any = assume_ok
    for i in range(10):
        ok = False
        for base_cmd in ([str(PLATFORM_TOOLS_DIR / "fastboot")], ["fastboot"]):
            try:
                result = run_cmd(base_cmd + ["set_active", "a"], timeout=10)
                if getattr(result, "returncode", 0) == 0:
                    ok = True
            except Exception:
                pass
            try:
                run_cmd(base_cmd + ["--set-active=a"], timeout=10)
            except Exception:
                pass
            if ok:
                break
        if ok:
            success_any = True
        if i < 9:
            time.sleep(2)
    if not assume_ok:
        if success_any:
            log("flow.ab_slot.switched", slot="A")
        else:
            log("flow.ab_slot.error")
    return



def run_global_firmware_upgrade_flow() -> None:
    clear_console()
    log('app.menu.separator')
    log('flow.start')
    log('app.menu.separator')
    if not wait_for_device():
        return
    time.sleep(3)
    log('flow.device_info_check')
    platform = _detect_platform()
    if platform is None:
        return
    _log_device_extra_info()
    _cleanup_before_flow()
    time.sleep(3)
    if not validate_firmware_image():
        return
    time.sleep(3)
    if not _check_flash_xml_platform(platform):
        return
    time.sleep(3)
    if not _prepare_prc_lkdtbo_files():
        return
    time.sleep(3)
    country_feature = _country_code_feature_enabled()
    if country_feature:
        change_plan = _ask_country_change_plan()
    else:
        log('country.feature_skip')
        change_plan = False
    time.sleep(3)
    log('flow.scatter_prepare')
    scatter_path = prepare_platform_scatter(platform, keep_user_data=False)
    if scatter_path is None:
        return
    time.sleep(3)
    disable_lk_dtbo_partitions(platform)
    time.sleep(3)
    apply_country_plan_to_proinfo(platform, change_plan)
    time.sleep(3)
    _delete_history_ini()
    if change_plan:
        launch_spft_gui()
        wait_and_patch_proinfo(platform)
    else:
        if country_feature:
            log('country.no_change')
    if not wait_for_device():
        return
    _force_slot_a_via_adb()
    try:
        run_adb(["reboot", "bootloader"])
    except Exception:
        log("flow.fastboot_reboot_failed")
        return
    log("flow.fastboot.detect")
    if not wait_for_fastboot(timeout=60):
        log("flow.fastboot_not_detected")
        return
    current_slot = _detect_current_ab_slot()
    _switch_ab_slot_fastboot(current_slot)
    backup_platform_scatter_to_logs(platform)
    log("preloader.waiting")
    try:
        run_cmd([str(PLATFORM_TOOLS_DIR / "fastboot"), "reboot"])
    except Exception:
        try:
            run_cmd(["fastboot", "reboot"])
        except Exception:
            pass
    log("preloader.detected")
    run_firmware_upgrade()
    _cleanup_after_flow(platform)
    log('flow.done')
    kill_adb_server()
