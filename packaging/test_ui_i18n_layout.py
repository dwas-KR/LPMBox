"""Static regressions for screenshot-reported layout and translation bugs."""
import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MAIN = (ROOT / "crates/lpmbox-gui/src/main.rs").read_text(encoding="utf-8")
SHELL = (ROOT / "crates/lpmbox-gui/src/gbst_shell.rs").read_text(encoding="utf-8")
LANGS = ("ko", "en", "ru", "ja", "zh-TW", "vi", "el", "hi", "ka", "nl", "ar", "es")


class MultilingualLayoutRegressionTests(unittest.TestCase):
    def test_version_prefix_only_once_in_all_three_ui_locations(self):
        self.assertIn('const APP_DISPLAY_VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));', MAIN)
        self.assertEqual(SHELL.count('format!("{APP_DISPLAY_VERSION}")'), 3)
        self.assertNotIn('format!("v{APP_DISPLAY_VERSION}")', SHELL)

    def test_active_sidebar_keeps_intentionally_hidden_tools_hidden(self):
        menu = SHELL.split("let list = column![", 1)[1].split("].spacing(1)", 1)[0]
        names = [
            'NavPage::Rom, "lpm.nav.rom"',
            'NavPage::Driver, "lpm.nav.driver"',
            'NavPage::Qna, "lpm.nav.qna"',
        ]
        positions = [menu.index(item) for item in names]
        self.assertEqual(positions, sorted(positions))
        for hidden in ("NavPage::Root,", "NavPage::Unroot,", "NavPage::FirmwareDownload,"):
            self.assertNotIn("self.gbst_nav_item(" + hidden, menu)

    def test_compact_sidebar_does_not_overlay_main_content(self):
        self.assertIn('row![self.gbst_sidebar(), main]', MAIN)
        self.assertIn('lpm_advanced_two_columns(self.window_size.0)', MAIN)
        self.assertNotIn('Stack::with_children(vec![base.into(),self.gbst_sidebar()])', MAIN)

    def test_country_confirm_and_rom_options_have_room_to_wrap(self):
        self.assertIn('self.gbst_modal_action(self.catalog("lpm.country.confirm")', MAIN)
        self.assertIn('.height(Length::Shrink)\n    .padding([24.0, 24.0])', MAIN)
        self.assertIn('let card_height = lpm_rom_card_height(active_language_option());', MAIN)
        self.assertIn('.wrapping(iced::widget::text::Wrapping::WordOrGlyph)', SHELL)

    def test_narrow_firmware_summary_stacks_and_scrolls(self):
        self.assertIn("lpm_firmware_summary_two_columns(self.window_size.0, self.gbst_sidebar_width())", MAIN)
        self.assertIn("ROM_FOLDER_LEFT_CARD_WIDTH: f32 = 378.0", MAIN)
        self.assertIn("container(iced::widget::scrollable(rom_panel)", MAIN)
        self.assertIn(".width(Length::Fill).max_width(560.0)", MAIN)

    def test_sidebar_icons_match_ltbox_pictograms(self):
        for name in ("root", "unroot", "info"):
            resource = ROOT / f"crates/lpmbox-gui/assets/icons/nav_ltbox_{name}.png"
            self.assertTrue(resource.is_file(), str(resource))
            with resource.open("rb") as stream:
                header = stream.read(24)
            self.assertEqual(header[:8], b"\x89PNG\r\n\x1a\n")
            self.assertEqual(int.from_bytes(header[16:20], "big"), 96)
            self.assertEqual(int.from_bytes(header[20:24], "big"), 96)
        self.assertIn("NavPage::Root => self.nav_root_handle.clone()", MAIN)
        self.assertIn("NavPage::Unroot => self.nav_unroot_handle.clone()", MAIN)
        self.assertIn("NavPage::About => self.nav_about_handle.clone()", MAIN)

    def test_slide_arrow_rail_fills_localized_card_height(self):
        part = MAIN.split("let slide_handle_area =", 1)[1].split("let slide_overlay_content =", 1)[0]
        self.assertIn(".height(Length::Fixed(card_height))", part)
        self.assertNotIn(".height(Length::Fixed(ROM_ROUTINE_HANDLE_HEIGHT))", part)

    def test_fixed_gui_literals_use_json_translations(self):
        fixed_dir = ROOT / "crates/lpmbox-core/lang"
        fixed = {}
        for code in LANGS:
            with (fixed_dir / f"fixed-{code}.json").open(encoding="utf-8") as f:
                fixed[code] = json.load(f)
        base = set(fixed["ko"])
        self.assertGreater(len(base), 1500)
        for code, values in fixed.items():
            self.assertEqual(set(values), base, code)
        literals = {}
        pattern = re.compile(r'\b(?:iced_)?text\(\s*("(?:[^"\\]|\\.)*")\s*\)', re.S)
        # These three are UI symbols, not localizable natural language.
        icons = {"●", "🌐", "›", "⚠"}
        for source in (MAIN, SHELL):
            for match in pattern.finditer(source):
                value = json.loads(match.group(1))
                if value.strip() and value not in icons:
                    literals[value] = True
        self.assertGreaterEqual(len(literals), 30, "literal source audit must actually enumerate UI strings")
        self.assertFalse(set(literals) - base)

    def test_default_window_dimensions_match_reference_screenshot(self):
        self.assertIn("const WINDOW_WIDTH: f32 = 1085.0;", MAIN)
        self.assertIn("const WINDOW_HEIGHT: f32 = 745.0;", MAIN)
        self.assertIn("resizable: true,", MAIN)
        self.assertIn("min_size: Some(Size::new(820.0, 720.0))", MAIN)

    def test_three_rom_chevron_rails_follow_active_accent(self):
        self.assertIn(".style(lpm_nav_rom_chevron_accent_style)", MAIN)
        self.assertIn("let accent = lpm_shell_palette(theme).accent;", MAIN)
        self.assertNotIn("Color::from_rgb8(219, 224, 240)", MAIN)
        self.assertIn("lpm_nav_rom_chevron_accent_style(&theme)", MAIN)

    def test_adb_warning_is_top_banner_not_an_accent_card(self):
        dash = SHELL.split("fn gbst_dashboard_panel(&self)", 1)[1].split(
            "fn gbst_footer(&self)", 1
        )[0]
        self.assertIn("shell_warning_palette(theme)", SHELL)
        self.assertIn(".style(shell_warning_outer_style)", dash)
        self.assertIn(".style(shell_warning_inner_style)", dash)
        self.assertIn(".style(shell_warning_button_style)", dash)
        self.assertIn('text("⚠")', dash)
        self.assertIn("Message::KillExternalAdbServer", dash)
        self.assertLess(dash.index("content.push(warning_card)"),
                        dash.index("content.push(self.gbst_device_card())"))
        self.assertNotIn("shell_highlight_style", dash)

    def test_rom_picker_validation_and_fitting_summary_are_centered(self):
        page = MAIN.split("let main_body: Element<'_, Message>", 1)[1].split(
            "let page_column =", 1
        )[0]
        self.assertIn("lpm_rom_panel_can_center(", page)
        self.assertIn("container(rom_panel)", page)
        self.assertIn("container(iced::widget::scrollable(rom_panel)", page)
        picker = MAIN.split("fn rom_image_folder_select_panel(&self)", 1)[1].split(
            "fn rom_select_folder_card(&self)", 1
        )[0]
        self.assertIn(".align_x(iced::alignment::Horizontal::Center)", picker)
        self.assertIn(".align_y(iced::alignment::Vertical::Center)", picker)
        helper = MAIN.split("fn lpm_rom_panel_can_center(", 1)[1].split(
            "fn lpm_rom_card_width(", 1
        )[0]
        self.assertIn("if !has_firmware_info", helper)
        self.assertIn("lpm_firmware_summary_two_columns(", helper)

    def test_external_adb_is_checked_at_every_rom_start_entry(self):
        handlers = (
            "RomContinueFromOptions",
            "StartConvertWipe",
            "StartRowUpdateKeepData",
            "StartReinstallWipe",
        )
        for handler in handlers:
            start = MAIN.index("Message::" + handler + " => {")
            segment = MAIN[start: start + 1550]
            self.assertIn("self.rom_external_adb_conflict()", segment, handler)
            if handler != "RomContinueFromOptions":
                self.assertLess(
                    segment.index("self.rom_external_adb_conflict()"),
                    segment.index("self.busy = true"),
                    handler
                )
        self.assertIn("lpmbox_device::adb_server_running()", MAIN)
        self.assertIn("if self.external_adb_server_active {", SHELL)

    def test_external_adb_rom_block_message_is_translated(self):
        for language in LANGS:
            catalog = json.loads((ROOT / "crates/lpmbox-core/lang" /
                                  (language + ".json")).read_text(encoding="utf-8"))
            self.assertTrue(catalog["lpm.adb.rom_blocked"].strip(), language)

    def test_dashboard_original_rom_slot_country_ota_rows(self):
        section = SHELL.split("fn gbst_device_card(&self)", 1)[1].split(
            "fn gbst_dashboard_panel(&self)", 1
        )[0]
        for field, label in (
            ("d.original_rom", "lpm.device.original_rom"),
            ("d.slot_suffix", "lpm.device.slot_suffix"),
            ("d.country_code", "lpm.device.country_code"),
            ("d.ota_status", "lpm.device.ota_status"),
        ):
            self.assertIn(field, section)
            self.assertIn(label, section)
        self.assertIn("left_details.into_iter().zip(right_details.into_iter())", section)
        self.assertIn("Self::shell_device_details_two_columns(", section)
        self.assertIn("left_details.into_iter().chain(right_details.into_iter())", section)
        self.assertIn("Wrapping::WordOrGlyph", section)

    def test_original_dashboard_adb_data_reuse(self):
        device = (ROOT / "crates/lpmbox-device/src/lib.rs").read_text(encoding="utf-8")
        for prop in (
            'dashboard_getprop(&mut adb, "ro.boot.region")',
            'dashboard_getprop(&mut adb, "ro.boot.slot_suffix")',
            'dashboard_getprop(&mut adb, "gsm.product.countrycode")',
            'dashboard_getprop(&mut adb, "ro.config.zui.region")',
            'dashboard_has_package(&mut adb, "com.lenovo.ota")',
        ):
            self.assertIn(prop, device)
        self.assertIn("ota_status: dashboard_ota_status_label(", device)

    def test_dashboard_extra_fields_and_ota_statuses_translated(self):
        new_keys = (
            "lpm.device.original_rom",
            "lpm.device.slot_suffix",
            "lpm.device.country_code",
            "lpm.device.ota_status",
            "lpm.device.ota.block_required",
            "lpm.device.ota.blocked",
            "lpm.device.ota.in_use",
            "lpm.device.ota.not_in_use",
            "lpm.device.ota.detected",
        )
        for lang in LANGS:
            content = json.loads((ROOT / "crates/lpmbox-core/lang" /
                                  (lang + ".json")).read_text(encoding="utf-8"))
            for key in new_keys:
                self.assertTrue(content[key].strip(), (lang, key))

    def test_all_twelve_language_catalogs_have_matching_nonempty_keys(self):
        catalogs = []
        for code in LANGS:
            with (ROOT / f"crates/lpmbox-core/lang/{code}.json").open(encoding="utf-8") as f:
                content = json.load(f)
            self.assertTrue(all(isinstance(v, str) and v.strip() for v in content.values()))
            catalogs.append(content)
        expected = set(catalogs[0])
        self.assertGreater(len(expected), 400)
        for code, content in zip(LANGS, catalogs):
            self.assertEqual(set(content), expected, code)


class UpdatePopupRegressionTests(unittest.TestCase):
    def test_update_popup_uses_flexible_content_and_wrapping(self):
        popup = MAIN.split("fn dashboard_update_notice_view(&self)", 1)[1].split(
            "fn startup_disclaimer_dialog(&self)", 1
        )[0]
        self.assertIn(".max_width(560.0)", popup)
        self.assertIn(".height(Length::Shrink)", popup)
        self.assertIn("Wrapping::WordOrGlyph", popup)
        self.assertNotIn(".height(Length::Fixed(150.0))", popup)
        self.assertNotIn(".width(Length::Fixed(260.0))", popup)
        for key in ("title", "description", "current", "latest", "install", "later", "detected"):
            self.assertIn('self.catalog("lpm.update.notice.' + key + '")', popup)

    def test_update_popup_labels_present_in_all_languages(self):
        directory = ROOT / "crates/lpmbox-core/lang"
        for locale in LANGS:
            dictionary = json.loads((directory / (locale + ".json")).read_text(encoding="utf-8"))
            for key in ("title", "description", "current", "latest", "install", "later", "detected"):
                label = dictionary["lpm.update.notice." + key]
                self.assertTrue(label.strip(), (locale, key))
                self.assertNotIn("\\n", label, (locale, key))


if __name__ == "__main__":
    unittest.main()
