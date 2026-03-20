import platform
import sys
import shutil
import subprocess
import zipfile
import tempfile
import time
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen
from .constants import TOOLS_DIR, TOOLS_DOWNLOAD_DIR, PLATFORM_TOOLS_DIR, PLATFORM_TOOLS_URLS, SPFT_ZIP_URLS, PYTHON_DIR, PYTHON_VERSION, PYTHON_EMBED_URL_TEMPLATE, PYTHON_PTH_FILENAME, GET_PIP_URL, REQUIRED_PYTHON_PACKAGES, SPFT_EXE, LKDTBO_DIR, LKDTBO_MODEL_TO_ZIP, LKDTBO_GITHUB_COMMIT, LKDTBO_ZIP_URLS
from .utils import log, get_term_width
 
def _download_file(url: str, dest: Path) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    req = Request(url, headers={'User-Agent': 'Mozilla/5.0'})
    with urlopen(req, timeout=600) as resp:
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


def _download_from_list(urls: list[str], dest: Path) -> None:
    last_error: Exception | None = None
    for url in urls:
        try:
            _download_file(url, dest)
            return
        except (URLError, HTTPError, OSError) as e:
            last_error = e
    log('dl.download_failed')
    if last_error is not None:
        raise last_error

def _extract_zip(zip_path: Path, dest_dir: Path) -> None:
    dest_dir.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(zip_path, 'r') as zf:
        zf.extractall(dest_dir)

def _detect_arch() -> str:
    name = platform.machine().lower()
    if 'arm' in name:
        return 'arm64'
    if '64' in name or 'amd64' in name or 'x86_64' in name:
        return 'amd64'
    return 'win32'

def ensure_python_embed() -> Path | None:
    exe = PYTHON_DIR / 'python.exe'
    if exe.is_file():
        log('dl.skip_python')
        return exe
    arch = _detect_arch()
    url = PYTHON_EMBED_URL_TEMPLATE.format(version=PYTHON_VERSION, arch=arch)
    filename = f'python-{PYTHON_VERSION}-embed-{arch}.zip'
    zip_path = PYTHON_DIR / filename
    log('dl.python_downloading', arch=arch)
    try:
        _download_file(url, zip_path)
    except Exception:
        return None
    log('dl.python_extracting', filename=zip_path.name)
    _extract_zip(zip_path, PYTHON_DIR)
    pth = PYTHON_DIR / PYTHON_PTH_FILENAME
    content = 'python314.zip\n.\n..\\\n.\\Lib\\site-packages\nimport site\n'
    try:
        with pth.open('w', encoding='utf-8') as f:
            f.write(content)
    except Exception:
        pass
    target = PYTHON_DIR / 'Lib' / 'site-packages'
    try:
        target.mkdir(parents=True, exist_ok=True)
    except Exception:
        pass
    try:
        get_pip_py = PYTHON_DIR / 'get-pip.py'
        _download_file(GET_PIP_URL, get_pip_py)
    except Exception:
        return None
    try:
        cmd = [str(exe), str(get_pip_py)]
        subprocess.run(cmd, check=True)
    except Exception:
        return None
    try:
        cmd = [str(exe), '-m', 'pip', 'install', '--upgrade'] + REQUIRED_PYTHON_PACKAGES
        subprocess.run(cmd, check=True)
    except Exception:
        return None
    return exe

def ensure_platform_tools() -> None:
    adb = PLATFORM_TOOLS_DIR / 'adb.exe'
    if adb.is_file():
        log('dl.pt_skip')
        return
    zip_path = TOOLS_DOWNLOAD_DIR / 'platform-tools.zip'
    log('dl.pt_downloading')
    _download_from_list(PLATFORM_TOOLS_URLS, zip_path)
    log('dl.pt_extracting')
    _extract_zip(zip_path, TOOLS_DIR)
    if adb.is_file():
        log('dl.pt_ready', path=str(PLATFORM_TOOLS_DIR))

def _find_file_recursively(root: Path, name: str) -> Path | None:
    for path in root.rglob(name):
        if path.is_file():
            return path
    return None

def ensure_spflashtool() -> bool:
    if SPFT_EXE.is_file():
        log('dl.spft_skip')
        return True
    try:
        TOOLS_DIR.mkdir(parents=True, exist_ok=True)
    except Exception:
        pass
    zip_path = TOOLS_DOWNLOAD_DIR / 'SPFlashToolV6.zip'
    log('dl.spft_downloading')
    try:
        _download_from_list(SPFT_ZIP_URLS, zip_path)
    except Exception:
        log('dl.download_failed')
        return False
    log('dl.spft_extracting')
    _extract_zip(zip_path, TOOLS_DIR)
    extracted_dir = TOOLS_DIR / 'SP_Flash_Tool_V6.2404_Win'
    if extracted_dir.is_dir():
        for item in extracted_dir.iterdir():
            dest = TOOLS_DIR / item.name
            try:
                if dest.exists():
                    if dest.is_file():
                        dest.unlink()
                    elif dest.is_dir():
                        shutil.rmtree(dest)
                shutil.move(str(item), str(dest))
            except Exception:
                pass
        try:
            shutil.rmtree(extracted_dir)
        except Exception:
            pass
    if SPFT_EXE.is_file():
        log('dl.spft_ready', path=str(SPFT_EXE))
        return True
    log('dl.spft_missing_after_extract')
    return False

def ensure_cryptography() -> bool:
    log('dl.check_crypto')
    try:
        import cryptography                        
        log('dl.crypto_skip')
        return True
    except Exception:
        pass
    exe = PYTHON_DIR / 'python.exe'
    if not exe.is_file():
        log('dl.crypto_failed')
        return False
    cmd = [str(exe), '-m', 'pip', 'install'] + REQUIRED_PYTHON_PACKAGES
    try:
        log('dl.crypto_install')
        subprocess.run(cmd, check=True)
        import cryptography                        
        log('dl.crypto_ready')
        return True
    except Exception:
        log('dl.crypto_failed')
        return False


def ensure_lkdtbo_zip_for_model(model: str) -> Path | None:
    name = LKDTBO_MODEL_TO_ZIP.get(model)
    if not name:
        return None
    dest = TOOLS_DOWNLOAD_DIR / name
    if dest.is_file():
        return dest
    urls = list(LKDTBO_ZIP_URLS.get(name, []))
    urls.extend([
        f'https://raw.githubusercontent.com/dwas-KR/LPMBox/{LKDTBO_GITHUB_COMMIT}/{name}',
        f'https://github.com/dwas-KR/LPMBox/raw/{LKDTBO_GITHUB_COMMIT}/{name}',
    ])
    try:
        _download_from_list(urls, dest)
    except Exception:
        return None
    return dest

def extract_lkdtbo_zip(zip_path: Path, dest_dir: Path | None=None) -> bool:
    if dest_dir is None:
        dest_dir = LKDTBO_DIR
    dest_dir.mkdir(parents=True, exist_ok=True)
    targets = ['lk_a', 'lk_b', 'dtbo_a', 'dtbo_b']
    for t in targets:
        p = dest_dir / t
        if p.exists():
            try:
                p.unlink()
            except OSError:
                pass
    try:
        with tempfile.TemporaryDirectory() as td:
            tmp = Path(td)
            _extract_zip(zip_path, tmp)
            for t in targets:
                found = None
                for p in tmp.rglob('*'):
                    if not p.is_file():
                        continue
                    n = p.name.lower()
                    if n == t.lower() or n == f'{t}.img' or n == f'{t}.bin':
                        found = p
                        break
                if found is None:
                    return False
                shutil.copy2(found, dest_dir / t)
    except Exception:
        return False
    return True
