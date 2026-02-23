from __future__ import annotations
from pathlib import Path
import time
from xml.etree import ElementTree as ET
from .adb_utils import adb_reboot, kill_adb_server
from .constants import IMAGE_DIR, TOOLS_DIR, READBACK_DIR
from .flash_spft import launch_spft_gui, run_firmware_upgrade
from .global_flow import _ask_country_change_plan, _check_flash_xml_platform, _cleanup_after_flow, _cleanup_before_flow, _detect_platform, _log_device_extra_info, _prepare_prc_lkdtbo_files
from .port_scan import wait_for_preloader
from .proinfo_country import wait_and_patch_proinfo
from .scatter import disable_lk_dtbo_partitions, prepare_platform_scatter, apply_country_plan_to_proinfo, backup_platform_scatter_to_logs
from .utils import clear_console, log, wait_for_device
 
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


def run_firmware_upgrade_keep_data_flow() -> None:
    clear_console()
    log('app.menu.separator')
    log('flow.keep_data.start')
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
    if not _check_flash_xml_platform(platform):
        return
    time.sleep(3)
    if not _prepare_prc_lkdtbo_files():
        return
    time.sleep(3)
    change_plan = _ask_country_change_plan()
    time.sleep(3)
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
        log('country.no_change')
    if not wait_for_device():
        return
    log('flow.reboot_now')
    backup_platform_scatter_to_logs(platform)
    adb_reboot()
    log('preloader.waiting')
    log('preloader.detected')
    run_firmware_upgrade()
    _cleanup_after_flow(platform)
    log('flow.done')
    kill_adb_server()
