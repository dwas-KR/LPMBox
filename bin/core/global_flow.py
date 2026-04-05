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
from .firmware_guard import validate_firmware_image, detect_vendor_boot_rom_type, inspect_vendor_boot_image, should_show_tb37x_qna_warning
from .scatter import disable_lk_dtbo_partitions, prepare_platform_scatter, apply_country_plan_to_proinfo, backup_platform_scatter_to_logs, ensure_prc_platform_scatter
from .utils import clear_console, log, log_text, wait_for_device, _write_log_line, run_adb, run_cmd, format_prompt_line, log_model_value, classify_model_name, log_model_support_messages, handle_unsupported_model

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


def _preserve_current_scatter_xml() -> bool:
    image_value = (getattr(adb_state, 'LAST_IMAGE_ROM_REGION', '') or '').strip().upper()
    device_value = (getattr(adb_state, 'LAST_DEVICE_ROM_REGION', '') or '').strip().upper()
    if image_value == 'PRC' or device_value == 'PRC':
        return True
    try:
        info = inspect_vendor_boot_image()
        region = (info.get('rom_region') or '').strip().upper()
        if region:
            adb_state.LAST_IMAGE_ROM_REGION = region
        return region == 'PRC'
    except Exception:
        return False


def _cleanup_before_flow() -> None:
    if not _preserve_current_scatter_xml():
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


def _normalize_rom_region(value: str) -> str:
    return re.sub(r'\s+', '', value or '').upper()


def _ask_country_change_plan() -> bool:
    code = adb_shell_getprop("ro.product.countrycode").strip()
    if not code:
        code = "UNKNOWN"
    code = (code or '').strip().upper() or 'UNKNOWN'
    if code == "UNKNOWN":
        log('cable.check_1')
        log('cable.check_2')
    while True:
        prompt = get_string("country.change_plan_prompt").format(code=code)
        base = f"{prompt}"
        print(format_prompt_line('country.change_plan_prompt', base), end="")
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
    if not _preserve_current_scatter_xml():
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
    region = adb_shell_getprop('ro.config.zui.region') or ''
    region_upper = _normalize_rom_region(region)
    image_region = detect_vendor_boot_rom_type()
    adb_state.LAST_DEVICE_ROM_REGION = region_upper
    adb_state.LAST_IMAGE_ROM_REGION = image_region or ''
    adb_state.PREFER_ROOT_FLASH_XML = False
    if region_upper == 'ROW' and image_region == 'ROW':
        log('flow.prc.rom_row_warn')
        kill_adb_server()
        return None
    if v_num is not None and v_num <= 14:
        log('flow.android_version_low')
        kill_adb_server()
        return None
    platform = (adb_shell_getprop('ro.vendor.mediatek.platform') or '').strip()
    adb_state.LAST_MTK_PLATFORM = platform
    return platform

def _log_device_extra_info() -> bool:
    hw = (adb_shell_getprop('ro.product.model') or '').strip()
    if not hw:
        hw = (adb_shell_getprop('ro.vendor.config.lgsi.hw.version') or '').strip()
    if not hw:
        hw = '?'
    normalized_model = log_model_value('flow.device_info_value', hw, field_name='hw')
    adb_state.LAST_DEVICE_MODEL = normalized_model
    region = (getattr(adb_state, 'LAST_DEVICE_ROM_REGION', '') or '').strip().upper()
    if region:
        log('flow.keep_data.rom_type', region=region)
    version = getattr(adb_state, 'LAST_ANDROID_VERSION_RELEASE', '') or (adb_shell_getprop('ro.build.version.release') or '').strip()
    adb_state.LAST_ANDROID_VERSION_RELEASE = version
    if version:
        log('flow.android_version_detected', version=version)
    platform = (getattr(adb_state, 'LAST_MTK_PLATFORM', '') or '').strip()
    log('flow.platform', platform=platform)
    category = classify_model_name(normalized_model)
    if category != 'supported':
        log_model_support_messages(normalized_model)
        return False
    if not platform or not platform.startswith('MT'):
        log('flow.not_mtk')
        return False
    return True

def _prepare_prc_lkdtbo_files_for_model(raw_model: str) -> bool:
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


def _prepare_prc_lkdtbo_files() -> bool:
    raw_model = adb_shell_getprop('ro.product.model').strip()
    return _prepare_prc_lkdtbo_files_for_model(raw_model)

def _should_show_tb37x_qna_warning() -> bool:
    image_info = inspect_vendor_boot_image()
    return should_show_tb37x_qna_warning(
        getattr(adb_state, 'LAST_DEVICE_MODEL', ''),
        getattr(adb_state, 'LAST_IMAGE_MODEL', ''),
        image_info.get('model', ''),
    )

def _maybe_log_tb37x_qna_warning() -> None:
    if _should_show_tb37x_qna_warning():
        log('flow.tb37x_qna_warn')

def _iter_flash_xml_candidates() -> list[Path]:
    return [FLASH_XML_DLAGENT, FLASH_XML_ROOT]

def _find_flash_xml() -> Path | None:
    for path in _iter_flash_xml_candidates():
        if path.is_file():
            return path
    return None

def _check_flash_xml_platform(platform: str) -> bool:
    found_any = False
    last_mismatch: tuple[str, str] | None = None
    for flash_xml in _iter_flash_xml_candidates():
        if not flash_xml.is_file():
            continue
        found_any = True
        try:
            text = flash_xml.read_text(encoding='utf-8', errors='ignore')
        except Exception:
            continue
        match = re.search(r'<scatter>\.\./(MT\d+)_Android_scatter\.xml</scatter>', text)
        if not match:
            continue
        flash_platform = match.group(1)
        if flash_platform == platform:
            log('flow.flash_xml_ok', platform=platform)
            return True
        last_mismatch = (flash_platform, platform)
    if not found_any:
        log('flow.flash_xml_missing')
        return False
    if last_mismatch is not None:
        log('flow.platform_mismatch')
        return False
    log('flow.flash_xml_read_error')
    return False






def _detect_current_ab_slot(log_detect: bool = True, log_current: bool = True) -> str | None:
    if log_detect:
        log('flow.ab_slot.detect')
    commands = [
        [str(PLATFORM_TOOLS_DIR / 'fastboot'), 'getvar', 'current-slot'],
        ['fastboot', 'getvar', 'current-slot'],
    ]
    output = ''
    for cmd in commands:
        try:
            result = run_cmd(cmd, timeout=10)
        except Exception:
            continue
        text = ((result.stdout or '') + '\n' + (result.stderr or '')).strip()
        if text:
            output = text
            break
    if not output:
        if log_detect:
            log('flow.ab_slot.skip')
        return None
    slot = None
    for line in output.splitlines():
        line_lower = line.strip().lower()
        if 'current-slot' not in line_lower:
            continue
        m = re.search(r'current-slot[^ab]*([ab])\b', line_lower)
        if m:
            slot = m.group(1)
            break
        for ch in reversed(line_lower):
            if ch in ('a', 'b'):
                slot = ch
                break
        if slot:
            break
    if slot not in ('a', 'b'):
        if log_detect:
            log('flow.ab_slot.skip')
        return None
    if log_current:
        log('flow.ab_slot.current', slot=slot.upper())
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


def _switch_ab_slot_fastboot(current_slot: str | None) -> bool:
    slot = (current_slot or '').lower()
    if slot == 'a':
        log('flow.ab_slot.ok')
    elif slot == 'b':
        log('flow.ab_slot.switch', from_slot='B', to_slot='A')
    else:
        log('flow.ab_slot.switch', from_slot='UNKNOWN', to_slot='A')
    log('flow.work_in_progress')

    def _run_fastboot_any(args: list[str], timeout: int = 10) -> bool:
        ok = False
        for base_cmd in ([str(PLATFORM_TOOLS_DIR / 'fastboot')], ['fastboot']):
            try:
                cp = run_cmd(base_cmd + args, timeout=timeout)
                ok = ok or getattr(cp, 'returncode', 1) == 0
            except Exception:
                continue
        return ok

    def _set_slot_sequence() -> None:
        for _ in range(3):
            _run_fastboot_any(['--set-active=a'])
            time.sleep(1)
            _run_fastboot_any(['set_active', 'a'])
            time.sleep(1)
            _run_fastboot_any(['-aa'])

    _set_slot_sequence()
    _run_fastboot_any(['reboot', 'bootloader'])
    if not wait_for_fastboot(timeout=30):
        log('flow.fastboot_not_detected')
        return False
    _set_slot_sequence()
    final_slot = _detect_current_ab_slot(log_detect=False, log_current=False)
    if final_slot in ('a', 'b'):
        log('flow.ab_slot.rechecked', slot=final_slot.upper())
        if final_slot == 'a':
            log('flow.ab_slot.ok')
            log('flow.stability_wait')
            time.sleep(5)
            return True
    log('flow.ab_slot.error')
    _run_fastboot_any(['reboot'])
    return False


def _trigger_rom_install_reboot_commands() -> None:
    log('flow.reboot_stability')
    try:
        run_adb(['reboot'], capture_output=True)
    except Exception:
        pass
    try:
        run_cmd([str(PLATFORM_TOOLS_DIR / 'fastboot'), 'reboot'])
    except Exception:
        try:
            run_cmd(['fastboot', 'reboot'])
        except Exception:
            pass


def run_current_slot_stage(stage_header_key: str = 'flow.stage4_header', require_device: bool = True) -> bool:
    log_text('')
    log(stage_header_key)
    if require_device:
        if not wait_for_device():
            return False
    else:
        return True
    _force_slot_a_via_adb()
    try:
        run_adb(['reboot', 'bootloader'], capture_output=True)
    except Exception:
        log('flow.fastboot_reboot_failed')
        return False
    log('flow.fastboot.detect')
    if not wait_for_fastboot(timeout=60):
        log('flow.fastboot_not_detected')
        return False
    current_slot = _detect_current_ab_slot()
    if not _switch_ab_slot_fastboot(current_slot):
        return False
    return True


def run_global_firmware_upgrade_flow() -> None:
    clear_console()
    log('app.menu.separator')
    log('flow.start')
    log('app.menu.separator')
    log('flow.stage1_header')
    if not wait_for_device():
        return
    time.sleep(3)
    platform = _detect_platform()
    if platform is None:
        return
    if not _log_device_extra_info():
        return
    _cleanup_before_flow()
    time.sleep(3)
    log_text('')
    log('flow.stage2_header')
    if not validate_firmware_image():
        return
    log('flow.tb37x_qna_warn')
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
        log('country.no_change')
        change_plan = False
    time.sleep(3)
    log_text('')
    log('flow.stage3_header')
    log('flow.scatter_prepare')
    scatter_path = prepare_platform_scatter(platform, keep_user_data=False)
    if scatter_path is None:
        return
    time.sleep(3)
    disable_lk_dtbo_partitions(platform)
    time.sleep(3)
    apply_country_plan_to_proinfo(platform, change_plan)
    ensure_prc_platform_scatter(platform, preserve_userdata_false=False)
    time.sleep(3)
    _delete_history_ini()
    if change_plan:
        launch_spft_gui()
        wait_and_patch_proinfo(platform)
    else:
        if country_feature:
            log('country.no_change')
    if not run_current_slot_stage('flow.stage4_header', require_device=True):
        return
    backup_platform_scatter_to_logs(platform)
    log_text('')
    log('flow.stage5_header')
    _trigger_rom_install_reboot_commands()
    if not wait_for_preloader():
        return
    run_firmware_upgrade()
    _cleanup_after_flow(platform)
    log('flow.done')
    kill_adb_server()

