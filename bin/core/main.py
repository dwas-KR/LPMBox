from .i18n import set_language, get_string
from .global_flow import run_global_firmware_upgrade_flow

def main():
    set_language('en')
    print(get_string('app.title'))
    run_global_firmware_upgrade_flow()
    input(get_string('app.press_enter'))
if __name__ == '__main__':
    main()
