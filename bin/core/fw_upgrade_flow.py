from __future__ import annotations
from pathlib import Path
import time
import re
from xml.etree import ElementTree as ET
from .adb_utils import adb_reboot, kill_adb_server
from . import adb_utils as adb_state
from .constants import IMAGE_DIR, TOOLS_DIR, READBACK_DIR, PLATFORM_TOOLS_DIR
from .flash_spft import launch_spft_gui, run_firmware_upgrade
from .global_flow import _ask_country_change_plan, _check_flash_xml_platform, _cleanup_after_flow, _cleanup_before_flow, _log_device_extra_info, _prepare_prc_lkdtbo_files, _country_code_feature_enabled, wait_for_fastboot, _detect_current_ab_slot, _switch_ab_slot_fastboot, _force_slot_a_via_adb, _normalize_rom_region
from .port_scan import wait_for_preloader
from .proinfo_country import wait_and_patch_proinfo
from .scatter import disable_lk_dtbo_partitions, prepare_platform_scatter, apply_country_plan_to_proinfo, backup_platform_scatter_to_logs
from .firmware_guard import validate_firmware_image
from .utils import clear_console, log, log_text, wait_for_device, adb_shell_getprop, run_adb, run_cmd
 
def _confirm_keep_data() -> bool:
    log('flow.keep_data.confirm')
    while True:
        answer = input().strip().lower()
        if answer in ('y', 'yes'):
            return True
        if answer in ('n', 'no'):
            return False

def _delete_history_ini() -> None:
    history = TOOLS_DIR / 'history.ini'
    if history.exists():
        try:
            history.unlink()
        except OSError:
            pass

def _iter_partitions(root: ET.Element):
    for tag in ('partition', 'partition_index'):
        for part in root.findall(f'.//{tag}'):
            name_elem = part.find('partition_name')
            if name_elem is None:
                continue
            name = (name_elem.text or '').strip()
            if not name:
                continue
            yield (part, name)

def _ensure_child_text(parent: ET.Element, tag: str, text: str) -> None:
    elem = parent.find(tag)
    if elem is None:
        elem = ET.SubElement(parent, tag)
    elem.text = text

def _patch_userdata_keep_data(scatter_path: Path) -> None:
    if not scatter_path.is_file():
        log('scatter.userdata_not_found')
        return
    try:
        tree = ET.parse(scatter_path)
    except ET.ParseError:
        log('scatter.userdata_not_found')
        return
    root = tree.getroot()
    found_proinfo = False
    found_userdata = False
    for part, name in _iter_partitions(root):
        low = name.lower()
        if low == 'proinfo':
            _ensure_child_text(part, 'file_name', 'proinfo')
            _ensure_child_text(part, 'is_download', 'true')
            _ensure_child_text(part, 'is_upgradable', 'true')
            found_proinfo = True
        elif low == 'userdata':
            _ensure_child_text(part, 'is_download', 'false')
            _ensure_child_text(part, 'is_upgradable', 'false')
            found_userdata = True
    if not found_proinfo:
        log('scatter.proinfo_not_found')
    if not found_userdata:
        log('scatter.userdata_not_found')
    tree.write(scatter_path, encoding='utf-8', xml_declaration=True)
    log('scatter.userdata_patched')


def _detect_platform_keep_data() -> str | None:
    version = (adb_shell_getprop('ro.build.version.release') or '').strip()
    adb_state.LAST_ANDROID_VERSION_RELEASE = version
    platform = (adb_shell_getprop('ro.vendor.mediatek.platform') or '').strip()
    if not platform or not platform.startswith('MT'):
        log('flow.not_mtk')
        return None
    adb_state.LAST_MTK_PLATFORM = platform
    return platform

def run_firmware_upgrade_keep_data_flow() -> None:
    clear_console()
    log('app.menu.separator')
    log('flow.keep_data.start')
    log('app.menu.separator')
    log('flow.stage1_header')
    if not wait_for_device():
        return
    region = adb_shell_getprop('ro.config.zui.region') or ''
    region_upper = _normalize_rom_region(region)
    adb_state.LAST_DEVICE_ROM_REGION = region_upper
    if region_upper != 'ROW':
        log('flow.keep_data.unknown_region')
        return
    platform = _detect_platform_keep_data()
    if platform is None:
        return
    _log_device_extra_info()
    _cleanup_before_flow()
    time.sleep(3)
    log_text('')
    log('flow.stage2_header')
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
        log('country.no_change')
        change_plan = False
    time.sleep(3)
    log_text('')
    log('flow.stage3_header')
    log('flow.scatter_prepare')
    scatter_path = prepare_platform_scatter(platform, keep_user_data=True)
    if scatter_path is None:
        return
    time.sleep(3)
    disable_lk_dtbo_partitions(platform)
    _patch_userdata_keep_data(scatter_path)
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
    log_text('')
    log('flow.stage4_header')
    if not wait_for_device():
        return
    _force_slot_a_via_adb()
    try:
        run_adb(['reboot', 'bootloader'], capture_output=True)
    except Exception:
        pass
    log('flow.fastboot.detect')
    if not wait_for_fastboot():
        log('flow.fastboot_not_detected')
        kill_adb_server()
        return
    current_slot = _detect_current_ab_slot()
    if not _switch_ab_slot_fastboot(current_slot):
        kill_adb_server()
        return
    log('flow.reboot_now')
    backup_platform_scatter_to_logs(platform)
    try:
        run_cmd([str(PLATFORM_TOOLS_DIR / 'fastboot'), 'reboot'])
    except Exception:
        try:
            run_cmd(['fastboot', 'reboot'])
        except Exception:
            pass
    log_text('')
    log('flow.stage5_header')
    log('preloader.waiting')
    log('preloader.detected')
    run_firmware_upgrade()
    _cleanup_after_flow(platform)
    log('flow.done')
    kill_adb_server()

