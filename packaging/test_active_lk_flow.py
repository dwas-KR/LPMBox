# SPDX-License-Identifier: GPL-3.0-only
"""Regression checks for TB375FC/ROW 084+ active-slot LK readback."""
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parent.parent
GUI = (ROOT/"crates/lpmbox-gui/src/main.rs").read_text(encoding="utf-8")
SLOT = (ROOT/"crates/lpmbox-device/src/lib.rs").read_text(encoding="utf-8")
SPFT = (ROOT/"crates/lpmbox-spft/src/lib.rs").read_text(encoding="utf-8")
SPFT_LK = (ROOT/"crates/lpmbox-spft/src/active_lk.rs").read_text(encoding="utf-8")
LK_FLOW = (ROOT/"crates/lpmbox-gui/src/rom_lk.rs").read_text(encoding="utf-8")
FIRMWARE = (ROOT/"crates/lpmbox-firmware/src/flash_plan.rs").read_text(encoding="utf-8")

class ActiveSlotLKRegression(unittest.TestCase):
    def test_both_original_rom_install_flows_call_readback_once(self):
        self.assertEqual(GUI.count("rom_lk::backup_running_lk_if_required("), 2)
        self.assertEqual(GUI.count("lpmbox_spft::execute_firmware_download_streaming("), 4)
        self.assertEqual(GUI.count("run_optional_country_proinfo_backup(&image_dir, country_code, &tx)"), 2)
        self.assertEqual(GUI.count("lpmbox_device::run_current_slot_a_stage_v2(|message|"), 2)

    def test_only_readonly_lk_a_or_lk_b_no_metadata_write(self):
        self.assertIn('"lk_a" => Ok("lk_a")', SPFT_LK) if False else None
        self.assertIn('"_a" | "a" | "lk_a" => Ok("lk_a")', SPFT_LK)
        self.assertIn('"_b" | "b" | "lk_b" => Ok("lk_b")', SPFT_LK)
        self.assertIn('build_readback_config_xml(', SPFT_LK)
        self.assertNotIn('write_partition(', SPFT_LK)
        self.assertIn('getprop ro.boot.slot_suffix', SLOT)
        self.assertNotIn('setprop ro.boot.slot_suffix', LK_FLOW)

    def test_same_proinfo_worker_and_safe_flash_plan_path(self):
        self.assertIn('PrearmedProinfoReadback::start_from_plan(plan)', SPFT_LK)
        self.assertIn('pub fn prepare_local_lk_dtbo_from_readback', (ROOT/"crates/lpmbox-firmware/src/lk_patch.rs").read_text(encoding="utf-8"))
        self.assertIn('prepare_flash_plan_with_active_lk_log', FIRMWARE)
        self.assertIn('read_running_slot_for_lk_backup', LK_FLOW)

if __name__ == "__main__":
    unittest.main()
