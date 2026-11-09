# LPMBox v3.1.3

Released: 2026-11-09

## Interface and stability

- Adopted GBST v2.0.0-style GUI and responsive sidebar, settings, and About page.
- Suppressed the console window from Windows registry theme detection and cached the About icon handle to avoid flickering.
- Hid the standalone Proinfo Backup navigation entry, without removing backup or country-code procedures.
- Fixed firmware folder picking: direct image/ folder or parent-with-image/ resolves to the image/ folder; missing image/ shows an error before inspection.
- Replaced static bright ROM cards, dropdowns, and popup surfaces with light/dark-aware palette styles.

## Localization

- Added 12 locale JSON catalogs containing 1,512 fixed Korean-source strings each, generated from the existing translation machinery.
- Preserved existing base GBST interface JSON and 228 legacy exact translations.
- Dynamic device errors, USB communication logs, and programmatically formatted messages retain the original compatibility translator.
- Automated checks verified matching localization keys. Natural-language quality still benefits from native-speaker review.

## Verification

- Windows x86-64 Release compilation succeeded.
- 28 workspace tests passed, no failures.
- No physical device ROM flash, restore, wipe, reboot or EDL tests were run.
- This release does not claim GBST APK installer internals or 64 KiB USB optimization were transplanted into LPMBox.

## License

GPL-3.0-only applies to the author's original works. LTBox-derived portions and other third-party materials retain their own terms; check THIRD_PARTY_LICENSES and provenance before redistributing any combined binary.
