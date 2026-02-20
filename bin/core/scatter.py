from __future__ import annotations
from pathlib import Path
from xml.etree import ElementTree as ET
import shutil
from datetime import datetime
from .constants import IMAGE_DIR, LOGS_DIR
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

def _ensure_child_text(parent: ET.Element, tag: str, text: str | None = None) -> ET.Element:
    elem = parent.find(tag)
    if elem is None:
        elem = ET.SubElement(parent, tag)
    if text is not None:
        elem.text = text
    return elem



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
    ab_slots: dict[str, dict[str, ET.Element]] = {}
    for part, name in _iter_partitions(root):
        lower = name.lower()
        if lower.endswith('_a') or lower.endswith('_b'):
            base = lower[:-2]
            slot = lower[-1]
            ab_slots.setdefault(base, {})[slot] = part
    truthy = {'1', 'true', 'True', 'TRUE'}
    for base, slots in ab_slots.items():
        a = slots.get('a')
        b = slots.get('b')
        if not a or not b:
            continue
        file_a = _ensure_child_text(a, 'file_name')
        file_b = _ensure_child_text(b, 'file_name')
        name_a = (file_a.text or '').strip()
        name_b = (file_b.text or '').strip()
        file_name = name_a or name_b
        if file_name and file_name.upper() != 'NONE':
            file_a.text = file_name
            file_b.text = file_name
        dl_a = _ensure_child_text(a, 'is_download')
        dl_b = _ensure_child_text(b, 'is_download')
        up_a = _ensure_child_text(a, 'is_upgradable')
        up_b = _ensure_child_text(b, 'is_upgradable')
        dl_val_a = (dl_a.text or '').strip()
        dl_val_b = (dl_b.text or '').strip()
        up_val_a = (up_a.text or '').strip()
        up_val_b = (up_b.text or '').strip()
        dl_combined = 'true' if dl_val_a in truthy or dl_val_b in truthy else 'false'
        up_combined = 'true' if up_val_a in truthy or up_val_b in truthy else 'false'
        dl_a.text = dl_combined
        dl_b.text = dl_combined
        up_a.text = up_combined
        up_b.text = up_combined
        if base in ('boot', 'vbmeta'):
            dl_a.text = 'true'
            dl_b.text = 'true'
            up_a.text = 'true'
            up_b.text = 'true'
    if not found_proinfo:
        log('scatter.proinfo_not_found')
    _fix_ab_slots(root)
    final_path = IMAGE_DIR / final_name
    tree.write(final_path, encoding='utf-8', xml_declaration=True)
    return final_path





def _fix_ab_slots(root: ET.Element) -> None:
    parts_by_key: dict[tuple[str, str, str], ET.Element] = {}
    for part, name in _iter_partitions(root):
        storage_el = part.find('storage')
        if storage_el is None:
            continue
        storage = (storage_el.text or '').strip()
        if not (name.endswith('_a') or name.endswith('_b')):
            continue
        base = name[:-2]
        suffix = name[-1]
        if base in ('lk', 'dtbo'):
            continue
        key = (base, suffix, storage)
        parts_by_key[key] = part
    ref_info: dict[tuple[str, str], str] = {}
    for (base, suffix, storage), part in parts_by_key.items():
        file_el = part.find('file_name')
        dl_el = part.find('is_download')
        fname = (file_el.text or '').strip() if file_el is not None else ''
        dl = (dl_el.text or '').strip().lower() if dl_el is not None else ''
        if fname and fname.upper() != 'NONE' and dl != 'false':
            ref_info[(base, suffix)] = fname
    for (base, suffix), fname in ref_info.items():
        for storage in ('HW_STORAGE_EMMC', 'HW_STORAGE_UFS'):
            part = parts_by_key.get((base, suffix, storage))
            if part is None:
                continue
            file_el = part.find('file_name')
            if file_el is None:
                file_el = ET.SubElement(part, 'file_name')
            file_el.text = fname
            dl_el = part.find('is_download')
            if dl_el is None:
                dl_el = ET.SubElement(part, 'is_download')
            dl_el.text = 'true'
            up_el = part.find('is_upgradable')
            if up_el is None:
                up_el = ET.SubElement(part, 'is_upgradable')
            up_el.text = 'true'


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
    platform_scatter = IMAGE_DIR / f"{platform}_Android_scatter.xml"
    try:
        if platform_scatter.is_file():
            platform_scatter.unlink()
    except OSError:
        pass
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


def backup_platform_scatter_to_logs(platform: str) -> None:
    try:
        src = IMAGE_DIR / f"{platform}_Android_scatter.xml"
        if not src.is_file():
            return
        LOGS_DIR.mkdir(parents=True, exist_ok=True)
        timestamp = datetime.now().strftime("%Y%m%d-%H%M%S")
        dst = LOGS_DIR / f"{platform}_Android_scatter_{timestamp}.xml"
        shutil.copy2(src, dst)
    except Exception:
        return
