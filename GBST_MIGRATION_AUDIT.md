# GBST v2.0.0 GUI to LPMBox v3.1.2 Migration and Verification Audit

Date: 2026-11-09
Verified source: G:/TB375FC, TB373FU/GBST, Git HEAD: 3d5bc964f6c7f92e6f1e52c4399a6b93031b41a2
Modified target: G:/TB375FC, TB373FU/LPMBox-rs-3.1.2, Git HEAD: 404953a
This is local-only development. The remote GitHub repositories have not been modified.

## 1. Goals and explicit scope
Bring the exact visual design structure of GBST v2.0.0 into LPMBox while retaining LPMBox ROM install, recovery, firmware checking, MTK, USB, Fastboot, SP Flash Tool and Proinfo backup workers.
The output is the GBST GUI shell adapted to LPMBox operations. Google Services installation itself is NOT imported because it is an application-specific flow, not a ROM flashing flow.
Do not claim the whole GMS application logic or all its copy/data models were transferred.

## 2. Actual source comparison
GBST: 3 Rust workspace crates: gbst-core, gbst-adb, gbst-gui. GUI source: gbst-gui/src/main.rs and gbst-gui/src/ltbox_visual.rs, the latter with GBST Material-style shell visuals.
LPMBox: 5 Rust workspace crates: lpmbox-core, lpmbox-device, lpmbox-firmware, lpmbox-spft, lpmbox-gui.
Both use Rust 2024 and Iced 0.14, allowing a direct source-level visual port with product-specific message handling.

## 3. Replaced runtime GUI layout
The actual App.view of lpmbox-gui/src/main.rs now uses a GBST shell rather than the former main dashboard UI:
- Same material colors and colors for dark/light, shared ShellColors, rounded cards, accent highlights.
- Same native window initial 820 x 720 and minimum 820 x 720, resizable.
- Same responsive breakpoint (narrow < 1080; wide >= 1080) and collapsed 80px / expanded 232px sidebar.
- GBST pill-style selected navigation, hover animation, floating overlay in compact windows and docked navigation in wide windows.
- Matching 64px top page header and full-width status footer.
- GBST device card with tablet image and model/Android/ROM/firmware fields; GBST generic Etc image for disconnected state.
- Three GBST-style dashboard cards (developer support, LPMBox ROM operations, developer YouTube) wired to LPMBox actions.
- Same card-style four-section Settings interface (appearance, connection, files, program), language selector, System/Light/Dark segmented controls, Indigo/Teal/Rose segmented accent controls, file paths with tooltip.
- Separate About page with centered LPMBox logo, developer links based on source GBST SVG controls.
- Original GBST license-presentation style: 520x590 modal, scrim, full scrollable GPLv3 text and independent scrollbar gutter, historical copyright notice.
- Confirmation modals for log and incomplete download cleanup.
- GBST-style ROM-operation execution panel, summary/status, activity log, real worker log-derived timeline; no invented flashing percentages.
- GBST original dark navigation bitmap recoloring, improved specifically for icons with white opaque interiors.
- Removed construction of obsolete unrendered LPMBox settings/log widgets in App.view; original work routines preserved.

File containing re-used/adapted GBST GUI primitives:
crates/lpmbox-gui/src/gbst_shell.rs.
Source GBST reference: crates/gbst-gui/src/ltbox_visual.rs.

## 4. Exact functional mappings
| GBST screen or action | LPMBox adapted screen or action | Safety |
| --- | --- | --- |
| Dashboard device card | LPMBox DashboardDeviceInfo via own USB workers | Read-only |
| Google service card | ROM Operations navigation | No GMS command imported |
| Donate card | Existing LPMBox OpenDonate | Existing link |
| Developer YouTube | Existing LPMBox OpenDeveloperYoutube | Existing link |
| Google Services installer | Existing LPMBox ROM selection + firmware validation | Original flash workflow |
| Installation progress | Real worker message stage labels + existing LPMBox log | Never claims unsupported progress |
| Log management | GBST style Activity Log, Export/Clear | Original LPMBox handlers |
| Language | Existing LanguageOption and GBST JSON label catalog | Old detailed ROM i18n preserved |
| Theme / Accent | GBST appearance.rs and Settings messages | Local config only |
| Cache cleanup | LPM app-owned interrupted files only | Firmware ZIP kept |
| Log cleanup | LPM own log files only | Backups and keys kept |
| About / GPL | LPMBox logo/links plus GPLv3 text, old license retained | Third-party rights respected |
| Driver / Firmware Download / QnA | LPMBox pre-existing actions | No GMS implementation mixed in |

## 5. Localization
All original GBST 2.0.0 locale JSON keys were merged, unchanged, into LPMBox's own locale resources.
12 languages supported: en, ko, ja, zh-TW, ru, vi, es, el, hi, ka, nl, ar.
Verified 393 keys per language, including original GBST keys and additional LPMBox GUI terms for ROM, driver, backup, current execution phases, licenses and logging.
Existing LPMBox thousands of lines of runtime/ROM-specific translations remain in main.rs to avoid destructive translation changes.
All available labels are NOT guaranteed fully translated outside these guaranteed catalogs: legacy LPM log paths may contain untranslated device-specific strings.
Japanese legacy code jp remains compatible with JSON ja.

## 6. Licenses and third-party rights
Root LICENSE is verbatim GBST GPLv3 and Cargo.toml declares GPL-3.0-only.
Previous LPMBox CC BY-NC-SA 4.0 license text is preserved at THIRD_PARTY_LICENSES/PRIOR_RELEASE_CC_BY_NC_SA_4.0.txt.
This GPL declaration applies only to material the author has the right to license.
Previous README credited LTBox and referenced its license. Any literal third-party LTBox-derived source or graphics may remain subject to its original license. A prior development permission alone does not guarantee permission to relicense a CC BY-NC-SA 4.0 contribution into GPLv3-only. Confirm all third-party rights before public GPL-only distribution.
Original GBST sources and their repository are untouched.

## 7. Safety and firmware preservation
Git diff verified no source modification under crates/lpmbox-device, crates/lpmbox-firmware, crates/lpmbox-spft.
ROM install, backup, image checks, driver operations, actual MTK PreLoader and SP Flash Tool flows are still LPMBox's own functions.
Test application was launched and clicked on desktop only; no connected tablet operation, ROM flash, recovery flash, or device reboot was initiated.
Runtime settings saved under LOCALAPPDATA/dwas_kr/LPMBox/appearance.json; temporary dark-mode preferences used for visual tests were removed/restored in finally.

## 8. Evidence from real Windows UI
All listed images are actual desktop screen captures of LPMBox running on the user-authorized PC, not graphical mockups or AI-generated renders:
ResearchReadOnly/LPMBox_GBST2_GUI_20261109.png: Dashboard
ResearchReadOnly/LPMBox_GBST2_Settings_20261109.png: Settings card sections and segmented theme
ResearchReadOnly/LPMBox_GBST2_Rom_20261109.png: Existing ROM image folder functionality inside new shell
ResearchReadOnly/LPMBox_GBST2_About_20261109.png: About, developer links and license
ResearchReadOnly/LPMBox_GBST2_Log_20261109.png: Activity log
ResearchReadOnly/LPMBox_GBST2_DarkDashboard_20261109.png: Dark mode
ResearchReadOnly/LPMBox_GBST2_DarkLicense_20261109.png: Scrollable GPLv3 license pop-up.
Screenshots are all under G:/TB375FC, TB373FU/ResearchReadOnly.

## 9. Programmatic validation
- cargo check -p lpmbox-gui --offline: passed.
- cargo test --workspace --offline --quiet: 25 tests passed, zero failed (2 core + 23 GUI).
- Includes GBST layout palette, breakpoint, 720px sidebar-height, translated ROM nav terms; also existing LPM i18n and safe cleanup tests.
- cargo build --release -p lpmbox-gui --offline: final release build PASSED after dark-mode image and locale fixes.
- Final Windows binary size: 46231552 bytes.
- Final Windows binary SHA-256: 81575CCF039FE9ACB69BA6934BE6D2A12C9EF0C478D829C01347C40F66542A66.
- Final build metadata: GBST_GUI_BUILD_MANIFEST_20261109.json.
- git diff --check: passed.
- There are harmless Rust warnings for obsolete LPM visual helper functions retained for safe rollback.
- Actual GUI launch, dashboard/settings/ROM/About/log navigation, dark theme and license popup inspected.
- Not tested: actual ROM flashing, recovery, device drivers, all devices, full 12-language live screenshots or all release link actions. Test unit pass is not proof of flashing compatibility.

## 10. Comprehensive GBST source inventory (190 entries)
The accompanying GBST_SOURCE_INVENTORY_20261109.json contains every original tracked file included in source scope, its byte count, category, port status, reason, and LPM destination. Status breakdown:
| Status | Files |
| --- | ---: |
| ALL_GBST_LOCALE_KEYS_MERGED | 12 |
| ARCHITECTURE_ADAPTED | 1 |
| ASSET_COPIED | 5 |
| ASSET_REVIEWED_NOT_COPIED | 27 |
| GBST_SPECIFIC_BACKEND_UNCHANGED | 9 |
| GUI_CONTROLLER_ADAPTED | 1 |
| GUI_RENDERER_PORTED | 1 |
| I18N_DATA_MERGED_BACKEND_PRESERVED | 2 |
| LICENSE_MATCHED | 1 |
| MODULE_ADAPTED | 3 |
| NOT_PORTED | 126 |
| PURPOSE_SPECIFIC_BACKEND_NOT_COPIED | 2 |

GBST GMS-specific backend remains separate. Original GBST illustrations, media resources and installation scripts are not blindly copied where LPMBox has device-specific equivalents.

## 11. Recovery and reproducibility
First-preflight Git backup branch: backup/pre-gbst-v2-port-20261109
Original pre-GUI port backup copy:
G:/TB375FC, TB373FU/ResearchReadOnly/LPMBox_before_full_GBST_GUI_20261109
All build source editing scripts are under:
G:/TB375FC, TB373FU/ResearchReadOnly
No GitHub push, PR, ROM file write or device modification performed.

## 12. Final distribution conditions
Before public release, resolve any LTBox/other third-party license constraints, test the GUI on the target Windows DPI/scaling modes, and perform safe non-destructive device tests, followed by separately authorized ROM installation regression testing.
Current output is a locally tested beta GUI integration, not a proven universal-device ROM flasher release.
