import platform
import shutil
import subprocess
import zipfile
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen
from .constants import TOOLS_DIR, TOOLS_DOWNLOAD_DIR, PLATFORM_TOOLS_DIR, PLATFORM_TOOLS_URLS, SPFT_ZIP_URLS, PYTHON_DIR, PYTHON_VERSION, PYTHON_EMBED_URL_TEMPLATE, PYTHON_PTH_FILENAME, GET_PIP_URL, REQUIRED_PYTHON_PACKAGES, SPFT_EXE, LK_DTBO_ZIP_URLS, LK_DTBO_DIR, LK_DTBO_ZIP_NAME, LK_DTBO_FILES
from .utils import log
 
def _download_file(url: str, dest: Path) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    req = Request(url, headers={'User-Agent': 'Mozilla/5.0'})
    with urlopen(req, timeout=600) as resp:
        with dest.open('wb') as f:
            shutil.copyfileobj(resp, f)

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


def _normalize_lk_dtbo_extract(dest_dir: Path) -> None:
    if not dest_dir.is_dir():
        return
    items = [p for p in dest_dir.iterdir()]
    if len(items) == 1 and items[0].is_dir():
        inner = items[0]
        for child in inner.iterdir():
            try:
                shutil.move(str(child), str(dest_dir / child.name))
            except Exception:
                pass
        try:
            shutil.rmtree(inner)
        except Exception:
            pass

def _ensure_lk_dtbo_filenames(dest_dir: Path) -> None:
    for name in LK_DTBO_FILES:
        direct = dest_dir / name
        if direct.is_file():
            continue
        alt = dest_dir / f"{name}.img"
        if alt.is_file():
            try:
                alt.rename(direct)
            except Exception:
                try:
                    shutil.copy2(alt, direct)
                except Exception:
                    pass

def ensure_lk_dtbo() -> bool:
    ok = True
    for name in LK_DTBO_FILES:
        if not (LK_DTBO_DIR / name).is_file():
            ok = False
            break
    if ok:
        log('dl.lkdtbo_ready')
        return True
    zip_path = TOOLS_DOWNLOAD_DIR / LK_DTBO_ZIP_NAME
    log('dl.lkdtbo_downloading')
    try:
        _download_from_list(LK_DTBO_ZIP_URLS, zip_path)
    except Exception:
        log('dl.download_failed')
        return False
    log('dl.lkdtbo_extracting')
    try:
        if LK_DTBO_DIR.exists():
            shutil.rmtree(LK_DTBO_DIR)
    except Exception:
        pass
    try:
        LK_DTBO_DIR.mkdir(parents=True, exist_ok=True)
    except Exception:
        pass
    try:
        _extract_zip(zip_path, LK_DTBO_DIR)
        _normalize_lk_dtbo_extract(LK_DTBO_DIR)
        _ensure_lk_dtbo_filenames(LK_DTBO_DIR)
    except Exception:
        log('dl.download_failed')
        return False
    for name in LK_DTBO_FILES:
        if not (LK_DTBO_DIR / name).is_file():
            log('dl.lkdtbo_missing_after_extract')
            return False
    log('dl.lkdtbo_ready')
    return True

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
