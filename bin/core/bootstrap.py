import os
import sys
import subprocess
import time
import json
from pathlib import Path
from .fw_upgrade_flow import run_firmware_upgrade_keep_data_flow
from .i18n import set_language, get_string
from .constants import PYTHON_DIR
from .utils import log, clear_console, kill_adb_server, enable_console_log_capture
from . import downloader

LANG_DIR = Path(__file__).resolve().parent / 'lang'
SETTINGS_PATH = LANG_DIR / 'settings.json'


def setup_console() -> None:
    try:
        import ctypes
        ctypes.windll.kernel32.SetConsoleTitleW('LPMBox')
        sys.stdout.write('\x1b[8;38;145t')
        sys.stdout.flush()
        os.system('mode con: cols=145 lines=38')
        try:
            handle = ctypes.windll.kernel32.GetStdHandle(-11)
            if handle not in (0, -1):
                class COORD(ctypes.Structure):
                    _fields_ = [
                        ("X", ctypes.c_short),
                        ("Y", ctypes.c_short),
                    ]
                buffer_size = COORD(145, 2000)
                ctypes.windll.kernel32.SetConsoleScreenBufferSize(handle, buffer_size)
        except Exception:
            pass
    except Exception:
        pass
def _is_embedded() -> bool:
    exe = Path(sys.executable).resolve()
    return exe.parent == PYTHON_DIR.resolve()


def _load_saved_language() -> str | None:
    try:
        if SETTINGS_PATH.is_file():
            data = json.loads(SETTINGS_PATH.read_text(encoding='utf-8', errors='ignore'))
            code = data.get('language')
            if isinstance(code, str) and code in ('ko', 'en', 'ru', 'jp'):
                return code
    except Exception:
        pass
    return None


def _save_language(code: str) -> None:
    try:
        SETTINGS_PATH.parent.mkdir(parents=True, exist_ok=True)
        SETTINGS_PATH.write_text(json.dumps({'language': code}, ensure_ascii=False, indent=2), encoding='utf-8')
    except Exception:
        pass


def _choose_language(force_prompt: bool = False) -> None:
    if not force_prompt:
        saved = _load_saved_language()
        if saved:
            set_language(saved)
            return
    while True:
        clear_console()
        separator = get_string('app.menu.separator')
        print()
        print(separator)
        print()
        options = [
            ('en', 'app.language_en'),
            ('ko', 'app.language_ko'),
            ('ru', 'app.language_ru'),
            ('jp', 'app.language_jp'),
        ]
        for idx, (code, key) in enumerate(options, start=1):
            label = get_string(key)
            print(f'  {idx}. {label}')
        print()
        print(separator)
        try:
            raw = input(get_string('app.language_prompt'))
        except EOFError:
            raw = ''
        choice = raw.strip()
        if not choice:
            code = 'ko'
            set_language(code)
            _save_language(code)
            return
        if not choice.isdigit():
            print(get_string('app.language_invalid'))
            time.sleep(1.5)
            continue
        num = int(choice)
        if 1 <= num <= len(options):
            code = options[num - 1][0]
            set_language(code)
            _save_language(code)
            return
        print(get_string('app.language_invalid'))
        time.sleep(1.5)


def _pause_back_to_menu() -> None:
    kill_adb_server()
    try:
        input(get_string('app.menu.back_to_menu'))
    except EOFError:
        pass


def _open_release_page() -> None:
    url = 'https://github.com/dwas-KR/LPMBox/releases'
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


def _check_for_updates(interactive: bool) -> None:
    from .constants import APP_VERSION
    from . import utils
    current_version = APP_VERSION
    try:
        print()
        print(get_string('update.checking'))
        latest_release, latest_prerelease = utils.get_latest_release_versions('dwas-KR', 'LPMBox')
        latest_version: str | None = None
        if latest_release:
            latest_version = latest_release
        if latest_prerelease and utils.is_update_available(latest_version or current_version, latest_prerelease):
            latest_version = latest_prerelease
        if latest_version and utils.is_update_available(current_version, latest_version):
            print(get_string('update.avail_title'))
            prompt = get_string('update.avail_prompt').format(curr=current_version, new=latest_version)
            try:
                choice = input(prompt).strip().lower()
            except EOFError:
                return
            if choice == 'y':
                print(get_string('update.open_web'))
                _open_release_page()
                if not interactive:
                    sys.exit(0)
        else:
            if interactive:
                print(get_string('update.no_update').format(version=current_version))
    except Exception as e:
        if interactive:
            try:
                print(get_string('update.error').format(e=e))
            except Exception:
                print(f'Failed to check for updates: {e}')


def _main_menu() -> None:
    from .global_flow import run_global_firmware_upgrade_flow
    from .fw_upgrade_flow import run_firmware_upgrade_keep_data_flow
    from .ota_disable_flow import run_ota_disable_flow
    from .mtk_driver import is_mtk_driver_installed, open_mtk_driver_site
    while True:
        clear_console()
        separator = get_string('app.menu.separator')
        print()
        print(separator)
        print(f"  {get_string('app.title')}")
        print(separator)
        print()
        print(f"    {get_string('app.menu.section_install')}")
        print(f"  1. {get_string('app.menu.option1')}")
        print(f"  2. {get_string('app.menu.option2')}")
        print(f"  3. {get_string('app.menu.option3')}")
        print()
        print(f"    {get_string('app.menu.section_other')}")
        driver_installed = is_mtk_driver_installed()
        status_key = 'app.mtk_driver.status_installed' if driver_installed else 'app.mtk_driver.status_needed'
        status = get_string(status_key)
        print(f"  4. {get_string('app.menu.option4')} {status}")
        print(f"  5. {get_string('app.menu.option5')}")
        print(f"  6. {get_string('app.menu.option6')}")
        print()
        print(f"  7. {get_string('app.menu.option7')}")
        print(f"  x. {get_string('app.menu.exit')}")
        print()
        print(separator)
        try:
            choice = input(get_string('app.menu.prompt')).strip().lower()
        except EOFError:
            break
        if choice in ('1', '2') and not driver_installed:
            log('app.mtk_driver.install_required')
            _pause_back_to_menu()
            continue
        if choice == '1':
            clear_console()
            print(get_string('app.title'))
            try:
                run_global_firmware_upgrade_flow()
            except KeyboardInterrupt:
                log('app.user_cancel')
            _pause_back_to_menu()
        elif choice == '2':
            clear_console()
            print(get_string('app.title'))
            try:
                run_firmware_upgrade_keep_data_flow()
            except KeyboardInterrupt:
                log('app.user_cancel')
            _pause_back_to_menu()
        elif choice == '3':
            clear_console()
            try:
                run_ota_disable_flow()
            except KeyboardInterrupt:
                log('app.user_cancel')
            _pause_back_to_menu()
        elif choice == '4':
            open_mtk_driver_site()
        elif choice == '5':
            clear_console()
            print(get_string('app.title'))
            _check_for_updates(interactive=True)
            _pause_back_to_menu()
        elif choice == '6':
            clear_console()
            _choose_language(force_prompt=True)
        elif choice == '7':
            try:
                os.startfile('http://www.youtube.com/@dwas_KR?sub_confirmation=1')
            except OSError:
                pass
        elif choice in ('x', 'q'):
            break
        else:
            print(get_string('app.menu.invalid_choice'))
            try:
                input()
            except EOFError:
                pass



def _acquire_single_instance_mutex():
    if os.name != 'nt':
        return 'Non-Windows-Mutex'
    try:
        import ctypes                        
    except Exception:
        return None
    kernel32 = ctypes.windll.kernel32
    mutex_name = "Global\\LPMBox_Singleton_Mutex"
    mutex = kernel32.CreateMutexW(None, False, mutex_name)
    if kernel32.GetLastError() == 183:
        return None
    return mutex


def main() -> None:
    setup_console()
    enable_console_log_capture()
    set_language('en')
    exe_embed = downloader.ensure_python_embed()
    if exe_embed is not None and (not _is_embedded()):
        log('bootstrap.embedded_restart')
        env = os.environ.copy()
        env['PYTHONUTF8'] = '1'
        env['PYTHONIOENCODING'] = 'utf-8'
        env['PYTHONPATH'] = str(PYTHON_DIR.parent)
        subprocess.run([str(exe_embed), '-m', 'core.bootstrap'], env=env, check=True)
        return
    clear_console()
    _choose_language()
    _check_for_updates(interactive=False)
    singleton = _acquire_single_instance_mutex()
    if not singleton:
        clear_console()
        try:
            print(get_string('app.already_running1'))
            print(get_string('app.already_running2'))
        except Exception:
            print("LPMBox is already running.\nPress Enter to close this window.")
        try:
            input()
        except EOFError:
            pass
        return
    log('bootstrap.start')
    downloader.ensure_platform_tools()
    downloader.ensure_spflashtool()
    ok_lkdtbo = downloader.ensure_lk_dtbo()
    if not ok_lkdtbo:
        try:
            input(get_string('app.press_enter'))
        except EOFError:
            pass
        kill_adb_server()
        return
    ok_crypto = downloader.ensure_cryptography()
    if not ok_crypto:
        try:
            input(get_string('app.press_enter'))
        except EOFError:
            pass
        kill_adb_server()
        return
    try:
        while True:
            try:
                _main_menu()
                break
            except KeyboardInterrupt:
                log('app.user_cancel')
                _pause_back_to_menu()
    finally:
        kill_adb_server()


if __name__ == '__main__':
    main()
