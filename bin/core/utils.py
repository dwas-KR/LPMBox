import os
import subprocess
import time
import sys
import io
import json
import urllib.request
import shutil
import re
import hashlib
try:
    import msvcrt
except Exception:
    msvcrt = None
from datetime import datetime
from pathlib import Path
from typing import Any
from .constants import LOGS_DIR, LOG_ENV_VAR, PLATFORM_TOOLS_DIR
from .i18n import get_string
_log_file_path: Path | None = None
_unauthorized_hint_shown: bool = False

_ANSI_RESET = '\x1b[0m'
_ANSI_GREEN = '\x1b[33m'

def _ansi_enabled() -> bool:
    if os.environ.get('LPMBOX_NO_COLOR'):
        return False
    try:
        return sys.stdout.isatty()
    except Exception:
        return False

def _colorize_line(msg: str, line: str) -> str:
    m = msg.strip()
    if not m or not _ansi_enabled():
        return line
    if m.startswith('[!]'):
        return _ANSI_GREEN + line + _ANSI_RESET
    return line


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


_original_stdout = None
_original_stderr = None
_console_logger_enabled = False

class _ConsoleLogger:
    def __init__(self, original):
        self._original = original
        self._buffer = ""
        self._at_line_start = True
        self._color_active = False

    def write(self, s):
        if not s:
            return 0
        out_s = s
        try:
            if _ansi_enabled() and ('\x1b' not in s):
                res: list[str] = []
                i = 0
                while i < len(s):
                    if self._at_line_start and (not self._color_active) and s.startswith('[!]', i):
                        self._color_active = True
                        res.append(_ANSI_GREEN)
                    ch = s[i]
                    if ch == '\r' and self._color_active:
                        res.append(_ANSI_RESET)
                        self._color_active = False
                        self._at_line_start = True
                        res.append(ch)
                        i += 1
                        continue
                    res.append(ch)
                    if ch == '\n':
                        if self._color_active:
                            res.append(_ANSI_RESET)
                            self._color_active = False
                        self._at_line_start = True
                    elif ch == '\r':
                        self._at_line_start = True
                    else:
                        if self._at_line_start:
                            self._at_line_start = False
                    i += 1
                out_s = ''.join(res)
        except Exception:
            out_s = s
        try:
            self._original.write(out_s)
        except Exception:
            pass
        if getattr(self, '_lpmbox_suppress_capture', False):
            return len(s)
        self._buffer += s
        while '\n' in self._buffer:
            line, self._buffer = self._buffer.split('\n', 1)
            text = line.rstrip('\r')
            if text:
                try:
                    _write_log_line(text)
                except Exception:
                    pass
        return len(s)

    def flush(self):
        try:
            self._original.flush()
        except Exception:
            pass

    def isatty(self):
        try:
            return bool(self._original.isatty())
        except Exception:
            return False

    def fileno(self):
        try:
            return int(self._original.fileno())
        except Exception:
            raise OSError()

    @property
    def encoding(self):
        return getattr(self._original, 'encoding', 'utf-8')

    @property
    def errors(self):
        return getattr(self._original, 'errors', 'replace')

def enable_console_log_capture() -> None:
    global _original_stdout, _original_stderr, _console_logger_enabled
    if _console_logger_enabled:
        return
    _console_logger_enabled = True
    if _log_file_path is None:
        _init_log_file()
    try:
        import sys as _sys
        logger = _ConsoleLogger(_sys.stdout)
        _original_stdout = _sys.stdout
        _original_stderr = _sys.stderr
        _sys.stdout = logger
        _sys.stderr = logger
    except Exception:
        _console_logger_enabled = False


def log(message_key: str, **kwargs) -> None:
    msg = get_string(message_key)
    if kwargs:
        try:
            msg = msg.format(**kwargs)
        except Exception:
            pass
    line = msg
    display_line = _colorize_line(msg, line)
    try:
        import sys as _sys
        out = _sys.stdout
        prev = getattr(out, '_lpmbox_suppress_capture', False)
        setattr(out, '_lpmbox_suppress_capture', True)
        try:
            print(display_line)
        finally:
            setattr(out, '_lpmbox_suppress_capture', prev)
    except Exception:
        print(display_line)
    _write_log_line(line)

def log_text(text: str) -> None:
    line = text
    try:
        import sys as _sys
        out = _sys.stdout
        prev = getattr(out, '_lpmbox_suppress_capture', False)
        try:
            setattr(out, '_lpmbox_suppress_capture', True)
            print(line)
        finally:
            setattr(out, '_lpmbox_suppress_capture', prev)
    except Exception:
        print(line)
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
    data = None
    try:
        req = urllib.request.Request(url, headers={'User-Agent': 'LPMBox', 'Accept': 'application/vnd.github+json'})
        with urllib.request.urlopen(req, timeout=7) as response:
            if getattr(response, 'status', 200) != 200:
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


def _github_api_get_json(url: str, timeout: int = 7) -> Any:
    try:
        req = urllib.request.Request(url, headers={'User-Agent': 'LPMBox', 'Accept': 'application/vnd.github+json'})
        with urllib.request.urlopen(req, timeout=timeout) as response:
            if getattr(response, 'status', 200) != 200:
                return None
            return json.loads(response.read().decode('utf-8', errors='ignore'))
    except Exception:
        return None


def get_latest_release_info(repo_owner: str, repo_name: str, include_prerelease: bool = False) -> dict | None:
    url = f'https://api.github.com/repos/{repo_owner}/{repo_name}/releases?per_page=40'
    data = _github_api_get_json(url, timeout=7)
    if not isinstance(data, list):
        return None
    best: dict | None = None
    for release in data:
        if not isinstance(release, dict):
            continue
        if release.get('draft'):
            continue
        if (not include_prerelease) and release.get('prerelease'):
            continue
        tag = release.get('tag_name')
        if not isinstance(tag, str) or not tag:
            continue
        if best is None or is_update_available(best.get('tag', '0'), tag):
            best = {
                'tag': tag,
                'html_url': release.get('html_url'),
                'assets': release.get('assets') if isinstance(release.get('assets'), list) else [],
                'prerelease': bool(release.get('prerelease')),
            }
    return best


def _parse_sha256_manifest(text: str) -> dict[str, str]:
    out: dict[str, str] = {}
    for raw in text.splitlines():
        line = raw.strip()
        if not line:
            continue
        if line.startswith('#'):
            continue
        m = re.match(r'^[a-fA-F0-9]{64}\s+\*?(.+)$', line)
        if m:
            h = line.split()[0].lower()
            fn = m.group(1).strip()
            out[fn] = h
            continue
        m2 = re.match(r'^SHA256\s*\((.+)\)\s*=\s*([a-fA-F0-9]{64})$', line)
        if m2:
            fn = m2.group(1).strip()
            h = m2.group(2).lower()
            out[fn] = h
    return out


def _fetch_text(url: str, timeout: int = 10) -> str | None:
    try:
        req = urllib.request.Request(url, headers={'User-Agent': 'LPMBox'})
        with urllib.request.urlopen(req, timeout=timeout) as response:
            if getattr(response, 'status', 200) != 200:
                return None
            return response.read().decode('utf-8', errors='ignore')
    except Exception:
        return None


def find_release_zip_asset(assets: list[dict]) -> dict | None:
    if not assets:
        return None
    zips: list[dict] = []
    for a in assets:
        if not isinstance(a, dict):
            continue
        name = a.get('name')
        if not isinstance(name, str):
            continue
        low = name.lower()
        if not low.endswith('.zip'):
            continue
        if 'source' in low:
            continue
        zips.append(a)
    if not zips:
        return None
    def score(a: dict) -> int:
        name = str(a.get('name') or '').lower()
        s = 0
        for k in ('win', 'windows', 'x64', 'amd64', 'x86', 'x86_x64'):
            if k in name:
                s += 2
        if 'lpmbox' in name:
            s += 3
        return s
    zips.sort(key=score, reverse=True)
    return zips[0]


def get_asset_expected_sha256(assets: list[dict], target_name: str) -> str | None:
    for a in assets:
        if not isinstance(a, dict):
            continue
        name = a.get('name')
        if name == target_name:
            digest = a.get('digest')
            if isinstance(digest, str):
                low = digest.strip().lower()
                if low.startswith('sha256:'):
                    return low.split(':', 1)[1].strip()
    checksum_assets: list[dict] = []
    for a in assets:
        if not isinstance(a, dict):
            continue
        name = a.get('name')
        if not isinstance(name, str):
            continue
        low = name.lower()
        if 'sha256' in low or 'checksum' in low or 'checksums' in low:
            checksum_assets.append(a)
    for a in checksum_assets:
        url = a.get('browser_download_url')
        if not isinstance(url, str) or not url:
            continue
        txt = _fetch_text(url, timeout=10)
        if not txt:
            continue
        m = _parse_sha256_manifest(txt)
        if target_name in m:
            return m[target_name]
        base = Path(target_name).name
        if base in m:
            return m[base]
    return None


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as f:
        while True:
            chunk = f.read(1024 * 1024)
            if not chunk:
                break
            h.update(chunk)
    return h.hexdigest()

def download_url(url: str, dest: Path) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    req = urllib.request.Request(url, headers={'User-Agent': 'LPMBox'})
    with urllib.request.urlopen(req, timeout=600) as resp:
        total = 0
        try:
            total = int(resp.headers.get('Content-Length') or 0)
        except Exception:
            total = 0
        done = 0
        start = time.time()
        last_draw = 0.0
        ncols = get_term_width(145)
        def fmt_bytes(n: int) -> str:
            units = ['B', 'K', 'M', 'G', 'T']
            v = float(n)
            u = 0
            while v >= 1024.0 and u < len(units) - 1:
                v /= 1024.0
                u += 1
            if u == 0:
                return f'{int(v)}{units[u]}'
            if v >= 100:
                return f'{v:.0f}{units[u]}'
            if v >= 10:
                return f'{v:.1f}{units[u]}'
            return f'{v:.2f}{units[u]}'
        def fmt_time(sec: float) -> str:
            s = int(sec + 0.5)
            m, s = divmod(s, 60)
            h, m = divmod(m, 60)
            if h > 0:
                return f'{h:02d}:{m:02d}:{s:02d}'
            return f'{m:02d}:{s:02d}'
        def draw(final: bool = False) -> None:
            nonlocal last_draw
            now = time.time()
            if (not final) and (now - last_draw) < 0.06:
                return
            last_draw = now
            if total <= 0:
                return
            pct = done / total if total else 0.0
            if pct < 0.0:
                pct = 0.0
            if pct > 1.0:
                pct = 1.0
            percent = int(pct * 100.0 + 0.5)
            elapsed = now - start
            rate = done / elapsed if elapsed > 0 else 0.0
            remaining = (total - done) / rate if rate > 0 else 0.0
            l_bar = f'{percent:3d}%|'
            r_bar = f'| {fmt_bytes(done)}/{fmt_bytes(total)} [{fmt_time(elapsed)}<{fmt_time(remaining)}]'
            bar_width = ncols - len(l_bar) - len(r_bar)
            if bar_width < 10:
                bar_width = 10
            filled = int(bar_width * pct + 0.5)
            if filled < 0:
                filled = 0
            if filled > bar_width:
                filled = bar_width
            bar = ('█' * filled) + (' ' * (bar_width - filled))
            line = (l_bar + bar + r_bar).ljust(ncols)
            try:
                out = sys.stdout
                prev = getattr(out, '_lpmbox_suppress_capture', False)
                setattr(out, '_lpmbox_suppress_capture', True)
                try:
                    out.write('\r' + line)
                    if final:
                        out.write('\n')
                    out.flush()
                finally:
                    setattr(out, '_lpmbox_suppress_capture', prev)
            except Exception:
                pass
        with dest.open('wb') as f:
            while True:
                chunk = resp.read(1024 * 64)
                if not chunk:
                    break
                f.write(chunk)
                done += len(chunk)
                draw(False)
        if total > 0:
            done = total
            draw(True)





def capture_spft_console_output_snapshot() -> None:
    import os
    if os.name != 'nt':
        return
    try:
        import ctypes
        from ctypes import wintypes
    except Exception:
        return
    try:
        kernel32 = ctypes.WinDLL('kernel32', use_last_error=True)
    except Exception:
        return
    STD_OUTPUT_HANDLE = -11

    class COORD(ctypes.Structure):
        _fields_ = [('X', wintypes.SHORT), ('Y', wintypes.SHORT)]

    class SMALL_RECT(ctypes.Structure):
        _fields_ = [('Left', wintypes.SHORT), ('Top', wintypes.SHORT), ('Right', wintypes.SHORT), ('Bottom', wintypes.SHORT)]

    class CONSOLE_SCREEN_BUFFER_INFO(ctypes.Structure):
        _fields_ = [
            ('dwSize', COORD),
            ('dwCursorPosition', COORD),
            ('wAttributes', wintypes.WORD),
            ('srWindow', SMALL_RECT),
            ('dwMaximumWindowSize', COORD),
        ]

    handle = kernel32.GetStdHandle(STD_OUTPUT_HANDLE)
    if not handle or handle == ctypes.c_void_p(-1).value:
        return
    csbi = CONSOLE_SCREEN_BUFFER_INFO()
    if not kernel32.GetConsoleScreenBufferInfo(handle, ctypes.byref(csbi)):
        return
    width = csbi.dwSize.X
    height = csbi.dwSize.Y
    if width <= 0 or height <= 0:
        return
    length = int(width) * int(height)
    if length <= 0:
        return
    if length > 120 * 2000:
        length = 120 * 2000
    buffer = ctypes.create_unicode_buffer(length)
    read = wintypes.DWORD(0)
    origin = COORD(0, 0)
    if not kernel32.ReadConsoleOutputCharacterW(handle, buffer, length, origin, ctypes.byref(read)):
        return
    chars = buffer[:read.value]
    text = ''.join(chars)
    lines = text.splitlines()
    if not lines:
        return
    keywords = (
        'Smart Phone Flash Tool',
        'Begin',
        'CMD:',
        'has been sent',
        'data has been sent',
        'DA data has been sent',
        'scan device',
        'connect brom',
        'EMMC-CONTROL',
        'XML schema',
        'verify load images checksum',
        'All command exec done',
        'Command line arguments',
        'Build Time:',
        'Init config from input arguments',
        'com port clue',
        'enter DA mode',
        'DA SLA',
        'scatter-file',
        'image data has been sent',
        'MediaTek PreLoader',
        'download_agent',
        'image',
        'about',
        'of',
    )
    seen = set()
    for line in lines:
        s = line.strip()
        if not s:
            continue
        if not any(k in s for k in keywords):
            continue
        if s in seen:
            continue
        seen.add(s)
        _write_log_line(s)


def get_term_width(fallback: int = 80) -> int:
    try:
        return int(shutil.get_terminal_size((fallback, 20)).columns)
    except Exception:
        return int(fallback)

def _repeat_sep(sep_template: str, width: int) -> str:
    s = (sep_template or '').strip()
    ch = '='
    for c in s:
        if not c.isspace():
            ch = c
            break
    if width < 20:
        width = 20
    return ch * width

class TerminalMenu:
    def __init__(self, title: str, breadcrumbs: str | None = None):
        self.title = title
        self.breadcrumbs = breadcrumbs
        self.items: list[tuple[str | None, str, bool]] = []
        self.valid_keys: list[str] = []

    def add_option(self, key: str, text: str) -> None:
        self.items.append((key, text, True))
        self.valid_keys.append(key.lower())

    def add_label(self, text: str) -> None:
        self.items.append((None, text, False))

    def add_separator(self) -> None:
        self.items.append((None, '', False))

    def _selectable_indexes(self) -> list[int]:
        idxs: list[int] = []
        for i, (k, _, sel) in enumerate(self.items):
            if sel and k is not None:
                idxs.append(i)
        return idxs

    def _build_lines(self, current_index: int | None, prompt: str | None) -> tuple[list[str], dict[int, int], dict[int, tuple[str, str]]]:
        width = get_term_width(145)
        sep = _repeat_sep(get_string('app.menu.separator'), width)
        lines: list[str] = []
        row_map: dict[int, int] = {}
        text_map: dict[int, tuple[str, str]] = {}
        lines.append('')
        lines.append(sep)
        display_title = self.title
        if self.breadcrumbs:
            display_title = f'{self.breadcrumbs} > {self.title}'
        lines.append(f'  {display_title}')
        lines.append(sep)
        lines.append('')
        for i, (k, text, selectable) in enumerate(self.items):
            if selectable and k is not None:
                sel_line = f' -> {k}. {text}'
                unsel_line = f'    {k}. {text}'
                row = len(lines) + 1
                row_map[i] = row
                text_map[i] = (sel_line, unsel_line)
                lines.append(sel_line if current_index == i else unsel_line)
            else:
                lines.append(f'    {text}' if text else '')
        lines.append('')
        lines.append(sep)
        if prompt:
            lines.append(prompt)
            try:
                lines.append(get_string('prompt.use_arrow_keys'))
            except Exception:
                try:
                    lines.append(get_string('prompt_use_arrow_keys'))
                except Exception:
                    lines.append('Use arrow keys to navigate, Enter to select.')
        return (lines, row_map, text_map)

    def _write_suppressed(self, s: str) -> None:
        try:
            import sys as _sys
            out = _sys.stdout
            prev = getattr(out, '_lpmbox_suppress_capture', False)
            setattr(out, '_lpmbox_suppress_capture', True)
            try:
                out.write(s)
                out.flush()
            finally:
                setattr(out, '_lpmbox_suppress_capture', prev)
        except Exception:
            try:
                print(s, end='')
            except Exception:
                pass

    def _rewrite_row(self, base_row: int, target_row: int, text: str) -> None:
        up = base_row - target_row
        if up < 0:
            up = 0
        try:
            import sys as _sys
            out = _sys.stdout
            prev = getattr(out, '_lpmbox_suppress_capture', False)
            setattr(out, '_lpmbox_suppress_capture', True)
            try:
                if up:
                    out.write(f'\x1b[{up}A')
                out.write('\r\x1b[2K')
                out.write(text)
                if up:
                    out.write(f'\x1b[{up}B')
                out.flush()
            finally:
                setattr(out, '_lpmbox_suppress_capture', prev)
        except Exception:
            pass

    def ask(self, prompt: str | None = None, default_key: str | None = None) -> str:
        selectable = self._selectable_indexes()
        if not selectable:
            raise KeyboardInterrupt()
        cur = selectable[0]
        if default_key:
            dk = default_key.lower()
            for i in selectable:
                k = self.items[i][0]
                if k and k.lower() == dk:
                    cur = i
                    break
        if msvcrt is None or os.name != 'nt':
            clear_console()
            lines, _, _ = self._build_lines(None, None)
            for line in lines:
                print(line)
            while True:
                try:
                    raw = input(prompt or '')
                except EOFError:
                    raise KeyboardInterrupt()
                choice = raw.strip().lower()
                if choice in self.valid_keys:
                    return choice
                print(get_string('app.menu.invalid_choice'))
        clear_console()
        try:
            self._write_suppressed('\x1b[?25l')
        except Exception:
            pass
        lines, row_map, text_map = self._build_lines(cur, prompt)
        content = '\n'.join(lines) + '\n'
        self._write_suppressed(content)
        base_row = len(lines) + 1
        if prompt:
            self._write_suppressed('')
        prev_sel = cur
        while True:
            ch = msvcrt.getwch()
            if ch in ('\r', '\n'):
                key = self.items[cur][0]
                if key is None:
                    continue
                try:
                    self._write_suppressed('\x1b[?25h')
                except Exception:
                    pass
                return key.lower()
            if ch == '\x1b':
                try:
                    self._write_suppressed('\x1b[?25h')
                except Exception:
                    pass
                raise KeyboardInterrupt()
            if ch in ('\x00', '\xe0'):
                code = msvcrt.getwch()
                if code == 'H':
                    pos = selectable.index(cur)
                    cur = selectable[(pos - 1) % len(selectable)]
                elif code == 'P':
                    pos = selectable.index(cur)
                    cur = selectable[(pos + 1) % len(selectable)]
                else:
                    continue
                if cur != prev_sel:
                    r_prev = row_map.get(prev_sel)
                    r_cur = row_map.get(cur)
                    if r_prev is not None and prev_sel in text_map:
                        self._rewrite_row(base_row, r_prev, text_map[prev_sel][1])
                    if r_cur is not None and cur in text_map:
                        self._rewrite_row(base_row, r_cur, text_map[cur][0])
                    prev_sel = cur
                continue
            if ch.isdigit():
                buf = ch
                start = time.time()
                while time.time() - start < 0.25:
                    if not msvcrt.kbhit():
                        time.sleep(0.01)
                        continue
                    nxt = msvcrt.getwch()
                    if nxt.isdigit():
                        buf += nxt
                        start = time.time()
                    else:
                        break
                choice = buf.lower()
                if choice in self.valid_keys:
                    try:
                        self._write_suppressed('\x1b[?25h')
                    except Exception:
                        pass
                    return choice
            ch_low = ch.lower()
            if ch_low in self.valid_keys:
                try:
                    self._write_suppressed('\x1b[?25h')
                except Exception:
                    pass
                return ch_low
