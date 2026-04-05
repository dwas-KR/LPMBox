from __future__ import annotations
import time
from . import adb_utils as adb_state
from .flash_spft import launch_spft_gui, run_firmware_upgrade, prepare_flash_files
from .firmware_guard import inspect_vendor_boot_image, inspect_flash_xml_platform
from .global_flow import _cleanup_after_flow, _cleanup_before_flow, _normalize_rom_region
from .port_scan import wait_for_preloader
from .proinfo_country import wait_and_patch_proinfo
from .scatter import prepare_country_reset_scatter
from .utils import clear_console, log, log_text, wait_for_device, adb_shell_getprop, run_adb, log_model_value, classify_model_name, log_model_support_messages


def _detect_platform_country_reset() -> tuple[str, str, str, str] | None:
    version = (adb_shell_getprop('ro.build.version.release') or '').strip()
    platform = (adb_shell_getprop('ro.vendor.mediatek.platform') or '').strip()
    model_raw = (adb_shell_getprop('ro.product.model') or '').strip()
    model = log_model_value('flow.device_info_value', model_raw, field_name='hw')
    adb_state.LAST_DEVICE_MODEL = model
    region = _normalize_rom_region(adb_shell_getprop('ro.config.zui.region') or '')
    adb_state.LAST_DEVICE_ROM_REGION = region
    adb_state.LAST_ANDROID_VERSION_RELEASE = version
    adb_state.LAST_MTK_PLATFORM = platform
    if region:
        log('flow.keep_data.rom_type', region=region)
    if version:
        log('flow.android_version_detected', version=version)
    log('flow.platform', platform=platform)
    category = classify_model_name(model)
    if category != 'supported':
        log_model_support_messages(model)
        return None
    if not platform or not platform.startswith('MT'):
        log('flow.not_mtk')
        return None
    return model, region, version, platform


def _inspect_image_folder_country_reset(platform: str) -> bool:
    from .constants import IMAGE_DIR
    log('flow.firmware_version_detecting')
    if not IMAGE_DIR.is_dir() or not (IMAGE_DIR / 'download_agent').is_dir():
        log('flow.image_folder_missing')
        return False
    info = inspect_vendor_boot_image()
    rom_region = (info.get('rom_region') or '').strip().upper()
    if rom_region == 'ROW':
        log('flow.reinstall.image_folder_row')
    elif rom_region == 'PRC':
        log('flow.reinstall.image_folder_prc')
    else:
        log('flow.firmware_version_not_found')
        return False
    log('flow.firmware_version_ok')
    flash_platform = (inspect_flash_xml_platform() or '').strip().upper()
    if flash_platform != platform:
        log('flow.platform_mismatch')
        return False
    log('flow.flash_xml_ok', platform=platform)
    adb_state.LAST_IMAGE_ROM_REGION = rom_region
    adb_state.LAST_IMAGE_PLATFORM = flash_platform
    return True


def _ask_country_change_plan() -> bool:
    code = (adb_shell_getprop('ro.product.countrycode') or '').strip().upper() or 'UNKNOWN'
    from .i18n import get_string
    prompt = get_string('country.change_plan_prompt').format(code=code)
    while True:
        print(prompt, end='')
        try:
            raw = input().strip()
        except EOFError:
            raw = ''
        from .utils import _write_log_line
        _write_log_line(f'{prompt}{raw}')
        answer = raw.lower()
        if answer in ('y', 'yes'):
            return True
        if answer in ('n', 'no'):
            return False
        log('input.retry')


def run_country_code_reset_flow() -> None:
    clear_console()
    log('app.menu.separator')
    log('flow.country_reset.start')
    log('app.menu.separator')
    log('flow.country_reset.stage1_header')
    if not wait_for_device():
        return
    detected = _detect_platform_country_reset()
    if detected is None:
        return
    _model, _region, _version, platform = detected
    _cleanup_before_flow()
    time.sleep(1)
    log_text('')
    log('flow.country_reset.stage2_header')
    if not _inspect_image_folder_country_reset(platform):
        return
    time.sleep(1)
    log_text('')
    log('flow.country_reset.stage3_header')
    log('flow.scatter_prepare')
    scatter_path = prepare_country_reset_scatter(platform)
    if scatter_path is None:
        return
    time.sleep(1)
    log_text('')
    log('flow.country_reset.stage4_header')
    change_plan = _ask_country_change_plan()
    if not change_plan:
        log('country.no_change')
        return
    if not prepare_flash_files():
        return
    launch_spft_gui()
    wait_and_patch_proinfo(platform)
    log_text('')
    log('flow.country_reset.stage5_header')
    if not wait_for_device():
        return
    log('flow.rebooting')
    try:
        run_adb(['reboot'], capture_output=True)
    except Exception:
        pass
    if not wait_for_preloader():
        return
    ok = run_firmware_upgrade()
    _cleanup_after_flow(platform)
    if not ok:
        return
    log('flow.done')
