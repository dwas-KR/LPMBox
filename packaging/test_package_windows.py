# SPDX-License-Identifier: GPL-3.0-only
import tempfile
import unittest
from pathlib import Path
from package_windows import read_version, source_files

class PackagePolicies(unittest.TestCase):
    def test_current_version(self):
        root = Path(__file__).resolve().parent.parent
        self.assertEqual(read_version(root), "4.0.1")

    def test_v4_release_metadata_uses_workspace_version(self):
        root = Path(__file__).resolve().parent.parent
        gui = (root / "crates/lpmbox-gui/src/main.rs").read_text(encoding="utf-8")
        self.assertIn('concat!("v", env!("CARGO_PKG_VERSION"))', gui)
        # Release changelog was intentionally removed from this source tree.
        self.assertFalse((root / "CHANGELOG_v4.0.1.md").exists())
        packer = (root / "packaging/package_windows.py").read_text(encoding="utf-8")
        self.assertNotIn('"CHANGELOG_v4.0.1.md"', packer)
        self.assertNotIn('f"CHANGELOG_v{version}.md"', packer)
        self.assertIn('prefix = f"LPMBox-v{version}"', packer)

    def test_never_package_git_or_arbitrary_build_artifacts(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / ".git").mkdir()
            (root / ".git" / "config").write_text("secret")
            (root / "target").mkdir()
            (root / "target" / "debug.bin").write_text("binary")
            (root / "crates").mkdir()
            (root / "crates" / "file.rs").write_text("fn main() {}")
            (root / ".env").write_text("TOKEN=secret")
            (root / "list.txt").write_text("local file inventory")
            (root / "release.zip").write_bytes(b"not a source file")
            (root / "docs").mkdir()
            (root / "docs" / "internal_report.md").write_text("local analysis")
            names = [name for _, name in source_files(root)]
            self.assertEqual(names, ["crates/file.rs"])

if __name__ == "__main__":
    unittest.main()
