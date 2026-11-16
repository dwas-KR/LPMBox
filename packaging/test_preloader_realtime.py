import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEVICE = (ROOT / "crates/lpmbox-device/src/lib.rs").read_text(encoding="utf-8")
PORTS = (ROOT / "crates/lpmbox-device/src/preloader_windows.rs").read_text(encoding="utf-8")
MAIN = (ROOT / "crates/lpmbox-gui/src/main.rs").read_text(encoding="utf-8")

class PreloaderFastWatchTests(unittest.TestCase):
    def test_six_reboot_flows_arm_monitor_first(self):
        self.assertEqual(MAIN.count("PreloaderWatch::start(30)"),6)
        self.assertEqual(MAIN.count("let detect = preloader_watch.wait();"),6)
        needle = "lpmbox_device::trigger_rom_install_reboot_commands("
        for m in re.finditer(re.escape(needle),MAIN):
            before=MAIN[max(0,m.start()-750):m.start()]
            self.assertIn("let preloader_watch = lpmbox_device::PreloaderWatch::start(30)", before)
            self.assertIn("if !preloader_watch.is_armed()", before)
            self.assertIn("안전을 위해 재부팅을 취소", before)

    def test_native_ports_only_and_no_slow_subprocess_polling(self):
        self.assertIn("SetupDiGetClassDevsW", PORTS)
        self.assertIn("SetupDiEnumDeviceInfo", PORTS)
        self.assertIn("DIGCF_PRESENT", PORTS)
        self.assertIn("SPDRP_HARDWAREID", PORTS)
        self.assertIn("PortName", PORTS)
        self.assertIn("PID_2000", PORTS)
        self.assertIn("PRELOADER_POLL_INTERVAL_MS: u64 = 75", DEVICE)
        self.assertNotIn("run_and_find_preloader", DEVICE)
        self.assertNotIn("pnputil /enum-devices /connected", DEVICE)

    def test_driver_is_detected_by_name_not_version(self):
        self.assertIn("driver.DisplayName -match", DEVICE)
        drivercheck=DEVICE.split("fn check_mtk_driver_installed_impl() -> Result<bool>",2)[-1].split(
            "pub fn prepare_mtk_driver_package",1)[0]
        self.assertNotIn("DisplayVersion",drivercheck)
        self.assertNotIn("5.1632",drivercheck)

    def test_watcher_is_armed_after_first_actual_port_scan(self):
        ready = DEVICE.index("let _ = ready_tx.send(());")
        sample = DEVICE.rfind("let baseline = detect_preloader_once();",0,ready)
        self.assertTrue(sample >= 0)
        self.assertLess(sample,ready)

    def test_brom_and_preloader_are_distinct_modes(self):
        self.assertIn("BROM_HARDWARE_ID",PORTS)
        self.assertIn("scan_present_mtk_ports()",DEVICE)
        self.assertIn("detected: false",DEVICE.split("let present = preloader_windows::scan_present_mtk_ports();",1)[1][:1000])
        self.assertIn("SPFT V6 PreLoader(PID_2000) 아님",DEVICE)
        self.assertIn("let mut last_observed_mode = None;",DEVICE)

    def test_fastboot_fallback_only_after_adb_reboot_failure(self):
        reboot = DEVICE.split("pub fn trigger_rom_install_reboot_commands",1)[1].split(
            "pub fn disable_ota_updates",1)[0]
        self.assertIn("if adb_reboot_ok {",reboot)
        self.assertLess(reboot.index("if adb_reboot_ok {"),reboot.index("fastboot_reboot().is_ok()"))

    def test_failure_logs_include_port_diagnostics_and_safe_guidance(self):
        self.assertEqual(MAIN.count("[Port] PreLoader 검사 상세:"),6)
        self.assertEqual(MAIN.count("플래싱은 시작되지 않았습니다."),6)
        self.assertIn("detect.method.as_deref()",MAIN)

    def test_live_rear_usb_preloader_capture_is_recorded(self):
        self.assertIn("감시 시작 후 {elapsed}ms에 PreLoader 인식", DEVICE)
        self.assertIn("Duration::from_millis(1600)", DEVICE)
        self.assertEqual(MAIN.count("[Port] 실기기 PreLoader 감지:"),6)
        self.assertIn('if label.to_ascii_uppercase().contains(&format!("({port})"))', PORTS)

    def test_no_new_flash_spft_worker_steps(self):
        self.assertEqual(MAIN.count("lpmbox_spft::execute_firmware_download_streaming("),4)
        # The two readback calls are now pre-armed before reboot, without
        # changing the four actual ROM/SPFT firmware download call sites.
        self.assertEqual(MAIN.count("lpmbox_spft::PrearmedProinfoReadback::start("),2)
        self.assertEqual(MAIN.count("lpmbox_spft::execute_proinfo_readback_streaming("),0)

if __name__=="__main__":
    unittest.main()
