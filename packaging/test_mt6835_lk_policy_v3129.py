"""Model-specific ROW LK patch requirement regression (no device writes)."""
from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[1]
LK=(ROOT/"crates/lpmbox-firmware/src/lk_model_policy.rs").read_text(encoding="utf-8")
PLAN=(ROOT/"crates/lpmbox-firmware/src/flash_plan.rs").read_text(encoding="utf-8")
CORE=(ROOT/"crates/lpmbox-firmware/src/lib.rs").read_text(encoding="utf-8")

class ModelLKPolicyTests(unittest.TestCase):
    def test_new_lk_policy_module_wired(self):
        self.assertIn("pub mod lk_model_policy;",CORE)
        self.assertIn("lk_model_policy::enforce_before_flash_plan(",PLAN)

    def test_gate_precedes_any_row_file_delete_or_download(self):
        before=PLAN.index("lk_model_policy::enforce_before_flash_plan(")
        after=PLAN.index("remove_row_lk_dtbo_img_files(firmware.region")
        download=PLAN.index("prepare_extra_partition_files_for_model(&firmware.model")
        self.assertLess(before,after)
        self.assertLess(before,download)

    def test_no_unverified_mt6897_patch_on_mt6835(self):
        self.assertIn("TB336FU",LK)
        self.assertIn("TB335FC",LK)
        self.assertIn("TB361FU",LK)
        self.assertIn("Mt6835VerifiedPatchMissing",LK)
        self.assertIn("MT6897 TB375FC",LK)
        self.assertNotIn("patch_prc_103_lk(",LK)
        self.assertNotIn("patch_row_084_lk(",LK)

    def test_current_model_version_policy_matches_the_installed_code(self):
        for marker in (
            "parsed>[1,1,10,299]",
            "if parsed[0] == 17",
            "parsed>[1,1,10,122]",
            "parsed==[17,5,10,375]",
            "Tb335PrcSharedUserBoottested",
            "Tb361Row375Experimental",
            "Signature(Entity) invalid",
        ):
            self.assertIn(marker, LK)
        self.assertIn("preflight_lk_before_device_changes(", PLAN)

    def test_country_reset_and_existing_patched_tb373_are_unchanged(self):
        self.assertIn("InstallMode::CountryReset",LK)
        self.assertIn('("TB373FU", RomRegion::Row)',LK)
        self.assertIn("Tb373ExistingVerifiedPatch",LK)

if __name__=="__main__":
    unittest.main()
