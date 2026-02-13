from .i18n import get_string
from .utils import log, log_text, wait_for_device, run_adb, kill_adb_server

def _apply_ota_settings() -> None:
    settings = [('global', 'ota_disable_automatic_update', '1'), ('global', 'setup_wizard_privacy_auto_update', '0'), ('global', 'setup_wizard_privacy_ota_key', '0'), ('system', 'ota_network_permission', '0'), ('secure', 'lenovo_ota_new_version_found', '0')]
    for scope, key, value in settings:
        run_adb(['shell', 'settings', 'put', scope, key, value], capture_output=True)

def _uninstall_ota_packages() -> None:
    packages = ['com.lenovo.ota', 'com.tblenovo.lenovowhatsnew', 'com.lenovo.tbengine']
    for pkg in packages:
        run_adb(['shell', 'pm', 'uninstall', '-k', '--user', '0', pkg], capture_output=True)
 
def run_ota_disable_flow() -> None:
    separator = get_string('app.menu.separator')
    log_text(separator)
    log_text(get_string('ota.task_title'))
    log_text(separator)
    log('ota.start')
    ok = wait_for_device()
    if not ok:
        kill_adb_server()
        return
    log('ota.adb_connected')
    log('ota.disabling')
    try:
        _apply_ota_settings()
        _uninstall_ota_packages()
    finally:
        kill_adb_server()
    log('ota.finished')
    log_text(separator)
