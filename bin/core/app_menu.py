from .i18n import get_string
from .utils import clear_console
 
def _pause_back_to_menu() -> None:
    try:
        input(get_string('app.menu.back_to_menu'))
    except EOFError:
        pass

def _pause() -> None:
    try:
        input()
    except EOFError:
        pass

def run_main_menu() -> None:
    from .global_flow import run_global_firmware_upgrade_flow
    from .fw_upgrade_flow import run_firmware_upgrade_keep_data_flow
    while True:
        clear_console()
        print(get_string('app.title'))
        print()
        print(get_string('app.menu.separator'))
        print(f" 1. {get_string('app.menu.option1')}")
        print(f" 2. {get_string('app.menu.option2')}")
        print(f" x. {get_string('app.menu.exit')}")
        print(get_string('app.menu.separator'))
        try:
            raw = input(get_string('app.menu.prompt'))
        except EOFError:
            raw = ''
        choice = raw.strip().lower()
        if choice == '1':
            clear_console()
            print(get_string('app.title'))
            run_global_firmware_upgrade_flow()
            _pause_back_to_menu()
        elif choice == '2':
            clear_console()
            print(get_string('app.title'))
            run_firmware_upgrade_keep_data_flow()
            _pause_back_to_menu()
        elif choice in ('x', 'q', 'exit'):
            break
