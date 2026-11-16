"""LPMBox Android 14 experimental ROM route regression.

Tests intentionally inspect source only: no ADB, Fastboot, reboot or firmware writes.
"""
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
GUI = (ROOT / "crates/lpmbox-gui/src/main.rs").read_text(encoding="utf-8")
DEVICE = (ROOT / "crates/lpmbox-device/src/lib.rs").read_text(encoding="utf-8")


class Android14ExperimentScopeTests(unittest.TestCase):
    def test_original_android15_policy_for_other_flows(self):
        self.assertIn("probe_adb_device_for_convert_wipe_inner(false, on_log)", DEVICE)
        self.assertIn("major > 14 || (allow_android14_convert_trial && major == 14)", DEVICE)
        self.assertIn("probe_adb_device_for_convert_wipe_inner(true, on_log)", DEVICE)

    def test_both_rom_install_and_row_update_probe_android14_before_strict_model_checks(self):
        install = GUI.split("fn run_convert_wipe_flow(", 1)[1].split("fn run_row_update_keep_data_flow(", 1)[0]
        update = GUI.split("fn run_row_update_keep_data_flow(", 1)[1].split("fn run_country_reset_flow(", 1)[0]
        reset = GUI.split("fn run_country_reset_flow(", 1)[1].split("fn run_proinfo_backup_flow(", 1)[0] if "fn run_proinfo_backup_flow(" in GUI.split("fn run_country_reset_flow(", 1)[1] else GUI.split("fn run_country_reset_flow(", 1)[1]
        self.assertIn("probe_adb_device_for_android14_supported_rom(", install)
        self.assertIn("probe_adb_device_for_android14_supported_rom(", update)
        self.assertIn("validate_device_and_firmware_for_convert_wipe(&device, &firmware)", install)
        self.assertIn("validate_device_and_firmware_for_row_update_keep_data(&device, &firmware)", update)
        self.assertIn("probe_adb_device_for_convert_wipe(", reset)
        self.assertNotIn("probe_adb_device_for_android14_supported_rom(", reset)

    def test_model_region_platform_guard_precedes_device_writes(self):
        install = GUI.split("fn run_convert_wipe_flow(", 1)[1].split("fn run_row_update_keep_data_flow(", 1)[0]
        check = install.index("validate_device_and_firmware_for_convert_wipe(&device, &firmware)")
        backup = install.index("run_optional_country_proinfo_backup(")
        patch = install.index("rom_lk::backup_running_lk_if_required(")
        slot = install.index("lpmbox_device::run_current_slot_a_stage_v2(")
        self.assertLess(check, backup)
        self.assertLess(backup, patch)
        self.assertLess(patch, slot)
        self.assertIn('device.android_major == Some(14)', GUI)
        self.assertIn('fn android14_supported_pair_allowed(', GUI)
        self.assertIn('matches!(m.as_str(), "TB375FC" | "TB373FU")', GUI)
        self.assertIn('matches!(m.as_str(), "TB335FC" | "TB336FU")', GUI)
        self.assertIn('device_platform.eq_ignore_ascii_case("MT6897")', GUI)
        self.assertIn('image_platform.eq_ignore_ascii_case("MT6897")', GUI)
        self.assertIn('device_platform.eq_ignore_ascii_case("MT6835")', GUI)
        self.assertIn('image_platform.eq_ignore_ascii_case("MT6835")', GUI)

    def test_no_existing_flash_step_removed(self):
        self.assertEqual(GUI.count("lpmbox_spft::execute_firmware_download_streaming("), 4)
        self.assertEqual(GUI.count("lpmbox_device::run_current_slot_a_stage_v2("), 2)


if __name__ == "__main__":
    unittest.main()
