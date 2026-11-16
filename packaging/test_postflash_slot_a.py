"""Regression checks: no post-flash ADB/Fastboot slot operations."""
import unittest
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
GUI = (ROOT / "crates/lpmbox-gui/src/main.rs").read_text(encoding="utf-8")
DEV = (ROOT / "crates/lpmbox-device/src/lib.rs").read_text(encoding="utf-8")
SPFT = (ROOT / "crates/lpmbox-spft/src/lib.rs").read_text(encoding="utf-8")
def flow(name, next_name):
    return GUI.split("fn "+name,1)[1].split("fn "+next_name,1)[0]
class PostFlashRemoval(unittest.TestCase):
    def test_no_postflash_usb_wait_reboot_or_slot_change(self):
        for content in (DEV,GUI):
            for token in ("verify_slot_a_after_firmware_flash", "read_adb_serial_for_postflash",
                          "report_optional_postflash_slot_verification", "PostFlashSlotAOutcome"):
                self.assertNotIn(token,content)
        self.assertNotIn("사후 슬롯 확인은 [PostFlash]",GUI)
    def test_first_two_flash_flows_finish_after_spft(self):
        for name,stop in [
            ("run_convert_wipe_flow(", "run_row_update_keep_data_flow("),
            ("run_row_update_keep_data_flow(", "run_reinstall_wipe_flow(")]:
            with self.subTest(flow=name):
                code=flow(name,stop)
                self.assertEqual(code.count("execute_firmware_download_streaming("),1)
                self.assertIn("Ok(()) => {",code)
                self.assertTrue(
                    "[완료]" in code
                    or "PRC_ROW_INSTALL_COMPLETE_LOG.to_string()" in code
                    or "ROW_KEEP_DATA_COMPLETE_LOG.to_string()" in code,
                    "After a successful SPFT result, the worker must send a terminal completion event",
                )
                self.assertNotIn("[PostFlash]",code)
    def test_original_safety_and_spft_unchanged(self):
        self.assertEqual(GUI.count("execute_firmware_download_streaming("),4)
        self.assertEqual(GUI.count("lpmbox_device::run_current_slot_a_stage_v2("),2)
        self.assertIn('command.arg("-c").arg("download");',SPFT)
        self.assertIn("run_spflashtoolv6_streaming(command, &mut on_event, None, None, None)?",SPFT)
        self.assertIn('if final_slot != "_a" {',DEV)
        self.assertIn("run_fastboot_set_active_a_sequence(&mut on_log, false)?;",DEV)
if __name__ == "__main__":
    unittest.main()
