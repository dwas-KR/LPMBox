# SPDX-License-Identifier: GPL-3.0-only
"""Keep the public source build independent of external SP Flash Tool binaries."""
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPFT = (ROOT / "crates/lpmbox-spft/src/lib.rs").read_text(encoding="utf-8")

class ExternalSpftDistributionTest(unittest.TestCase):
    def test_no_proprietary_spft_archive_required_at_compile_time(self):
        self.assertIn("const SPFT_TOOL_ZIP: &[u8] = &[];", SPFT)
        self.assertNotIn('"/assets/SPFlashToolV6_tool.zip"', SPFT)
        self.assertFalse((ROOT / "crates/lpmbox-spft/assets/SPFlashToolV6_tool.zip").exists())

    def test_download_cache_fallback_flow_is_preserved(self):
        stage = SPFT.split("fn ensure_embedded_spflashtoolv6_tool()", 1)[1].split(
            "fn download_official_spflashtoolv6_zip()", 1
        )[0]
        self.assertIn("download_official_spflashtoolv6_zip()", stage)
        self.assertIn("cached_official_spflashtoolv6_zip()", stage)
        self.assertIn("extract_embedded_spflashtoolv6_zip(&tool_dir)", stage)
        self.assertIn("if SPFT_TOOL_ZIP.is_empty()", SPFT)

if __name__ == "__main__":
    unittest.main()
