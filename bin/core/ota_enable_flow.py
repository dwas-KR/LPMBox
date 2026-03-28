from __future__ import annotations
import time
from .adb_utils import kill_adb_server
from .utils import clear_console, log, wait_for_device, adb_shell_getprop, run_adb

_PACKAGES = (
    'com.zui.homesettings',
    'com.lenovo.tbengine',
    'com.lenovo.ue.device',
    'com.lenovo.ota',
    'com.zui.safecenter',
)

def _restore_ota_packages() -> None:
    for pkg in _PACKAGES:
        try:
            run_adb(['shell', 'cmd', 'package', 'install-existing', '--user', '0', pkg], capture_output=True)
        except Exception:
            pass
    for pkg in _PACKAGES:
        try:
            run_adb(['shell', 'pm', 'enable', '--user', '0', pkg], capture_output=True)
        except Exception:
            pass

def run_ota_enable_flow() -> None:
    clear_console()
    log('app.menu.separator')
    log('ota_enable.task_title')
    log('app.menu.separator')
    log('ota_enable.start')
    if not wait_for_device():
        return
    region = (adb_shell_getprop('ro.config.zui.region') or '').strip()
    region_upper = ''.join(region.split()).upper()
    log('flow.keep_data.rom_type', region=region_upper if region_upper else region)
    if region_upper == 'ROW':
        log('ota_enable.rom_row_warn')
        return
    if region_upper == 'PRC':
        log('flow.prc.rom_prc_ok')
    log('ota_enable.enabling')
    _restore_ota_packages()
    time.sleep(1)
    log('ota_enable.done')
    log('ota.software_update_hint')
    kill_adb_server()
