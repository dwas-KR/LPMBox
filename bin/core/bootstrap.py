import os
import sys
import subprocess
import time
import json
from pathlib import Path
from .fw_upgrade_flow import run_firmware_upgrade_keep_data_flow
from .i18n import set_language, get_string
from .constants import PYTHON_DIR
from .utils import log, clear_console, kill_adb_server, enable_console_log_capture, TerminalMenu
from . import downloader



def _load_settings() -> dict:
    try:
        if SETTINGS_PATH.is_file():
            data = json.loads(SETTINGS_PATH.read_text(encoding='utf-8', errors='ignore'))
            if isinstance(data, dict):
                return data
    except Exception:
        pass
    return {}

def _save_settings(data: dict) -> None:
    try:
        SETTINGS_PATH.parent.mkdir(parents=True, exist_ok=True)
        SETTINGS_PATH.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding='utf-8')
    except Exception:
        pass
LANG_DIR = Path(__file__).resolve().parent / 'lang'
SETTINGS_PATH = LANG_DIR / 'settings.json'


def setup_console() -> None:
    try:
        import ctypes
        if os.name == 'nt':
            kernel32 = ctypes.windll.kernel32
            try:
                kernel32.SetConsoleTitleW('LPMBox')
            except Exception:
                pass
            try:
                STD_INPUT_HANDLE = -10
                ENABLE_QUICK_EDIT_MODE = 0x0040
                ENABLE_EXTENDED_FLAGS = 0x0080
                hStdIn = kernel32.GetStdHandle(STD_INPUT_HANDLE)
                mode = ctypes.c_uint32()
                if kernel32.GetConsoleMode(hStdIn, ctypes.byref(mode)):
                    mode.value &= ~ENABLE_QUICK_EDIT_MODE
                    mode.value |= ENABLE_EXTENDED_FLAGS
                    kernel32.SetConsoleMode(hStdIn, mode.value)
            except Exception:
                pass
            try:
                STD_OUTPUT_HANDLE = -11
                ENABLE_VIRTUAL_TERMINAL_PROCESSING = 0x0004
                hStdOut = kernel32.GetStdHandle(STD_OUTPUT_HANDLE)
                out_mode = ctypes.c_uint32()
                if kernel32.GetConsoleMode(hStdOut, ctypes.byref(out_mode)):
                    out_mode.value |= ENABLE_VIRTUAL_TERMINAL_PROCESSING
                    kernel32.SetConsoleMode(hStdOut, out_mode.value)
            except Exception:
                pass
        sys.stdout.write('\x1b[8;38;145t')
        sys.stdout.flush()
        os.system('mode con: cols=145 lines=38')
        try:
            import ctypes as _ct
            handle = _ct.windll.kernel32.GetStdHandle(-11)
            if handle not in (0, -1):
                class COORD(_ct.Structure):
                    _fields_ = [
                        ('X', _ct.c_short),
                        ('Y', _ct.c_short),
                    ]
                buffer_size = COORD(145, 2000)
                _ct.windll.kernel32.SetConsoleScreenBufferSize(handle, buffer_size)
        except Exception:
            pass
    except Exception:
        pass
def _quickedit_enabled() -> bool:
    if os.name != 'nt':
        return False
    try:
        import ctypes
        kernel32 = ctypes.windll.kernel32
        hStdIn = kernel32.GetStdHandle(-10)
        mode = ctypes.c_uint32()
        if kernel32.GetConsoleMode(hStdIn, ctypes.byref(mode)):
            return bool(mode.value & 0x0040)
    except Exception:
        return False
    return False

def _is_embedded() -> bool:
    exe = Path(sys.executable).resolve()
    return exe.parent == PYTHON_DIR.resolve()


def _load_saved_language() -> str | None:
    try:
        data = _load_settings()
        code = data.get('language')
        if isinstance(code, str):
            c = code.strip().lower()
            if c in ('zh_tw', 'zh_hk'):
                return 'zh_cn'
            if c == 'en_au':
                return 'en'
            if c in ('ko', 'en', 'ru', 'jp', 'el', 'zh_cn', 'vi', 'ka', 'nl'):
                return c
    except Exception:
        pass
    return None


def _save_language(code: str) -> None:
    try:
        data = _load_settings()
        data['language'] = code
        _save_settings(data)
    except Exception:
        pass


def _choose_language(force_prompt: bool = False) -> None:
    options = [
        ('1', 'en', 'app.language_en'),
        ('2', 'ko', 'app.language_ko'),
        ('3', 'ru', 'app.language_ru'),
        ('4', 'jp', 'app.language_jp'),
        ('5', 'zh_cn', 'app.language_zh_cn'),
        ('6', 'vi', 'app.language_vi'),
        ('7', 'el', 'app.language_el'),
        ('8', 'ka', 'app.language_ka'),
        ('9', 'nl', 'app.language_nl'),
    ]
    if not force_prompt:
        saved = _load_saved_language()
        if saved:
            set_language(saved)
            return
    while True:
        try:
            menu = TerminalMenu(get_string('app.language_title'), breadcrumbs=get_string('breadcrumb.settings'))
            for key, _, text_key in options:
                menu.add_option(key, get_string(text_key))
            choice = menu.ask(prompt=get_string('app.language_prompt'), default_key='1')
        except KeyboardInterrupt:
            code = 'ko'
            set_language(code)
            _save_language(code)
            return
        for key, code, _ in options:
            if choice == key:
                set_language(code)
                _save_language(code)
                return


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
        data = _load_settings()
        if not interactive:
            last = data.get('last_update_check')
            try:
                last_ts = float(last) if last is not None else 0.0
            except Exception:
                last_ts = 0.0
            if time.time() - last_ts < 6 * 3600:
                return
        print()
        print(get_string('update.checking'))
        channel = data.get('update_channel')
        include_prerelease = False
        if isinstance(channel, str):
            c = channel.strip().lower()
            if c in ('prerelease', 'pre', 'preview', 'beta', 'rc', 'all', 'include_prerelease'):
                include_prerelease = True
        info = utils.get_latest_release_info('dwas-KR', 'LPMBox', include_prerelease=include_prerelease)
        latest_version: str | None = None
        if isinstance(info, dict):
            tag = info.get('tag')
            if isinstance(tag, str) and tag:
                latest_version = tag
        if latest_version and utils.is_update_available(current_version, latest_version):
            print(get_string('update.avail_title'))
            print(get_string('update.avail_warning'))
            prompt = get_string('update.avail_prompt').format(curr=current_version, new=latest_version)
            try:
                choice = input(prompt).strip().lower()
            except EOFError:
                return
            if choice == 'y':
                try:
                    if isinstance(info, dict):
                        assets = info.get('assets')
                        if isinstance(assets, list):
                            zip_asset = utils.find_release_zip_asset(assets)
                            if isinstance(zip_asset, dict):
                                name = zip_asset.get('name')
                                if isinstance(name, str) and name:
                                    expected = utils.get_asset_expected_sha256(assets, name)
                                    if expected:
                                        print(f'Checksum (sha256): {expected}  ({name})')
                                    auto = data.get('update_auto_download')
                                    if auto is True:
                                        url = zip_asset.get('browser_download_url')
                                        if isinstance(url, str) and url:
                                            updates_dir = Path(__file__).resolve().parents[2] / 'updates'
                                            updates_dir.mkdir(parents=True, exist_ok=True)
                                            dest = updates_dir / name
                                            print(f'Downloading update: {name}')
                                            utils.download_url(url, dest)
                                            actual = utils.sha256_file(dest)
                                            if expected and actual.lower() != expected.lower():
                                                print('[!] Checksum mismatch. Please re-download the update file.')
                                            else:
                                                print(f'Downloaded: {dest}')
                except Exception:
                    pass
                print(get_string('update.open_web'))
                _open_release_page()
                if not interactive:
                    sys.exit(0)
        else:
            if interactive:
                print(get_string('update.no_update').format(version=current_version))
        data['last_update_check'] = str(time.time())
        _save_settings(data)
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
        driver_installed = is_mtk_driver_installed()
        status_key = 'app.mtk_driver.status_installed' if driver_installed else 'app.mtk_driver.status_needed'
        status = get_string(status_key)
        menu = TerminalMenu(get_string('app.title'), breadcrumbs=get_string('breadcrumb.main'))
        menu.add_label(get_string('app.menu.section_install'))
        menu.add_option('1', get_string('app.menu.option1'))
        menu.add_option('2', get_string('app.menu.option2'))
        menu.add_option('3', get_string('app.menu.option3'))
        menu.add_separator()
        menu.add_label(get_string('app.menu.section_other'))
        menu.add_option('4', f"{get_string('app.menu.option4')} {status}")
        menu.add_option('5', get_string('app.menu.option5'))
        menu.add_option('6', get_string('app.menu.option6'))
        menu.add_separator()
        menu.add_option('7', get_string('app.menu.option7'))
        menu.add_option('x', get_string('app.menu.exit'))
        try:
            choice = menu.ask(prompt=get_string('app.menu.prompt'), default_key='1')
        except KeyboardInterrupt:
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
    if _quickedit_enabled():
        try:
            print('[!] ' + get_string('console.quickedit_warn'))
        except Exception:
            print('[!] QuickEdit mode is enabled. Disable QuickEdit in the console properties.')
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
