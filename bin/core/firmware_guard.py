from __future__ import annotations
from pathlib import Path
import re
from .constants import BLOCK_FIRMWARE_INI, IMAGE_DIR, FLASH_XML_DLAGENT, FLASH_XML_ROOT
from . import adb_utils as adb_state
from .utils import log

_VERSION_RE = re.compile(rb'(?:ZUI|ZUXOS)_[0-9]+(?:\.[0-9]+)+_[A-Z]+')
_MODEL_RE = re.compile(rb'TB\d{3}[A-Z]{2}')
_FLASH_PLATFORM_RE = re.compile(r'<scatter>\.\./(MT\d+)_Android_scatter\.xml</scatter>', re.IGNORECASE)


def _read_bytes(path: Path) -> bytes | None:
    try:
        return path.read_bytes()
    except Exception:
        return None


def _detect_section(data: bytes, version: str | None = None) -> str | None:
    if b'CN_OPEN_USER' in data or b'PRC_OPEN_USER' in data:
        return 'PRC ROM'
    if b'ROW_OPEN_USER' in data or b'OW_OPEN_USER' in data:
        return 'ROW ROM'
    if version:
        up = version.upper()
        if up.startswith('ZUXOS_'):
            return 'PRC ROM'
    return None


def _extract_unique(pattern: re.Pattern[bytes], data: bytes) -> list[str]:
    found: list[str] = []
    for match in pattern.findall(data):
        value = match.decode('ascii', 'ignore').strip().upper()
        if value and value not in found:
            found.append(value)
    return found


def _extract_version(data: bytes) -> str | None:
    found = _extract_unique(_VERSION_RE, data)
    if len(found) == 1:
        return found[0]
    return None


def _extract_model(data: bytes) -> str | None:
    found = _extract_unique(_MODEL_RE, data)
    if len(found) == 1:
        return found[0]
    return None


def _load_blocked_versions(path: Path) -> dict[str, dict[str, set[str]]]:
    result: dict[str, dict[str, set[str]]] = {}
    current = ''
    try:
        lines = path.read_text(encoding='utf-8', errors='ignore').splitlines()
    except Exception:
        return result
    for raw in lines:
        line = raw.strip()
        if not line:
            continue
        if line.startswith('//'):
            probe = line[2:].strip()
            if probe.startswith('[') and probe.endswith(']'):
                current = probe[1:-1].strip().upper()
                result.setdefault(current, {})
            continue
        if line.startswith('[') and line.endswith(']'):
            current = line[1:-1].strip().upper()
            result.setdefault(current, {})
            continue
        if '=' not in line or not current:
            continue
        key, value = line.split('=', 1)
        model = key.strip().upper()
        version = value.strip().upper()
        if not model or not version:
            continue
        result.setdefault(current, {}).setdefault(model, set()).add(version)
    return result


def inspect_vendor_boot_image() -> dict[str, str]:
    image_path = IMAGE_DIR / 'vendor_boot-debug.img'
    info = {
        'model': '',
        'version': '',
        'rom_region': '',
        'rom_label': '',
    }
    if not image_path.is_file():
        return info
    data = _read_bytes(image_path)
    if not data:
        return info
    version = _extract_version(data) or ''
    model = _extract_model(data) or ''
    section = _detect_section(data, version)
    rom_region = ''
    rom_label = ''
    if section == 'PRC ROM':
        rom_region = 'PRC'
        rom_label = 'PRC(중국 내수롬)'
    elif section == 'ROW ROM':
        rom_region = 'ROW'
        rom_label = 'ROW(글로벌롬)'
    info.update({
        'model': model,
        'version': version,
        'rom_region': rom_region,
        'rom_label': rom_label,
    })
    return info


def inspect_flash_xml_platform() -> str | None:
    for flash_xml in (FLASH_XML_DLAGENT, FLASH_XML_ROOT):
        if not flash_xml.is_file():
            continue
        try:
            text = flash_xml.read_text(encoding='utf-8', errors='ignore')
        except Exception:
            continue
        match = _FLASH_PLATFORM_RE.search(text)
        if match:
            return match.group(1).upper()
    return None


def detect_vendor_boot_rom_type() -> str | None:
    info = inspect_vendor_boot_image()
    return info.get('rom_region') or None


def should_show_tb37x_qna_warning(*models: str) -> bool:
    targets = {'TB373FU', 'TB375FC'}
    for model in models:
        if (model or '').strip().upper() in targets:
            return True
    return False


def validate_firmware_image() -> bool:
    log('flow.firmware_version_detecting')
    if not IMAGE_DIR.is_dir() or not (IMAGE_DIR / 'download_agent').is_dir():
        log('flow.image_folder_missing')
        return False
    image_path = IMAGE_DIR / 'vendor_boot-debug.img'
    if not image_path.is_file():
        log('flow.firmware_version_file_missing')
        return False
    ini_path = BLOCK_FIRMWARE_INI
    if not ini_path.is_file():
        log('flow.block_firmware_missing')
        return False
    info = inspect_vendor_boot_image()
    version = info.get('version') or ''
    model = info.get('model') or ''
    rom_region = info.get('rom_region') or ''
    platform = inspect_flash_xml_platform() or ''
    adb_state.LAST_IMAGE_MODEL = model
    adb_state.LAST_IMAGE_VERSION = version
    adb_state.LAST_IMAGE_ROM_REGION = rom_region
    adb_state.LAST_IMAGE_PLATFORM = platform
    if not version or not model:
        log('flow.firmware_version_not_found')
        return False
    if rom_region == 'PRC':
        log('flow.prc.device_row_image_prc')
    elif rom_region == 'ROW':
        log('flow.image_folder_row')
    blocked = _load_blocked_versions(ini_path)
    candidates: set[str] = set()
    section_name = ''
    if rom_region == 'PRC':
        section_name = 'PRC ROM'
    elif rom_region == 'ROW':
        section_name = 'ROW ROM'
    if section_name:
        candidates |= blocked.get(section_name.upper(), {}).get(model, set())
    if not candidates:
        for group in blocked.values():
            candidates |= group.get(model, set())
    if version.upper() in candidates:
        log('flow.firmware_version_blocked')
        return False
    log('flow.firmware_version_ok')
    return True
