from __future__ import annotations
import time
import subprocess
from pathlib import Path
from . import adb_utils as adb_state
from .flash_spft import launch_spft_gui, run_firmware_upgrade
from .firmware_guard import inspect_vendor_boot_image, inspect_flash_xml_platform, should_show_tb37x_qna_warning
from .global_flow import _cleanup_after_flow, _cleanup_before_flow, _country_code_feature_enabled, _delete_history_ini, _prepare_prc_lkdtbo_files_for_model
from .port_scan import wait_for_preloader
from .proinfo_country import wait_and_patch_proinfo
from .scatter import disable_lk_dtbo_partitions, prepare_platform_scatter, apply_country_plan_to_proinfo, backup_platform_scatter_to_logs, ensure_prc_platform_scatter
from .utils import clear_console, log, log_text, log_model_value, classify_model_name, log_model_support_messages
from .i18n import get_string
from .constants import IMAGE_DIR, PLATFORM_TOOLS_DIR


def _spawn_quiet(cmd: list[str]) -> None:
    try:
        kwargs = {
            'stdout': subprocess.DEVNULL,
            'stderr': subprocess.DEVNULL,
            'stdin': subprocess.DEVNULL,
        }
        if hasattr(subprocess, 'CREATE_NO_WINDOW'):
            kwargs['creationflags'] = subprocess.CREATE_NO_WINDOW
        subprocess.Popen(cmd, **kwargs)
    except Exception:
        pass


def _trigger_reboot_commands() -> None:
    adb_path = PLATFORM_TOOLS_DIR / 'adb.exe'
    fastboot_path = PLATFORM_TOOLS_DIR / 'fastboot.exe'
    if adb_path.is_file():
        _spawn_quiet([str(adb_path), 'reboot'])
    else:
        _spawn_quiet(['adb', 'reboot'])
    time.sleep(0.2)
    if fastboot_path.is_file():
        _spawn_quiet([str(fastboot_path), 'reboot'])
    else:
        _spawn_quiet(['fastboot', 'reboot'])


def _ask_country_change_plan_proinfo() -> bool:
    base = get_string('country.change_plan_prompt_proinfo')
    while True:
        print(base, end='')
        try:
            raw = input().strip()
        except EOFError:
            raw = ''
        answer = raw.lower()
        line = f'{base}{raw}'
        from .utils import _write_log_line
        _write_log_line(line)
        if answer in ('y', 'yes'):
            return True
        if answer in ('n', 'no'):
            return False
        log('input.retry')


def _log_tb37x_warning(model: str) -> None:
    if should_show_tb37x_qna_warning(model):
        log('flow.tb37x_qna_warn')


def _ask_country_change_plan_reinstall() -> bool:
    base = get_string('country.change_plan_prompt_reinstall')
    while True:
        print(base, end='')
        try:
            raw = input().strip()
        except EOFError:
            raw = ''
        answer = raw.lower()
        line = f'{base}{raw}'
        from .utils import _write_log_line
        _write_log_line(line)
        if answer in ('y', 'yes'):
            return True
        if answer in ('n', 'no'):
            return False
        log('input.retry')


def _inspect_image_folder() -> tuple[str, str, str] | None:
    log('flow.firmware_version_detecting')
    if not IMAGE_DIR.is_dir() or not (IMAGE_DIR / 'download_agent').is_dir():
        log('flow.image_folder_missing')
        return None
    info = inspect_vendor_boot_image()
    model = (info.get('model') or '').strip().upper()
    version = (info.get('version') or '').strip().upper()
    rom_region = (info.get('rom_region') or '').strip().upper()
    if not model or not version or rom_region not in {'PRC', 'ROW'}:
        log('flow.firmware_version_not_found')
        return None
    platform = (inspect_flash_xml_platform() or '').strip().upper()
    if not platform:
        log('flow.flash_xml_missing')
        return None
    model = log_model_value('flow.reinstall.model', model, field_name='model')
    adb_state.LAST_IMAGE_MODEL = model
    adb_state.LAST_IMAGE_VERSION = version
    adb_state.LAST_IMAGE_ROM_REGION = rom_region
    adb_state.LAST_IMAGE_PLATFORM = platform
    adb_state.LAST_DEVICE_MODEL = model
    if classify_model_name(model) != 'supported':
        log_model_support_messages(model)
        return None
    if rom_region == 'ROW':
        log('flow.reinstall.image_folder_row')
    else:
        log('flow.reinstall.image_folder_prc')
    log('flow.reinstall.version', version=version)
    log('flow.reinstall.platform', platform=platform)
    log('flow.firmware_version_ok')
    log('flow.tb37x_qna_warn')
    return model, version, platform


def run_firmware_reinstall_flow() -> None:
    clear_console()
    log('app.menu.separator')
    log('flow.reinstall.start')
    log('app.menu.separator')
    log('flow.reinstall.stage1_header')
    info = _inspect_image_folder()
    if info is None:
        return
    model, version, platform = info
    _cleanup_before_flow()
    time.sleep(1)
    if not _prepare_prc_lkdtbo_files_for_model(model):
        return
    time.sleep(1)
    country_feature = _country_code_feature_enabled()
    if country_feature:
        change_plan = _ask_country_change_plan_reinstall()
    else:
        log('country.no_change')
        change_plan = False
    log_text('')
    log('flow.reinstall.stage2_header')
    log('flow.scatter_prepare')
    scatter_path = prepare_platform_scatter(platform, keep_user_data=False)
    if scatter_path is None:
        return
    time.sleep(1)
    disable_lk_dtbo_partitions(platform)
    time.sleep(1)
    apply_country_plan_to_proinfo(platform, change_plan)
    ensure_prc_platform_scatter(platform, preserve_userdata_false=False)
    time.sleep(1)
    _delete_history_ini()
    if change_plan:
        launch_spft_gui()
        wait_and_patch_proinfo(platform)
    else:
        if country_feature:
            log('country.no_change')
    backup_platform_scatter_to_logs(platform)
    log_text('')
    log('flow.reinstall.stage3_header')
    log('flow.reboot_stability')
    _trigger_reboot_commands()
    if not wait_for_preloader():
        return
    log_text('')
    log('flow.reinstall.stage4_header')
    ok = run_firmware_upgrade()
    _cleanup_after_flow(platform)
    if not ok:
        return
    log('flow.done')
