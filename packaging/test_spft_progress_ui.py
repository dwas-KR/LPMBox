"""Source-level regression checks for the SPFT readback window and progress UI.

Hardware handshake and flash results must be validated separately on a
recovery-capable test tablet; unit tests cannot prove live V6 DA readiness.
"""
import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GUI = (ROOT / "crates/lpmbox-gui/src/main.rs").read_text(encoding="utf-8")
SHELL = (ROOT / "crates/lpmbox-gui/src/gbst_shell.rs").read_text(encoding="utf-8")
SPFT = (ROOT / "crates/lpmbox-spft/src/lib.rs").read_text(encoding="utf-8")
LANG_DIR = ROOT / "crates/lpmbox-core/lang"

def section(name):
    start = GUI.index("fn " + name + "(")
    end = GUI.find("\nfn ", start + 5)
    return GUI[start:end if end >= 0 else None]


class SpftReadbackPrearmTests(unittest.TestCase):
    def test_both_readback_flows_prearm_before_usb_reboot(self):
        for method in ("run_optional_country_proinfo_backup",
                       "run_proinfo_backup_flow"):
            body = section(method)
            prearm = body.index("PrearmedProinfoReadback::start(")
            watcher = body.index("PreloaderWatch::start(30)")
            reboot = body.index("trigger_rom_install_reboot_commands(")
            self.assertLess(prearm, watcher)
            self.assertLess(watcher, reboot)
            self.assertIn("prearmed_readback.cancel();", body)
            self.assertIn("prearmed_readback.finish(", body)
            self.assertNotIn("execute_proinfo_readback_streaming(", body)

    def test_prearm_has_own_child_cancellation_and_watchdog(self):
        self.assertIn("pub struct PrearmedProinfoReadback", SPFT)
        self.assertIn("started_rx.recv_timeout", SPFT)
        self.assertIn("readback_timeout", SPFT)
        self.assertIn("Some(Duration::from_secs(180))", SPFT)
        self.assertIn("let _ = child.kill();", SPFT)
        self.assertIn("let _ = child.wait();", SPFT)
        self.assertIn("PreLoader 검증 실패", SPFT)
        self.assertIn('command.arg("-i").arg(&plan.config_xml);', SPFT)

    def test_spft_error_has_real_diagnostics(self):
        self.assertIn("elapsed={}ms / stdout={} / stderr={}", SPFT)
        self.assertIn("SPFT 로그 폴더", SPFT)
        self.assertIn("ProinfoLiveEvent::Log(format!(\"[backup] {message}\")",GUI)

    def test_backup_failure_skips_success_only_five_second_wait(self):
        body = section("run_optional_country_proinfo_backup")
        self.assertLess(body.index("let result = match readback_result"),
                        body.index("backup_file_unlock_wait"))

    def test_firmware_flash_stages_unmodified(self):
        self.assertEqual(GUI.count("lpmbox_spft::execute_firmware_download_streaming("),4)
        self.assertEqual(GUI.count("lpmbox_spft::execute_proinfo_readback_streaming("),0)
        self.assertIn('command.arg("-c").arg("download");',SPFT)


class GbstExecutionUiTests(unittest.TestCase):
    def test_log_page_uses_execution_summary_timeline_and_logs(self):
        self.assertIn("self.gbst_rom_execution_panel()", GUI)
        self.assertIn("self.gbst_execution_summary()",SHELL)
        self.assertIn("self.gbst_execution_timeline(",SHELL)
        self.assertIn("self.gbst_log_panel()",SHELL)

    def test_overall_percentage_is_stage_based_not_fake_bytes(self):
        self.assertIn("fn gbst_execution_overall_progress(",SHELL)
        self.assertIn("if self.runtime_last_outcome == Some(true) { return 100.0; }",SHELL)
        self.assertIn("(reached as f32 * 100.0 / 7.0).min(99.0)",SHELL)
        self.assertIn("fn gbst_execution_elapsed(",SHELL)
        self.assertIn("self.latest_spft_progress()",SHELL)
        self.assertIn("runtime_log_start_index",SHELL)
        self.assertIn("reached.unwrap_or(0).max(index - 1)",SHELL)
        self.assertNotIn('self.catalog("lpm.exec.back_to_rom")',SHELL)

    def test_all_twelve_json_catalogs_have_new_ui_labels(self):
        language_files = ["ko","en","ja","zh-TW","ru","vi","es","el",
                          "hi","ka","nl","ar"]
        keys = ["lpm.exec.status.running","lpm.exec.status.completed",
                "lpm.exec.status.failed","lpm.exec.status.ready",
                "lpm.exec.milestones","lpm.exec.stage_basis",
                "lpm.exec.stage_progress","lpm.exec.connection.adb",
                "lpm.exec.connection.preloader"]
        for language in language_files:
            with self.subTest(language=language):
                data = json.loads((LANG_DIR / f"{language}.json").read_text(encoding="utf-8"))
                for key in keys:
                    self.assertTrue(data.get(key),f"{language}:{key}")
                self.assertEqual(data["lpm.exec.connection.preloader"],"PreLoader")


if __name__ == "__main__":
    unittest.main()
