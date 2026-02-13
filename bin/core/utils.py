import os
import subprocess
import time
import json
import urllib.request
from datetime import datetime
from pathlib import Path
from .constants import LOGS_DIR, LOG_ENV_VAR, PLATFORM_TOOLS_DIR
from .i18n import get_string
_log_file_path: Path | None = None
_unauthorized_hint_shown: bool = False

def _init_log_file() -> None:
    global _log_file_path
    env_path = os.environ.get(LOG_ENV_VAR)
    if env_path:
        _log_file_path = Path(env_path).resolve()
        _log_file_path.parent.mkdir(parents=True, exist_ok=True)
        return
    LOGS_DIR.mkdir(parents=True, exist_ok=True)
    ts = datetime.now().strftime('%Y.%m.%d.%H.%M')
    _log_file_path = (LOGS_DIR / f'run_{ts}.log').resolve()
 
def _write_log_line(line: str) -> None:
    global _log_file_path
    if _log_file_path is None:
        _init_log_file()
    try:
        assert _log_file_path is not None
        with _log_file_path.open('a', encoding='utf-8') as f:
            f.write(line + '\n')
    except Exception:
        pass

def log(message_key: str, **kwargs) -> None:
    msg = get_string(message_key)
    if kwargs:
        try:
            msg = msg.format(**kwargs)
        except Exception:
            pass
    ts = datetime.now().strftime('%H:%M:%S')
    line = f'{ts} - {msg}'
    print(line)
    _write_log_line(line)

def log_text(text: str) -> None:
    ts = datetime.now().strftime('%H:%M:%S')
    line = f'{ts} - {text}'
    print(line)
    _write_log_line(line)
def log_text_file_only(text: str) -> None:
    if text.strip() == "":
        return
    ts = datetime.now().strftime("%H:%M:%S")
    line = f"{ts} - {text}"
    _write_log_line(line)



def clear_console() -> None:
    try:
        os.system('cls')
    except Exception:
        pass

def find_adb_path() -> str:
    adb = PLATFORM_TOOLS_DIR / 'adb.exe'
    if adb.is_file():
        return str(adb)
    return 'adb'

def run_adb(args: list[str], capture_output: bool=True) -> subprocess.CompletedProcess:
    adb = find_adb_path()
    cmd = [adb] + args
    return subprocess.run(cmd, capture_output=capture_output, text=True, encoding='utf-8', errors='replace')

def kill_adb_server() -> None:
    try:
        run_adb(['kill-server'], capture_output=True)
    except Exception:
        pass

def wait_for_device(timeout_sec: int | None=None) -> bool:
    global _unauthorized_hint_shown
    log('adb.wait_usb_debugging')
    start = time.time()
    while True:
        if timeout_sec is not None and time.time() - start > timeout_sec:
            log('adb.timeout')
            return False
        try:
            cp = run_adb(['devices'], capture_output=True)
            out = (cp.stdout or '') + '\n' + (cp.stderr or '')
            lines = [x.strip() for x in out.splitlines() if x.strip()]
            devices: list[tuple[str, str]] = []
            for line in lines:
                if line.startswith('List of devices'):
                    continue
                if '\t' in line:
                    sn, state = line.split('\t', 1)
                    devices.append((sn.strip(), state.strip()))
            if not devices:
                time.sleep(2)
                continue
            for sn, state in devices:
                if state == 'device':
                    log('adb.device_ok', serial=sn)
                    return True
                if state == 'unauthorized' and (not _unauthorized_hint_shown):
                    _unauthorized_hint_shown = True
                    log('adb.unauthorized_hint')
        except Exception:
            pass
        time.sleep(2)

def adb_shell_getprop(prop: str) -> str:
    cp = run_adb(['shell', 'getprop', prop], capture_output=True)
    value = (cp.stdout or '').strip()
    return value

def adb_reboot() -> None:
    run_adb(['reboot'], capture_output=True)

def run_cmd(cmd: list[str], cwd: str | None=None, timeout: int | None=None) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, cwd=cwd, timeout=timeout, capture_output=True, text=True, encoding='utf-8', errors='replace')

def run_powershell(ps_script: str) -> subprocess.CompletedProcess:
    return subprocess.run(['powershell', '-NoProfile', '-ExecutionPolicy', 'Bypass', '-Command', ps_script], capture_output=True, text=True, encoding='utf-8', errors='replace')

def ensure_dir(p: Path) -> None:
    p.mkdir(parents=True, exist_ok=True)

def safe_unlink(p: Path) -> None:
    try:
        if p.is_file():
            p.unlink()
    except Exception:
        pass

def safe_rmtree(p: Path) -> None:
    if not p.exists():
        return
    try:
        if p.is_file():
            p.unlink()
            return
    except Exception:
        pass
    try:
        for child in p.rglob('*'):
            try:
                if child.is_file():
                    child.unlink()
            except Exception:
                pass
        for child in sorted(p.rglob('*'), reverse=True):
            try:
                if child.is_dir():
                    child.rmdir()
            except Exception:
                pass
        try:
            p.rmdir()
        except Exception:
            pass
    except Exception:
        pass


def _version_to_tuple(version: str) -> tuple[int, ...]:
    v = version.strip()
    if v.startswith('v') or v.startswith('V'):
        v = v[1:]
    parts = v.split('.')
    numbers: list[int] = []
    for part in parts:
        try:
            numbers.append(int(part))
        except ValueError:
            numbers.append(0)
    return tuple(numbers)


def is_update_available(current: str, latest: str) -> bool:
    try:
        return _version_to_tuple(latest) > _version_to_tuple(current)
    except Exception:
        return False


def get_latest_release_versions(repo_owner: str, repo_name: str) -> tuple[str | None, str | None]:
    url = f'https://api.github.com/repos/{repo_owner}/{repo_name}/releases?per_page=20'
    latest_release: str | None = None
    latest_prerelease: str | None = None
    try:
        with urllib.request.urlopen(url, timeout=5) as response:
            if response.status != 200:
                return (None, None)
            data = json.loads(response.read().decode('utf-8', errors='ignore'))
    except Exception:
        return (None, None)
    if not isinstance(data, list):
        return (None, None)
    for release in data:
        if not isinstance(release, dict):
            continue
        if release.get('draft'):
            continue
        tag = release.get('tag_name')
        if not isinstance(tag, str) or not tag:
            continue
        if release.get('prerelease'):
            if latest_prerelease is None or is_update_available(latest_prerelease, tag):
                latest_prerelease = tag
        else:
            if latest_release is None or is_update_available(latest_release, tag):
                latest_release = tag
    return (latest_release, latest_prerelease)

