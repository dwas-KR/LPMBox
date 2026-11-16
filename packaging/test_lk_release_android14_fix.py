"""Offline regression checks for boot-reported LK release assets and Android 14 routing."""
from pathlib import Path
import hashlib
import unittest

ROOT = Path(__file__).resolve().parents[1]
LK = (ROOT / "crates/lpmbox-firmware/src/lk_patch.rs").read_text(encoding="utf-8")
POLICY = (ROOT / "crates/lpmbox-firmware/src/lk_model_policy.rs").read_text(encoding="utf-8")
GUI = (ROOT / "crates/lpmbox-gui/src/main.rs").read_text(encoding="utf-8")
DEVICE = (ROOT / "crates/lpmbox-device/src/lib.rs").read_text(encoding="utf-8")
PACKER = (ROOT / "packaging/package_windows.py").read_text(encoding="utf-8")
ASSETS = {
    "tools/TB375FC/TB375FC.TB373FU_lk_patched.img": ("f1e5b7f3a2cc9ead1c4fc2f57821e27a9ba5a49410cd7d91e1e9922087bbfa66", 3372448),
    "crates/lpmbox-firmware/assets/TB375FC.TB373FU_lk_patched.img": ("f1e5b7f3a2cc9ead1c4fc2f57821e27a9ba5a49410cd7d91e1e9922087bbfa66", 3372448),
    "tools/TB335FC/TB335FC.TB336FU_lk_patched.img": ("61e80798dd4fe8cc490543b43ccc8dd009e4dd5e5ed677eb9cf754e801c2eae8", 2909184),
}

class LkReleaseFixTests(unittest.TestCase):
    def test_exact_release_assets_are_present_and_sha_pinned(self):
        for rel, (expected, expected_size) in ASSETS.items():
            with self.subTest(asset=rel):
                data = (ROOT / rel).read_bytes()
                self.assertEqual(len(data), expected_size)
                self.assertEqual(hashlib.sha256(data).hexdigest(), expected)
                self.assertIn(rel, PACKER)

    def test_release_embedded_in_application_and_preflight(self):
        self.assertIn('include_bytes!("../assets/TB375FC.TB373FU_lk_patched.img")', LK)
        self.assertIn("verified_tb375_release_lk()?.to_vec()", LK)
        self.assertIn("validate_tb373_row_release(version)?", POLICY)
        self.assertIn("parsed[0] == 17", POLICY)
        self.assertIn("if parsed>[1,1,10,299]", POLICY)

    def test_row057_and_version_lookalikes(self):
        self.assertIn("(17, 5, 10, 57)", LK)
        self.assertIn('Some("ZUI_17.5.10.057_ST")', LK)
        self.assertIn('Some("ZUI_17.5.10.056_ST")', LK)
        self.assertIn("verify_sha256(TB375_RELEASE_BYTES", LK)

    def test_android14_for_both_rom_modes_before_device_changes(self):
        install=GUI.split("fn run_convert_wipe_flow(",1)[1].split("fn run_row_update_keep_data_flow(",1)[0]
        update=GUI.split("fn run_row_update_keep_data_flow(",1)[1].split("fn run_country_reset_flow(",1)[0]
        for src in (install,update):
            self.assertIn("probe_adb_device_for_android14_supported_rom(",src)
            self.assertIn("preflight_lk_before_device_changes(",src)
        self.assertIn('matches!(m.as_str(), "TB375FC" | "TB373FU")',GUI)
        self.assertIn('matches!(m.as_str(), "TB335FC" | "TB336FU")',GUI)
        self.assertIn("major > 14 || (allow_android14_convert_trial && major == 14)", DEVICE)
        self.assertIn("probe_adb_device_for_convert_wipe_inner(false, on_log)", DEVICE)

    def test_firmware_stage_count_unchanged(self):
        self.assertEqual(GUI.count("lpmbox_spft::execute_firmware_download_streaming("),4)
        self.assertEqual(GUI.count("lpmbox_device::run_current_slot_a_stage_v2("),2)

if __name__ == "__main__":
    unittest.main()
