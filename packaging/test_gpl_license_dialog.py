# SPDX-License-Identifier: GPL-3.0-only
"""Guard the GPLv3 information dialog against duplicate legacy notices."""
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SHELL = ROOT / "crates/lpmbox-gui/src/gbst_shell.rs"

class GplLicenseDialogTests(unittest.TestCase):
    def test_gplv3_popup_shows_single_current_license(self):
        source = SHELL.read_text(encoding="utf-8")
        popup = source.split("fn gbst_license_popup(&self)", 1)[1].split(
            "fn gbst_", 1
        )[0]
        self.assertIn('const OWN_LICENSE: &str = include_str!("../../../LICENSE");', popup)
        self.assertIn("GNU GPL v3.0 (GPL-3.0-only)", popup)
        self.assertIn("raw_iced_text(OWN_LICENSE)", popup)
        self.assertEqual(popup.count('self.catalog("lpm.about.third_party_notice")'), 1)
        self.assertNotIn("OLD_LICENSE", popup)
        self.assertNotIn('license_row("LTBox"', popup)
        self.assertNotIn("lpm.about.prior_license", popup)

    def test_licenses_preserved_and_changelog_not_required(self):
        self.assertTrue((ROOT / "LICENSE").read_text(
            encoding="utf-8"
        ).startswith("GNU GENERAL PUBLIC LICENSE"))
        self.assertTrue((ROOT / "THIRD_PARTY_LICENSES" /
                         "PRIOR_RELEASE_CC_BY_NC_SA_4.0.txt").is_file())
        self.assertTrue((ROOT / "crates/lpmbox-gui/assets/flags/LICENSE").is_file())
        self.assertFalse((ROOT / "CHANGELOG_v4.0.1.md").exists())
        self.assertNotIn("CHANGELOG_v4.0.1.md", (ROOT / "README.md").read_text(
            encoding="utf-8"
        ))

if __name__ == "__main__":
    unittest.main()
