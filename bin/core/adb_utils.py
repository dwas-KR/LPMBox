import subprocess
from pathlib import Path
from .constants import PLATFORM_TOOLS_DIR
 
def _adb_path() -> str:
    adb = PLATFORM_TOOLS_DIR / 'adb.exe'
    if adb.is_file():
        return str(adb)
    return 'adb'

def adb_shell_getprop(name: str) -> str:
    cmd = [_adb_path(), 'shell', 'getprop', name]
    try:
        out = subprocess.check_output(cmd, stderr=subprocess.STDOUT)
    except Exception:
        return ''
    try:
        return out.decode('utf-8', errors='ignore').strip()
    except Exception:
        return ''

def adb_reboot() -> None:
    adb = _adb_path()
    try:
        subprocess.run([adb, 'reboot'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    except Exception:
        pass
    try:
        subprocess.run([adb, 'kill-server'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    except Exception:
        pass

def kill_adb_server() -> None:
    adb = _adb_path()
    try:
        subprocess.run([adb, 'kill-server'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    except Exception:
        pass
