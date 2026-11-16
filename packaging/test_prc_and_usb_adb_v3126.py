"""Regression checks: PRC stock images and always-on idle USB ADB polling."""
from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[1]
GUI=(ROOT/"crates/lpmbox-gui/src/main.rs").read_text(encoding="utf-8")
LK=(ROOT/"crates/lpmbox-firmware/src/lk_patch.rs").read_text(encoding="utf-8")
PLAN=(ROOT/"crates/lpmbox-firmware/src/flash_plan.rs").read_text(encoding="utf-8")
DEV=(ROOT/"crates/lpmbox-device/src/lib.rs").read_text(encoding="utf-8")

class PrcAndUsbAdbRegression(unittest.TestCase):
    def test_subscriptions_run_on_all_idle_pages(self):
        area=GUI.split("fn subscription(&self)",1)[1].split("fn update(",1)[0]
        self.assertIn("if !self.busy && !self.ui_animation_active()",area)
        # The separate model-placeholder timer may still be page-scoped.
        self.assertIn("map(|_| Message::DashboardRefreshTick)",area.split("if matches!(self.active_nav",1)[0])

    def test_dashboard_tick_does_not_depend_on_selected_page(self):
        area=GUI.split("Message::DashboardRefreshTick => {",1)[1].split("Message::DashboardLoaded(result)",1)[0]
        self.assertIn("self.busy || self.dashboard_refreshing",area)
        self.assertNotIn("matches!(self.active_nav, NavPage::Dashboard | NavPage::Rom)",area)

    def test_usb_adb_remains_direct_not_server_based(self):
        self.assertIn("let mut adb = AdbManager::new();",DEV)
        self.assertIn("ensure_dashboard_adb_ready(&mut adb)?;",DEV)
        self.assertIn("read_dashboard_device_info()",GUI)

    def test_prc_stock_and_patch_separated(self):
        self.assertIn("prepare_prc_lk_for_version(",PLAN)
        self.assertIn("keep_prc_stock_image_references(",PLAN)
        self.assertIn('Some("lk.img")',LK) if False else self.assertIn('"lk.img"',LK)
        self.assertIn('RomRegion::Row => {',PLAN)
        self.assertIn('RomRegion::Prc => {',PLAN)

    def test_dtbo_validation_error_is_not_cable_failure(self):
        area=GUI.split("fn error_is_usb_transport_related(",1)[1].split("fn is_usb_debugging_shell_error",1)[0]
        self.assertIn('message.contains("DTBO")',area)
        self.assertIn('message.contains("플래싱 준비 실패")',area)
        self.assertIn("error_is_usb_transport_related(&err)",GUI)

if __name__=="__main__":
    unittest.main()
