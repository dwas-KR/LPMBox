"""Regression tests for SPFlashToolV6 v6.2404 legacy Boost paths.

The XML schema can validate even when Boost cannot create a Unicode log
directory. These tests verify that readback uses an ASCII staging workspace,
and only successful output is copied back to the existing LPMBox backup.
Real-device USB/DA protocol is beyond source-level tests.
"""
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPFT = (ROOT / "crates/lpmbox-spft/src/lib.rs").read_text(encoding="utf-8")
GUI = (ROOT / "crates/lpmbox-gui/src/main.rs").read_text(encoding="utf-8")
WORKSPACE = (ROOT / "Cargo.toml").read_text(encoding="utf-8")


class SpftUnicodePathRegression(unittest.TestCase):
    def test_ascii_staging_and_original_output_preservation(self):
        self.assertIn("fn ascii_readback_workspace()", SPFT)
        self.assertIn("fs::create_dir_all(&log_dir)?;", SPFT)
        self.assertIn('let config_xml = workspace.join("readback_config.xml");', SPFT)
        self.assertIn('let staged_proinfo = workspace.join("backup").join("proinfo");',SPFT)
        self.assertIn("let proinfo_out = app_paths::proinfo_backup_path();",SPFT)
        self.assertIn("build_readback_config_xml(&flash_xml, &log_dir, &staged_proinfo)",SPFT)
        self.assertIn("if output.success && staged_proinfo.is_file()", SPFT)
        self.assertIn("fs::copy(&staged_proinfo, &plan.proinfo_out)",SPFT)
        self.assertIn("let proinfo_exists = plan.proinfo_out.is_file();",SPFT)

    def test_third_party_tool_does_not_receive_non_ascii_paths(self):
        self.assertIn("require_spft_ascii_path(&flash_xml,",SPFT)
        self.assertIn("require_spft_ascii_path(&spft_exe,",SPFT)
        self.assertIn("require_spft_ascii_path(&workspace,",SPFT)
        self.assertIn("path.to_string_lossy().is_ascii()",SPFT)

    def test_read_partition_command_kept_unchanged(self):
        self.assertIn('command.arg("-i").arg(&plan.config_xml);', SPFT)
        self.assertIn('<READ-PARTITION version="1.0" partition="proinfo">',SPFT)
        self.assertIn('<REBOOT version="1.0" action="COLD-RESET"/>',SPFT)
        self.assertEqual(GUI.count("lpmbox_spft::execute_firmware_download_streaming("),4)

    def test_window_and_version(self):
        self.assertIn("const WINDOW_WIDTH: f32 = 1085.0;", GUI)
        self.assertIn("const WINDOW_HEIGHT: f32 = 745.0;", GUI)
        self.assertIn('version = "4.0.1"',WORKSPACE)

if __name__ == "__main__":
    unittest.main()
