"""Source-level regression checks for A/B active-slot verification and UI."""
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEVICE = (ROOT/"crates/lpmbox-device/src/lib.rs").read_text(encoding="utf-8")
GUI = (ROOT/"crates/lpmbox-gui/src/gbst_shell.rs").read_text(encoding="utf-8")
FLOW = (ROOT/"crates/lpmbox-gui/src/main.rs").read_text(encoding="utf-8")
LANG = ROOT/"crates/lpmbox-core/lang"

class SlotSafety(unittest.TestCase):
    def test_no_duplicate_native_fastboot_commands(self):
        self.assertNotIn("set_active_a_compat(",DEVICE)
        self.assertNotIn("set_active_a_legacy(",DEVICE)
        self.assertEqual(DEVICE.count('self.command("set_active:a")'),1)

    def test_readonly_slot_queries_before_write(self):
        code=DEVICE.split("fn run_fastboot_set_active_a_sequence<",1)[1].split("fn normalize_region",1)[0]
        self.assertLess(code.index('device.get_slot_suffix()?'),code.index('device.set_active_a()'))
        self.assertIn('if before == "_a" {',code)
        self.assertIn('device.getvar("slot-count")',code)
        self.assertIn('device.getvar("has-slot:boot")',code)
        self.assertIn('let after = fastboot_get_current_slot()?',code)

    def test_adb_next_boot_a_skips_redundant_bootctl(self):
        code=DEVICE.split("pub fn run_current_slot_a_stage_with_adb_timeout",1)[1].split("pub fn trigger_rom_install_reboot_commands",1)[0]
        self.assertIn('adb_get_slot_suffix_checked()?',code)
        self.assertIn('let next_boot_slot = adb_shell_checked("bootctl get-active-boot-slot")',code)
        self.assertIn('bootctl is-slot-bootable 0',code)
        self.assertIn('bootctl set-active-boot-slot 0',code)
        self.assertIn('if verified == Some(0)',code)
        self.assertIn('if bootctl_ok {',code)

    def test_one_retry_after_bootloader_reentry(self):
        self.assertIn('run_fastboot_set_active_a_sequence(&mut on_log, true)?;',DEVICE)
        self.assertIn('run_fastboot_set_active_a_sequence(&mut on_log, false)?;',DEVICE)
        self.assertIn('if allow_retry_after_reboot {',DEVICE)
        self.assertIn('verified.as_deref() == Some("_a")',DEVICE)
        self.assertIn('let final_slot = fastboot_get_current_slot()?',DEVICE)
        self.assertIn('if final_slot != "_a" {',DEVICE)

    def test_unknown_and_single_slot_must_not_flash(self):
        self.assertIn('slot_count.trim() == "1"', DEVICE)
        self.assertIn('has_slot_boot.trim().eq_ignore_ascii_case("no")',DEVICE)
        self.assertIn('slot_support_diagnostic(&slot_count, &has_slot_boot)?;',DEVICE)
        self.assertIn('return Err(LpmError::Fastboot(format!(',DEVICE)
        self.assertEqual(FLOW.count("lpmbox_device::run_current_slot_a_stage_v2("),2)
        self.assertEqual(FLOW.count("lpmbox_spft::execute_firmware_download_streaming("),4)

    def test_completion_button_removed(self):
        self.assertNotIn('self.catalog("lpm.exec.back_to_rom")',GUI)
        self.assertIn("fn gbst_rom_execution_panel(",GUI)

    def test_twelve_locales_show_preloader_only(self):
        import json
        for path in LANG.glob("*.json"):
            if path.stem in {"ko","en","ja","zh-TW","ru","vi","es","el","hi","ka","nl","ar"}:
                data=json.loads(path.read_text(encoding="utf-8"))
                label=data["lpm.exec.connection.preloader"]
                self.assertNotIn("MediaTek",label,path.name)
                self.assertTrue(label,path.name)

if __name__=="__main__":
    unittest.main()
