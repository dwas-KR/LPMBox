import subprocess
from pathlib import Path
from .constants import TOOLS_DIR, SPFT_EXE, FLASH_XML_DLAGENT, FLASH_XML_ROOT, DA_AUTH_DLAGENT, DA_AUTH_ROOT
from .utils import log, log_text, _write_log_line

def _resolve_flash_xml() -> Path | None:
    if FLASH_XML_DLAGENT.is_file():
        return FLASH_XML_DLAGENT
    if FLASH_XML_ROOT.is_file():
        return FLASH_XML_ROOT
    return None
 
def _resolve_da_auth() -> Path | None:
    if DA_AUTH_DLAGENT.is_file():
        return DA_AUTH_DLAGENT
    if DA_AUTH_ROOT.is_file():
        return DA_AUTH_ROOT
    return None

def _update_history_ini(flash_xml: Path, da_auth: Path) -> None:
    ini_path = TOOLS_DIR / 'history.ini'
    flash_value = str(flash_xml.resolve())
    auth_value = str(da_auth.resolve())
    lines: list[str] = []
    if ini_path.is_file():
        try:
            text = ini_path.read_text(encoding='utf-8', errors='ignore')
        except Exception:
            text = ''
        if text:
            lines = text.splitlines()
    new_lines: list[str] = []
    found_flash = False
    found_auth = False
    for raw in lines:
        stripped = raw.lstrip()
        if stripped.startswith('flashxmlHistory='):
            new_lines.append('flashxmlHistory=' + flash_value)
            found_flash = True
        elif stripped.startswith('lastAuthDir='):
            new_lines.append('lastAuthDir=' + auth_value)
            found_auth = True
        else:
            new_lines.append(raw)
    if not found_flash:
        new_lines.append('flashxmlHistory=' + flash_value)
    if not found_auth:
        new_lines.append('lastAuthDir=' + auth_value)
    text_out = '\n'.join(new_lines) + '\n'
    try:
        ini_path.write_text(text_out, encoding='utf-8')
    except Exception:
        pass

def prepare_flash_files() -> bool:
    ini_path = TOOLS_DIR / 'history.ini'
    try:
        if ini_path.is_file():
            ini_path.unlink()
    except Exception:
        pass
    flash_xml = _resolve_flash_xml()
    if flash_xml is None:
        log('flow.no_flash_xml')
        return False
    da_auth = _resolve_da_auth()
    if da_auth is None:
        log('flow.no_da_auth')
        return False
    _update_history_ini(flash_xml, da_auth)
    log('flow.copy_flash_done')
    return True

def launch_spft_gui() -> None:
    exe = SPFT_EXE
    if not exe.is_file():
        log('flash.no_spft')
        return
    try:
        subprocess.Popen([str(exe)], cwd=str(TOOLS_DIR))
        log('flash.gui_started')
    except Exception:
        log('flash.no_spft')

def run_firmware_upgrade() -> bool:
    exe = SPFT_EXE
    if not exe.is_file():
        log('flash.no_spft')
        return False

    flash_xml = _resolve_flash_xml()
    if flash_xml is None:
        return False

    da_auth = _resolve_da_auth()
    if da_auth is None:
        return False

    cmd = [
        str(exe),
        '-a',
        str(da_auth),
        '-f',
        str(flash_xml),
        '-c',
        'download',
    ]
    command_str = f'"{exe}" -a "{da_auth}" -f "{flash_xml}" -c download'
    log_text(f'user input cmds: {command_str}')

    try:
        proc = subprocess.Popen(
            cmd,
            cwd=str(TOOLS_DIR),
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            bufsize=1,
            universal_newlines=True,
        )
    except FileNotFoundError:
        log('flash.no_spft')
        return False
    except Exception as e:
        log_text(str(e))
        log('flash.failed', code=-1)
        return False

    capture_started = False
    code: int | None = None
    try:
        assert proc.stdout is not None
        import sys as _sys
        out = _sys.stdout
        prev_flag = getattr(out, '_lpmbox_suppress_capture', False)
        for raw_line in proc.stdout:
            line = raw_line.rstrip('\r\n')
            if not line:
                continue
            setattr(out, '_lpmbox_suppress_capture', True)
            try:
                print(line)
            finally:
                setattr(out, '_lpmbox_suppress_capture', prev_flag)
            _write_log_line(line)
        code = proc.wait()
    except KeyboardInterrupt:
        print()
        try:
            proc.kill()
        except Exception:
            pass
        raise
    except Exception as e:
        log_text(str(e))
        code = proc.poll()
    finally:
        try:
            if proc.stdout is not None:
                proc.stdout.close()
        except Exception:
            pass
        if code is None:
            try:
                code = proc.wait()
            except Exception:
                code = -1

    if code == 0:
        log('flash.done')
        return True
    log('flash.failed', code=code or -1)
    return False