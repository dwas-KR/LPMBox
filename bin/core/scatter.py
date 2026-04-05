from __future__ import annotations
from pathlib import Path
from xml.etree import ElementTree as ET
import shutil
import re
from datetime import datetime
from .constants import IMAGE_DIR, LOGS_DIR, LKDTBO_MODEL_TO_ZIP
from .adb_utils import adb_shell_getprop
from . import adb_utils as adb_state
from .utils import log
from .xml_crypto import decrypt_scatter_x
from .firmware_guard import inspect_vendor_boot_image

_SCATTER_XML_RE = re.compile(r'^MT\d+_Android_scatter\.xml$', re.IGNORECASE)
_SCATTER_X_RE = re.compile(r'^MT\d+_Android_scatter\.x$', re.IGNORECASE)

PRC_TRUE_PARTITIONS = {
    'preloader_a', 'preloader_b', 'vbmeta_a', 'vbmeta_system_a', 'vbmeta_vendor_a',
    'spmfw_a', 'audio_dsp_a', 'pi_img_a', 'dpm_a', 'scp_a', 'ccu_a', 'vcp_a', 'sspm_a',
    'mcupm_a', 'gpueb_a', 'apusys_a', 'mvpu_algo_a', 'gz_a', 'lk_a', 'boot_a',
    'vendor_boot_a', 'init_boot_a', 'dtbo_a', 'tee_a', 'connsys_bt_a', 'connsys_wifi_a',
    'connsys_gnss_a', 'logo_a', 'lenovocust', 'lenovoraw', 'super', 'userdata',
}


def _iter_scatter_named_files(pattern: re.Pattern[str], base_dir: Path | None = None) -> list[Path]:
    directory = base_dir or IMAGE_DIR
    if not directory.is_dir():
        return []
    return [
        path
        for path in sorted(directory.iterdir())
        if path.is_file() and pattern.fullmatch(path.name)
    ]


def _current_image_rom_region() -> str:
    value = (getattr(adb_state, 'LAST_IMAGE_ROM_REGION', '') or '').strip().upper()
    if value:
        return value
    try:
        info = inspect_vendor_boot_image()
        region = (info.get('rom_region') or '').strip().upper()
        if region:
            adb_state.LAST_IMAGE_ROM_REGION = region
        return region
    except Exception:
        return ''


def _is_prc_image_context() -> bool:
    return _current_image_rom_region() == 'PRC'


def _is_prc_context_any() -> bool:
    image_region = _current_image_rom_region()
    device_region = (getattr(adb_state, 'LAST_DEVICE_ROM_REGION', '') or '').strip().upper()
    return image_region == 'PRC' or device_region == 'PRC'


def _find_scatter_source(platform: str) -> Path:
    if not IMAGE_DIR.is_dir():
        raise FileNotFoundError('image directory not found')
    expected_xml = IMAGE_DIR / f'{platform}_Android_scatter.xml'
    if expected_xml.is_file():
        return expected_xml
    xml_candidates = _iter_scatter_named_files(_SCATTER_XML_RE, IMAGE_DIR)
    if xml_candidates:
        return xml_candidates[0]
    expected_x = IMAGE_DIR / f'{platform}_Android_scatter.x'
    if expected_x.is_file():
        return expected_x
    x_candidates = _iter_scatter_named_files(_SCATTER_X_RE, IMAGE_DIR)
    if x_candidates:
        return x_candidates[0]
    raise FileNotFoundError('no scatter source file')


def _scatter_text_to_xml(text: str) -> str:
    root = ET.Element('scatter')
    current: ET.Element | None = None
    for raw_line in text.splitlines():
        line = raw_line.replace('\ufeff', '').strip()
        if not line or line.startswith('#'):
            continue
        if line.endswith(':'):
            probe = line[:-1].strip()
            if probe in {'info', 'config', 'layout_check'}:
                continue
        stripped = line
        is_dash = stripped.startswith('- ')
        if is_dash:
            stripped = stripped[2:].strip()
        if ':' not in stripped:
            continue
        key, value = stripped.split(':', 1)
        key = key.strip()
        value = value.strip()
        if key == 'general':
            current = ET.SubElement(root, 'general')
            if value:
                ET.SubElement(current, 'section_name').text = value
            continue
        if key in {'partition_index', 'partition'} and is_dash:
            current = ET.SubElement(root, 'partition')
            ET.SubElement(current, 'partition_index').text = value
            continue
        if current is None:
            current = ET.SubElement(root, 'general')
        ET.SubElement(current, key).text = value
    return ET.tostring(root, encoding='unicode')


def _convert_source_to_xml(scatter_source: Path, platform: str) -> Path:
    xml_path = IMAGE_DIR / f'{platform}_Android_scatter.xml'
    if scatter_source.is_file():
        try:
            if scatter_source.resolve() == xml_path.resolve():
                return xml_path
        except Exception:
            pass
    log('scatter.convert', path=str(scatter_source))
    try:
        xml_text: str | None = None
        suffix = scatter_source.suffix.lower()
        if suffix == '.xml':
            xml_text = scatter_source.read_text(encoding='utf-8', errors='ignore')
        elif suffix == '.x':
            decrypted = decrypt_scatter_x(scatter_source)
            xml_text = decrypted.decode('utf-8', errors='ignore')
        else:
            raise ValueError('unsupported scatter source')
        xml_path.write_text(xml_text or '', encoding='utf-8')
    except Exception:
        log('scatter.convert_failed')
        raise
    log('scatter.convert_done', path=str(xml_path))
    return xml_path

def _create_ab_scatter(xml_path: Path) -> Path:
    log('scatter.create_ab')
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


def _get_text(parent: ET.Element, tag: str) -> str:
    elem = parent.find(tag)
    if elem is None or elem.text is None:
        return ''
    return elem.text.strip()

def _set_text(parent: ET.Element, tag: str, text: str) -> ET.Element:
    return _ensure_child_text(parent, tag, text)



def _cleanup_temp_scatter(xml_path: Path, ab_path: Path, preserve: tuple[Path, ...] = ()) -> None:
    preserve_set = set()
    for item in preserve:
        try:
            preserve_set.add(item.resolve())
        except Exception:
            pass
    for path in (xml_path, ab_path):
        try:
            if path.resolve() in preserve_set:
                continue
        except Exception:
            pass
        try:
            if path.is_file():
                path.unlink()
        except Exception:
            pass

def _disable_none_file_partitions(root: ET.Element) -> None:
    for part, _name in _iter_partitions(root):
        file_name = _get_text(part, 'file_name')
        if file_name.upper() == 'NONE':
            _ensure_child_text(part, 'is_download', 'false')
            _ensure_child_text(part, 'is_upgradable', 'false')


def _apply_prc_download_profile(root: ET.Element) -> None:
    for part, name in _iter_partitions(root):
        lower = name.lower()
        is_enabled = lower in PRC_TRUE_PARTITIONS
        file_name = _get_text(part, 'file_name')
        if file_name.upper() == 'NONE':
            is_enabled = False
        _ensure_child_text(part, 'is_download', 'true' if is_enabled else 'false')
    _disable_none_file_partitions(root)


def _apply_prc_download_profile_file(scatter_path: Path) -> None:
    if not scatter_path.is_file():
        return
    try:
        tree = ET.parse(scatter_path)
    except ET.ParseError:
        return
    root = tree.getroot()
    _apply_prc_download_profile(root)
    tree.write(scatter_path, encoding='utf-8', xml_declaration=True)


def _resolve_lkdtbo_model(raw_model: str) -> str | None:
    for key in LKDTBO_MODEL_TO_ZIP.keys():
        if key in raw_model:
            return key
    return None


def _should_enable_lkdtbo_for_model(raw_model: str) -> bool:
    model = _resolve_lkdtbo_model(raw_model)
    return model in {'TB375FC', 'TB373FU'}


def _apply_model_lkdtbo_partitions(root: ET.Element, raw_model: str, prc_context: bool) -> bool:
    enable = _should_enable_lkdtbo_for_model(raw_model)
    updated = False
    for part, name in _iter_partitions(root):
        low = name.lower()
        if low in {'lk_a', 'lk_b', 'dtbo_a', 'dtbo_b'}:
            _set_text(part, 'file_name', name)
            if prc_context:
                slot_enable = enable and low in {'lk_a', 'dtbo_a'}
            else:
                slot_enable = enable
            value = 'true' if slot_enable else 'false'
            _set_text(part, 'is_download', value)
            _set_text(part, 'is_upgradable', value)
            updated = True
    return updated

def ensure_prc_platform_scatter(platform: str, preserve_userdata_false: bool = False) -> None:
    if not _is_prc_context_any():
        return
    scatter_path = IMAGE_DIR / f'{platform}_Android_scatter.xml'
    if not scatter_path.is_file():
        return
    try:
        tree = ET.parse(scatter_path)
    except ET.ParseError:
        return
    root = tree.getroot()
    preserve_proinfo = None
    preserve_userdata = None
    raw_model = getattr(adb_state, 'LAST_DEVICE_MODEL', '') or ''
    if not raw_model:
        try:
            raw_model = adb_shell_getprop('ro.product.model').strip()
        except Exception:
            raw_model = ''
    for part, name in _iter_partitions(root):
        lower = name.lower()
        if lower == 'proinfo':
            preserve_proinfo = (_get_text(part, 'is_download') or '').lower()
        elif lower == 'userdata':
            preserve_userdata = (_get_text(part, 'is_download') or '').lower()
    _apply_prc_download_profile(root)
    _apply_model_lkdtbo_partitions(root, raw_model, True)
    _disable_none_file_partitions(root)
    for part, name in _iter_partitions(root):
        lower = name.lower()
        if lower == 'proinfo' and preserve_proinfo == 'true':
            _ensure_child_text(part, 'is_download', 'true')
        elif lower == 'userdata' and preserve_userdata_false and preserve_userdata == 'false':
            _ensure_child_text(part, 'is_download', 'false')
    tree.write(scatter_path, encoding='utf-8', xml_declaration=True)


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
    if _is_prc_context_any():
        _apply_prc_download_profile(root)
    _disable_none_file_partitions(root)
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
        scatter_source = _find_scatter_source(platform)
    except FileNotFoundError:
        log('scatter.not_found')
        return None
    platform_scatter = IMAGE_DIR / f"{platform}_Android_scatter.xml"
    preserve_existing = _is_prc_context_any() and platform_scatter.is_file()
    if preserve_existing:
        xml_path = platform_scatter
    else:
        try:
            if platform_scatter.is_file() and scatter_source.resolve() != platform_scatter.resolve():
                platform_scatter.unlink()
        except OSError:
            pass
        xml_path = _convert_source_to_xml(scatter_source, platform)
    ab_path = _create_ab_scatter(xml_path)
    final_path = _patch_proinfo(ab_path, f'{platform}_Android_scatter.xml', keep_user_data)
    hw = (getattr(adb_state, 'LAST_DEVICE_MODEL', '') or platform).strip() or platform
    log('scatter.final_saved', hw=hw)
    _cleanup_temp_scatter(xml_path, ab_path, preserve=(final_path,))
    return final_path

def disable_lk_dtbo_partitions(platform: str) -> None:
    for name in ('lk.img', 'dtbo.img'):
        path = IMAGE_DIR / name
        if path.exists():
            try:
                path.unlink()
            except OSError:
                pass
    scatter_xml = IMAGE_DIR / f'{platform}_Android_scatter.xml'
    if not scatter_xml.is_file():
        return
    raw_model = getattr(adb_state, 'LAST_DEVICE_MODEL', '') or ''
    if not raw_model:
        try:
            raw_model = adb_shell_getprop('ro.product.model').strip()
        except Exception:
            raw_model = ''
    enable = _should_enable_lkdtbo_for_model(raw_model)
    prc_context = _is_prc_context_any()
    tree = ET.parse(scatter_xml)
    root = tree.getroot()
    updated = _apply_model_lkdtbo_partitions(root, raw_model, prc_context)
    if updated:
        tree.write(scatter_xml, encoding='utf-8', xml_declaration=True)
        log('scatter.lk_dtbo_enabled' if enable else 'scatter.lk_dtbo_disabled', path=str(scatter_xml))


def backup_platform_scatter_to_logs(platform: str) -> None:
    try:
        src = IMAGE_DIR / f"{platform}_Android_scatter.xml"
        if not src.is_file():
            return
        if _is_prc_context_any():
            return
        LOGS_DIR.mkdir(parents=True, exist_ok=True)
        timestamp = datetime.now().strftime("%Y%m%d-%H%M%S")
        dst = LOGS_DIR / f"{platform}_Android_scatter_{timestamp}.xml"
        shutil.copy2(src, dst)
    except Exception:
        return

def prepare_country_reset_scatter(platform: str) -> Path | None:
    scatter_path = prepare_platform_scatter(platform, keep_user_data=False)
    if scatter_path is None or not scatter_path.is_file():
        return None
    try:
        tree = ET.parse(scatter_path)
    except ET.ParseError:
        return None
    root = tree.getroot()
    found = False
    for part, name in _iter_partitions(root):
        low = name.lower()
        for elem in part.iter():
            if elem is part:
                continue
            if (elem.text or '').strip().lower() == 'true':
                elem.text = 'false'
        if low == 'proinfo':
            _ensure_child_text(part, 'file_name', 'proinfo')
            _ensure_child_text(part, 'is_download', 'true')
            found = True
    if not found:
        log('scatter.proinfo_not_found')
        return None
    tree.write(scatter_path, encoding='utf-8', xml_declaration=True)
    return scatter_path
