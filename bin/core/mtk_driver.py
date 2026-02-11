import os
import subprocess

def _check_with_pnputil() -> bool:
    if os.name != 'nt':
        return False
    try:
        completed = subprocess.run(['pnputil', '/enum-drivers'], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True, encoding='utf-8', errors='ignore', check=False)
    except OSError:
        return False
    output = completed.stdout or ''
    lower_text = output.lower()
    if 'mediatek sp driver' in lower_text:
        return True
    if 'mediatek usb port' in lower_text:
        return True
    return False
 
def _check_with_registry() -> bool:
    if os.name != 'nt':
        return False
    try:
        import winreg
    except ImportError:
        return False
    uninstall_paths = ['SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall', 'SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall']
    for root in (winreg.HKEY_LOCAL_MACHINE,):
        for path in uninstall_paths:
            try:
                with winreg.OpenKey(root, path) as key:
                    subkey_count = winreg.QueryInfoKey(key)[0]
                    for index in range(subkey_count):
                        try:
                            subkey_name = winreg.EnumKey(key, index)
                            with winreg.OpenKey(key, subkey_name) as subkey:
                                try:
                                    display_name, _ = winreg.QueryValueEx(subkey, 'DisplayName')
                                except FileNotFoundError:
                                    continue
                                if isinstance(display_name, str) and 'mediatek sp driver' in display_name.lower():
                                    return True
                        except OSError:
                            continue
            except FileNotFoundError:
                continue
    return False

def is_mtk_driver_installed() -> bool:
    if _check_with_pnputil():
        return True
    if _check_with_registry():
        return True
    return False

def open_mtk_driver_site() -> None:
    url = 'https://mtkdriver.com/mtk-driver-v5-2307'
    if os.name == 'nt':
        try:
            os.startfile(url)
            return
        except OSError:
            pass
    try:
        import webbrowser
        webbrowser.open(url)
    except Exception:
        pass
