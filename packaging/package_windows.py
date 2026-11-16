#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Reproducibly package a compiled LPMBox Windows GUI and complete source tree.

Inspired by LTBox 3.3.5's separate packaging policy/tests, adapted for LPMBox.
Does not build firmware, flash a device, or publish GitHub releases.
"""
from __future__ import annotations
import argparse
import hashlib
import sys
import tomllib
import zipfile
from pathlib import Path

# Prevent build outputs and local reports from entering the source archive.
# Do not exclude Rust modules based on names such as "research" or "trial";
# those can be compiled source dependencies.
EXCLUDE_DIRS = {
    ".git", "target", "dist", "__pycache__", ".idea", ".vscode", ".venv",
    "logs", "backup", "work", "docs", "reports", "report", "analysis",
    "research", "temp", "tmp",
}
EXCLUDE_NAMES = {".env", "adbkey", "adbkey.pub", "list.txt", "readback_config.xml"}
EXCLUDE_SUFFIXES = {".pdb", ".ilk", ".obj", ".exe", ".zip", ".log", ".bak", ".tmp", ".part"}
REQUIRED_SOURCE = {"Cargo.toml", "Cargo.lock", "LICENSE", "README.md",
                   "crates/lpmbox-gui/src/main.rs",
                   "crates/lpmbox-core/src/config_store.rs",
                   "crates/lpmbox-firmware/src/lk_patch.rs",
                   "crates/lpmbox-firmware/src/lk_model_policy.rs",
                   "crates/lpmbox-gui/src/rom_lk.rs",
                   "crates/lpmbox-spft/src/active_lk.rs",
                   "tools/TB375FC/ROW_084_lk_patched.img",
                   "tools/TB375FC/TB375FC.TB373FU_lk_patched.img",
                   "tools/TB335FC/TB335FC.TB336FU_lk_patched.img",
                   "crates/lpmbox-firmware/assets/TB375FC.TB373FU_lk_patched.img",
                   "crates/lpmbox-firmware/assets/TB336FU_ROW_17_5_10_321_LK_REGION_PATCH_EXPERIMENTAL_UNVERIFIED.img"}
LK084_ASSET = "tools/TB375FC/ROW_084_lk_patched.img"
LK084_SHA256 = "f277c69c122e6f1c88e0fdf71c10ca2b07abf921da04550dec5ce98e04abca33"
RELEASE_LK_ASSETS = {
    "tools/TB375FC/TB375FC.TB373FU_lk_patched.img":
        "f1e5b7f3a2cc9ead1c4fc2f57821e27a9ba5a49410cd7d91e1e9922087bbfa66",
    "tools/TB335FC/TB335FC.TB336FU_lk_patched.img":
        "61e80798dd4fe8cc490543b43ccc8dd009e4dd5e5ed677eb9cf754e801c2eae8",
    "crates/lpmbox-firmware/assets/TB375FC.TB373FU_lk_patched.img":
        "f1e5b7f3a2cc9ead1c4fc2f57821e27a9ba5a49410cd7d91e1e9922087bbfa66",
    "crates/lpmbox-firmware/assets/TB336FU_ROW_17_5_10_321_LK_REGION_PATCH_EXPERIMENTAL_UNVERIFIED.img":
        "61e80798dd4fe8cc490543b43ccc8dd009e4dd5e5ed677eb9cf754e801c2eae8",
}


def read_version(root: Path) -> str:
    cargo = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    return str(cargo["workspace"]["package"]["version"])


def sha256(path: Path) -> str:
    with path.open("rb") as fp:
        digest = hashlib.sha256()
        while data := fp.read(1024 * 1024):
            digest.update(data)
        return digest.hexdigest()


def source_files(root: Path) -> list[tuple[Path, str]]:
    files = []
    for file in root.rglob("*"):
        rel = file.relative_to(root)
        if any(part in EXCLUDE_DIRS for part in rel.parts):
            continue
        if file.is_symlink() or not file.is_file():
            continue
        if file.name in EXCLUDE_NAMES or file.suffix.lower() in EXCLUDE_SUFFIXES:
            continue
        files.append((file, rel.as_posix()))
    return sorted(files, key=lambda item: item[1])


def create_zip(target: Path, entries: list[tuple[Path, str]], required: set[str],
               exe_sha: str, exe_name: str) -> dict:
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.exists():
        target.unlink()
    with zipfile.ZipFile(target, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=6,
                         allowZip64=True) as archive:
        for file, arcname in entries:
            archive.write(file, arcname=arcname)
    with zipfile.ZipFile(target) as archive:
        if archive.testzip() is not None:
            raise RuntimeError(f"ZIP CRC failed: {target}")
        names = archive.namelist()
        if len(set(names)) != len(names):
            raise RuntimeError(f"Duplicate ZIP entry: {target}")
        missing = required - set(names)
        if missing:
            raise RuntimeError(f"Missing ZIP entries: {sorted(missing)}")
        if hashlib.sha256(archive.read(exe_name)).hexdigest() != exe_sha:
            raise RuntimeError(f"Bundled executable differs from Release build: {target}")
    return {"path": str(target), "bytes": target.stat().st_size,
            "files": len(entries), "sha256": sha256(target)}


def package(root: Path, output: Path, release_exe: Path | None = None) -> dict:
    version = read_version(root)
    # Support a versioned binary while an older LPMBox instance is running:
    # Windows may lock target/release/lpmbox.exe against replacement.
    exe = release_exe or root / "target" / "release" / "lpmbox.exe"
    if not exe.is_file() or exe.stat().st_size < 1024 * 1024:
        raise FileNotFoundError(f"Run cargo build --release -p lpmbox-gui first: {exe}")
    exe_hash = sha256(exe)
    lk_asset = root / LK084_ASSET
    if not lk_asset.is_file() or sha256(lk_asset) != LK084_SHA256:
        raise RuntimeError("Missing or invalid verified ROW 084 LK runtime asset")
    for asset_name, expected_digest in RELEASE_LK_ASSETS.items():
        asset = root / asset_name
        if not asset.is_file() or sha256(asset) != expected_digest:
            raise RuntimeError(f"Missing or invalid SHA-pinned LK release asset: {asset_name}")
    # Reject a stale executable even when its filename is lpmbox.exe.
    # SHA-pinned assets must be embedded in this very Windows Release EXE.
    exe_bytes = exe.read_bytes()
    build_marker = f"LPMBOX_LK_RELEASE_BUILD_V{version}".encode("ascii")
    if build_marker not in exe_bytes:
        raise RuntimeError(
            f"Stale executable or wrong release version: missing {build_marker!r}"
        )
    for name in (
        "crates/lpmbox-firmware/assets/TB375FC.TB373FU_lk_patched.img",
        "crates/lpmbox-firmware/assets/TB336FU_ROW_17_5_10_321_LK_REGION_PATCH_EXPERIMENTAL_UNVERIFIED.img",
    ):
        if (root / name).read_bytes() not in exe_bytes:
            raise RuntimeError(f"Release EXE does not fully embed SHA-pinned LK: {name}")
    prefix = f"LPMBox-v{version}"
    source = source_files(root)
    missing_sources = REQUIRED_SOURCE - {name for _, name in source}
    if missing_sources:
        raise RuntimeError(f"Source tree incomplete: {sorted(missing_sources)}")

    runtime = [(exe, "lpmbox.exe")]
    for name in ["LICENSE", "README.md"]:
        if (root / name).is_file():
            runtime.append((root / name, name))
    for file in (root / "THIRD_PARTY_LICENSES").rglob("*"):
        if file.is_file() and not file.is_symlink():
            runtime.append((file, file.relative_to(root).as_posix()))
    # The flags' MIT notice is stored beside their source SVGs. Also include
    # it in Windows binary distributions where the SVG files are embedded.
    flags_license = root / "crates/lpmbox-gui/assets/flags/LICENSE"
    if not flags_license.is_file():
        raise FileNotFoundError(f"Missing flag-icons MIT license: {flags_license}")
    runtime.append((flags_license, "THIRD_PARTY_LICENSES/flag-icons-MIT-LICENSE"))
    # Retain the historical ROW084 resource for compatibility with existing
    # packaging, but the v4.0.1 TB373FU path uses its own embedded SHA-pinned LK.
    # Include the independently checked release files in both ZIP archives.
    for name in [LK084_ASSET, *RELEASE_LK_ASSETS.keys(),
                 "third_party/TB375FC-LK-patch-LICENSE.txt"]:
        file = root / name
        if file.is_file():
            runtime.append((file, name))

    windows_name = output / f"{prefix}-win-x86-x64.zip"
    project_name = output / f"{prefix}-full-project.zip"
    results = [
        create_zip(
            windows_name, runtime,
            {"lpmbox.exe", "LICENSE", "README.md",
             "THIRD_PARTY_LICENSES/flag-icons-MIT-LICENSE",
             LK084_ASSET, *RELEASE_LK_ASSETS.keys()},
            exe_hash, "lpmbox.exe"
        ),
        create_zip(project_name, source + [(exe, "target/release/lpmbox.exe")],
                   REQUIRED_SOURCE | {"target/release/lpmbox.exe"},
                   exe_hash, "target/release/lpmbox.exe"),
    ]
    sums = output / f"SHA256SUMS-LPMBox-v{version}.txt"
    sums.write_text(
        "".join(f"{entry['sha256']}  {Path(entry['path']).name}\n" for entry in results)
        + f"{exe_hash}  lpmbox.exe\n", encoding="utf-8")
    return {"version": version, "files": results, "checksums": str(sums),
            "exe_sha256": exe_hash}


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--project", type=Path,
                        default=Path(__file__).resolve().parent.parent)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--release-exe", type=Path,
                        help="Compiled EXE override, useful if the previous EXE is still running")
    args = parser.parse_args()
    root = args.project.resolve()
    dest = (args.output or root.parent).resolve()
    import json
    print(json.dumps(package(root, dest, args.release_exe), ensure_ascii=False, indent=2))
