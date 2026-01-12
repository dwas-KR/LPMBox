import os
import sys
import subprocess
import time
import json
from pathlib import Path

from .fw_upgrade_flow import run_firmware_upgrade_keep_data_flow
from .i18n import set_language, get_string
from .constants import PYTHON_DIR
from .utils import log, clear_console, kill_adb_server
from . import downloader

LANG_DIR = Path(__file__).resolve().parent / "lang"
SETTINGS_PATH = LANG_DIR / "settings.json"


def setup_console() -> None:
    try:
        import ctypes
        ctypes.windll.kernel32.SetConsoleTitleW("LPMBox")
        sys.stdout.write("\x1b[8;38;145t")
        sys.stdout.flush()
        os.system("mode con: cols=145 lines=38")
    except Exception:
        pass


def _is_embedded() -> bool:
    exe = Path(sys.executable).resolve()
    return exe.parent == PYTHON_DIR.resolve()


def _load_saved_language() -> str | None:
    try:
        if SETTINGS_PATH.is_file():
            data = json.loads(SETTINGS_PATH.read_text(encoding="utf-8", errors="ignore"))
            code = data.get("language")
            if isinstance(code, str) and code in ("ko", "en", "ru", "jp"):
                return code
    except Exception:
        pass
    return None


def _save_language(code: str) -> None:
    try:
        SETTINGS_PATH.parent.mkdir(parents=True, exist_ok=True)
        SETTINGS_PATH.write_text(
            json.dumps({"language": code}, ensure_ascii=False, indent=2),
            encoding="utf-8",
        )
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
        print()
        separator = "=" * 78
        print(separator)
        print(f"   {get_string('app.language_menu_title')}")
        print(separator)
        print()
        options = [
            ("en", "app.language_en"),
            ("ko", "app.language_ko"),
            ("ru", "app.language_ru"),
            ("jp", "app.language_jp"),
        ]
        for idx, (code, key) in enumerate(options, start=1):
            label = get_string(key)
            print(f"   {idx}. {label}")
        print()
        print(separator)
        try:
            raw = input(get_string("app.language_prompt"))
        except EOFError:
            raw = ""
        choice = raw.strip()
        if not choice:
            code = "ko"
            set_language(code)
            _save_language(code)
            return
        if not choice.isdigit():
            print(get_string("app.language_invalid"))
            time.sleep(1.5)
            continue
        num = int(choice)
        if 1 <= num <= len(options):
            code = options[num - 1][0]
            set_language(code)
            _save_language(code)
            return
        print(get_string("app.language_invalid"))
        time.sleep(1.5)


def _main_menu() -> None:
    from .global_flow import run_global_firmware_upgrade_flow
    from .fw_upgrade_flow import run_firmware_upgrade_keep_data_flow
    from .ota_disable_flow import run_ota_disable_flow
    from .mtk_driver import is_mtk_driver_installed, open_mtk_driver_site

    while True:
        clear_console()
        separator = get_string("app.menu.separator")
        print(separator)
        print(get_string("app.title"))
        print(separator)
        print()
        print(get_string("app.menu.section_install"))
        print(f" 1. {get_string('app.menu.option1')}")
        print(f" 2. {get_string('app.menu.option2')}")
        print()
        print(get_string("app.menu.section_other"))

        driver_installed = is_mtk_driver_installed()
        status_key = "app.mtk_driver.status_installed" if driver_installed else "app.mtk_driver.status_needed"
        status = get_string(status_key)

        print(f" 3. {get_string('app.menu.option3')}")
        print(f" 4. {get_string('app.menu.option4')} {status}")
        print(f" 5. {get_string('app.menu.option5')}")
        print(f" x. {get_string('app.menu.exit')}")
        print()
        print(separator)
        try:
            choice = input(get_string("app.menu.prompt")).strip().lower()
        except EOFError:
            break

        if choice == "1":
            clear_console()
            print(get_string("app.title"))
            run_global_firmware_upgrade_flow()
            try:
                input(get_string("app.menu.back_to_menu"))
            except EOFError:
                pass
        elif choice == "2":
            clear_console()
            print(get_string("app.title"))
            run_firmware_upgrade_keep_data_flow()
            try:
                input(get_string("app.menu.back_to_menu"))
            except EOFError:
                pass
        elif choice == "3":
            clear_console()
            print(get_string("app.title"))
            run_ota_disable_flow()
            try:
                input(get_string("app.menu.back_to_menu"))
            except EOFError:
                pass
        elif choice == "4":
            open_mtk_driver_site()
        elif choice == "5":
            try:
                os.startfile("http://www.youtube.com/@dwas_KR?sub_confirmation=1")
            except OSError:
                pass
        elif choice in ("x", "q"):
            break
        else:
            print(get_string("app.menu.invalid_choice"))
            try:
                input()
            except EOFError:
                pass


def main() -> None:
    setup_console()
    set_language("en")
    exe_embed = downloader.ensure_python_embed()
    if exe_embed is not None and not _is_embedded():
        log("bootstrap.embedded_restart")
        env = os.environ.copy()
        env["PYTHONUTF8"] = "1"
        env["PYTHONIOENCODING"] = "utf-8"
        env["PYTHONPATH"] = str(PYTHON_DIR.parent)
        subprocess.run([str(exe_embed), "-m", "core.bootstrap"], env=env, check=True)
        return

    clear_console()
    _choose_language()
    log("bootstrap.start")
    downloader.ensure_platform_tools()
    downloader.ensure_spflashtool()
    downloader.ensure_prc()
    downloader.ensure_lk_dtbo()
    ok_crypto = downloader.ensure_cryptography()
    if not ok_crypto:
        try:
            input(get_string("app.press_enter"))
        except EOFError:
            pass
        kill_adb_server()
        return

    try:
        _main_menu()
    finally:
        kill_adb_server()


if __name__ == "__main__":
    main()
