from __future__ import annotations
from pathlib import Path
from xml.etree import ElementTree as ET
import shutil
from .constants import IMAGE_DIR
from .utils import log
from .xml_crypto import decrypt_scatter_x

def _find_scatter_x(platform: str) -> Path:
    if not IMAGE_DIR.is_dir():
        raise FileNotFoundError('image directory not found')
    expected = IMAGE_DIR / f'{platform}_Android_scatter.x'
    if expected.is_file():
        return expected
    candidates = sorted(IMAGE_DIR.glob('*_Android_scatter.x'))
    if not candidates:
        raise FileNotFoundError('no scatter .x file')
    return candidates[0]
 
def _convert_x_to_xml(scatter_x: Path) -> Path:
    xml_path = IMAGE_DIR / 'Android_scatter.xml'
    log('scatter.convert', path=str(scatter_x))
    try:
        raw = scatter_x.read_bytes()
        xml_text: str | None = None
        try:
            text = raw.decode('utf-8')
            if '<scatter' in text or '<partition_index' in text or '<partition' in text:
                xml_text = text
        except UnicodeDecodeError:
            xml_text = None
        if xml_text is None:
            decrypted = decrypt_scatter_x(scatter_x)
            xml_text = decrypted.decode('utf-8', errors='ignore')
        xml_path.write_text(xml_text, encoding='utf-8')
    except Exception:
        log('scatter.convert_failed')
        raise
    log('scatter.convert_done', path=str(xml_path))
    return xml_path

def _create_ab_scatter(xml_path: Path) -> Path:
    ab_path = IMAGE_DIR / 'Android_scatter_A,B.xml'
    shutil.copy2(xml_path, ab_path)
    return ab_path

def _iter_partitions(root: ET.Element):
    for tag in ('partition', 'partition_index'):
        for part in root.findall(f'.//{tag}'):
            name_elem = part.find('partition_name')
            if name_elem is None:
                continue
            name = (name_elem.text or '').strip()
            if not name:
                continue
            yield (part, name)

def _ensure_child_text(parent: ET.Element, tag: str, text: str) -> None:
    elem = parent.find(tag)
    if elem is None:
        elem = ET.SubElement(parent, tag)
    elem.text = text

def _cleanup_temp_scatter(xml_path: Path, ab_path: Path) -> None:
    for path in (xml_path, ab_path):
        try:
            if path.is_file():
                path.unlink()
        except Exception:
            pass

def _patch_proinfo(ab_scatter: Path, final_name: str, keep_user_data: bool) -> Path:
    tree = ET.parse(ab_scatter)
    root = tree.getroot()
    found_proinfo = False
    found_userdata = False
    for part, name in _iter_partitions(root):
        lower = name.lower()
        if lower == 'proinfo':
            _ensure_child_text(part, 'file_name', 'proinfo')
            _ensure_child_text(part, 'is_download', 'true')
            _ensure_child_text(part, 'is_upgradable', 'true')
            found_proinfo = True
        if keep_user_data and lower == 'userdata':
            _ensure_child_text(part, 'file_name', 'userdata.img')
            _ensure_child_text(part, 'is_download', 'false')
            _ensure_child_text(part, 'is_upgradable', 'false')
            found_userdata = True
    if not found_proinfo:
        log('scatter.proinfo_not_found')
    final_path = IMAGE_DIR / final_name
    tree.write(final_path, encoding='utf-8', xml_declaration=True)
    return final_path


def apply_country_plan_to_proinfo(platform: str, enable: bool) -> None:
    
    scatter_xml = IMAGE_DIR / f"{platform}_Android_scatter.xml"
    if not scatter_xml.is_file():
        return
    try:
        tree = ET.parse(scatter_xml)
    except ET.ParseError:
        return
    root = tree.getroot()
    updated = False
    for part, name in _iter_partitions(root):
        if name.lower() == "proinfo":
            if enable:
                _ensure_child_text(part, "file_name", "proinfo")
                _ensure_child_text(part, "is_download", "true")
                _ensure_child_text(part, "is_upgradable", "true")
            else:
                _ensure_child_text(part, "file_name", "NONE")
                _ensure_child_text(part, "is_download", "false")
                _ensure_child_text(part, "is_upgradable", "false")
            updated = True
    if not updated:
        log("scatter.proinfo_not_found")
        return
    tree.write(scatter_xml, encoding="utf-8", xml_declaration=True)


def prepare_platform_scatter(platform: str, keep_user_data: bool) -> Path | None:
    try:
        scatter_x = _find_scatter_x(platform)
    except FileNotFoundError:
        log('scatter.not_found')
        return None
    xml_path = _convert_x_to_xml(scatter_x)
    ab_path = _create_ab_scatter(xml_path)
    final_name = scatter_x.name.replace('.x', '.xml')
    final_path = _patch_proinfo(ab_path, final_name, keep_user_data)
    _cleanup_temp_scatter(xml_path, ab_path)
    return final_path

def disable_lk_dtbo_partitions(platform: str) -> None:
    for name in ('lk.img', 'dtbo.img', 'lk_a', 'lk_b', 'dtbo_a', 'dtbo_b'):
        path = IMAGE_DIR / name
        try:
            if path.exists():
                path.unlink()
        except OSError:
            continue
    scatter_xml = IMAGE_DIR / f'{platform}_Android_scatter.xml'
    if not scatter_xml.is_file():
        return
    try:
        tree = ET.parse(scatter_xml)
    except ET.ParseError:
        return
    root = tree.getroot()
    updated = False
    for part, name in _iter_partitions(root):
        low = name.lower()
        if low in {'lk', 'lk_a', 'lk_b', 'dtbo', 'dtbo_a', 'dtbo_b'}:
            _ensure_child_text(part, 'is_download', 'false')
            _ensure_child_text(part, 'is_upgradable', 'false')
            updated = True
    if updated:
        tree.write(scatter_xml, encoding='utf-8', xml_declaration=True)
        log('scatter.lk_dtbo_disabled', path=str(scatter_xml))
