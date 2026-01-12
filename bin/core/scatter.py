from __future__ import annotations

from pathlib import Path
from xml.etree import ElementTree as ET
import shutil

from .constants import IMAGE_DIR, LK_DTBO_DIR
from .utils import log
from .xml_crypto import decrypt_scatter_x


def _find_scatter_x(platform: str) -> Path | None:
    expected = IMAGE_DIR / f"{platform}_Android_scatter.x"
    if expected.is_file():
        return expected
    for p in IMAGE_DIR.glob("*_Android_scatter.x"):
        if p.is_file():
            return p
    log("scatter.not_found")
    return None


def _convert_x_to_xml(scatter_x: Path) -> Path:
    log("scatter.convert")
    xml_path = IMAGE_DIR / "Android_scatter.xml"
    decrypted = decrypt_scatter_x(scatter_x)
    xml_path.write_bytes(decrypted)
    log("scatter.convert_done")
    return xml_path


def _create_ab_scatter(xml_path: Path) -> Path:
    log("scatter.create_ab")
    tree = ET.parse(xml_path)
    root = tree.getroot()
    partitions = root.findall("partition")
    if not partitions:
        return xml_path
    ab_root = ET.Element(root.tag, root.attrib)
    for part in partitions:
        ab_root.append(part)
    ab_tree = ET.ElementTree(ab_root)
    ab_path = IMAGE_DIR / "Android_scatter_A,B.xml"
    ab_tree.write(ab_path, encoding="utf-8", xml_declaration=True)
    return ab_path


def _patch_proinfo(scatter_path: Path, final_name: str) -> Path:
    tree = ET.parse(scatter_path)
    root = tree.getroot()
    found = False
    for part in root.findall("partition"):
        name_el = part.find("name")
        if name_el is None or not name_el.text:
            continue
        name = name_el.text.strip().lower()
        if name != "proinfo":
            continue
        found = True
        file_el = part.find("file_name")
        if file_el is not None:
            file_el.text = "proinfo"
        upg_el = part.find("is_upgradable")
        if upg_el is not None:
            upg_el.text = "true"
        dl_el = part.find("is_download")
        if dl_el is not None:
            dl_el.text = "true"
    if not found:
        log("scatter.proinfo_not_found")
    final_path = IMAGE_DIR / final_name
    tree.write(final_path, encoding="utf-8", xml_declaration=True)
    return final_path


def _cleanup_temp() -> None:
    for name in ("Android_scatter.xml", "Android_scatter_A,B.xml"):
        path = IMAGE_DIR / name
        if path.is_file():
            path.unlink()
    log("scatter.temp_cleanup")


def _copy_lk_dtbo_images() -> None:
    if not LK_DTBO_DIR.is_dir():
        return
    try:
        IMAGE_DIR.mkdir(parents=True, exist_ok=True)
    except Exception:
        pass
    copied = False
    for src in LK_DTBO_DIR.iterdir():
        if not src.is_file():
            continue
        name_lower = src.name.lower()
        if ("lk" not in name_lower and "dtbo" not in name_lower) or ("_a" not in name_lower and "_b" not in name_lower):
            continue
        dst = IMAGE_DIR / src.name
        try:
            shutil.copy2(src, dst)
            copied = True
        except Exception:
            pass
    if copied:
        log("scatter.lk_dtbo_copied")


def prepare_platform_scatter(platform: str) -> Path | None:
    scatter_x = _find_scatter_x(platform)
    if scatter_x is None:
        return None
    log("scatter.found_x", name=scatter_x.name)
    xml_path = _convert_x_to_xml(scatter_x)
    ab_path = _create_ab_scatter(xml_path)
    final_name = scatter_x.name.replace(".x", ".xml")
    final_path = _patch_proinfo(ab_path, final_name)
    _cleanup_temp()
    _copy_lk_dtbo_images()
    log("scatter.final_saved", path=str(final_path))
    return final_path
