# LPMBOX

## v3.1.3 (2026-11-09)

- Fixed Windows startup console flash from registry-based theme lookup.
- Stabilized the About-screen brand icon during responsive-sidebar animation.
- Removed the separate Proinfo backup item from the sidebar without deleting the related device or country-code work functions.
- Accepted either the exact firmware image directory or its immediate parent; blocked invalid directory selections before the firmware worker starts.
- Made previously white ROM/driver/modal sections theme-aware under Dark mode.
- Added 12 fixed-text locale JSON catalogs covering 1,512 extracted Korean source strings, while preserving existing dynamic log fallbacks.
- Preserved ROM flash/recovery operations and workflows. No firmware flashing was performed in testing.
- Verified Rust workspace unit tests and a Windows x86-64 Release build.


![License: GNU GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)

> The original LPMBox project code is licensed under **GNU GPL v3 (GPL-3.0-only)**, like GBST v2.0.0. Third-party code and assets retain their own rights. See THIRD_PARTY_LICENSES.
>
> * [LTBox](https://github.com/jjhitel/LTBox), [LTBox License](https://github.com/jjhitel/LTBox?tab=License-1-ov-file)
> * Inspired by LTBox and adapted for **MediaTek-based Lenovo tablet firmware workflows**.
> * **This project is not affiliated with or endorsed by the LTBox developer. I only received permission to develop LPMBox.**
> * Developed with reference to LTBox (original CC BY-NC-SA 4.0 license); modified and extended by **돠스 (dwas)** for MTK Lenovo firmware workflows.
> * **GPLv3:** Commercial use and redistribution are permitted subject to the GPLv3 terms, including corresponding-source obligations. Third-party rights still apply.

---

## 1. Developer YouTube
* Maintained by: **돠스 (dwas)**
* [Youtube Channel](https://www.youtube.com/@dwas_KR?sub_confirmation=1)
* [Watch the video](https://youtu.be/M3hUF-AVJbM?si=CkAkY9Rv6jFtvQmm)
  
[![Watch the video](https://img.youtube.com/vi/M3hUF-AVJbM/0.jpg)](https://youtu.be/M3hUF-AVJbM?si=CkAkY9Rv6jFtvQmm)

---

## 2. Overview

**LPMBox** is a helper tool designed to make PRC (CN) / ROW (Global) firmware operations easier on **MediaTek (MTK)-based Lenovo tablets**.

### Languages & LPMBOX Error List and Solutions
* [English (en)](https://dwas.tistory.com/20) / [한국어 (ko)](https://dwas.tistory.com/19) / [Русский (ru)](https://dwas.tistory.com/25) / [日本語 (jp)](https://dwas.tistory.com/23)
* [繁體中文 (CN/TW)](https://dwas.tistory.com/27) / Tiếng Việt (vi) / Ελληνικά (el) / हिन्दी (hi)
* ქართული (ka) / Nederlands (nl) / العربية (ar) / Español (es)

### Target models
* Xiaoxin Pad Pro 2025 (TB375FC, TB373FU)
* Xiaoxin Pad 12.1 (TB365FC, TB361FU)
* Xiaoxin Pad 2025 (TB335FC, TB336FU)
* Other Lenovo tablets using MediaTek Dimensity chipsets

> ⚠️ Note: Behavior may vary depending on device model, ROM, and SoC/platform.  
> Always use **official firmware packages** intended for your specific device.

---

## 3. Menu (Functions)

### 3.1 Option 1: Install/Flash PRC (CN) ↔ ROW (Global) Firmware [Data Wipe]
Performs a clean installation (factory reset). **All user data will be erased.** 

### 3.2 Option 2: Update ROW (Global) Firmware [Keep Data]
Use this option to freely switch between **ROW (Global)** firmware.

### 3.3 Option 3: Disable OTA
Disables system OTA update checks, notifications, and related components.

### 3.4 Option 4: Enable OTA [China ROM Only]
Restores the system update app and OTA functionality on **PRC (China) ROM** devices.  
If the device is running **ROW firmware**, a warning will be displayed and the program will return to the main menu.

### 3.5 Option 5: Download MediaTek Driver
Opens the MediaTek driver download page and shows the current driver status  
(e.g., **Installed / Not installed**).

### 3.6 Option 6: Country Code Change Feature [ON/OFF]
Toggles the **Country Code Change** feature.

- When **OFF**, Menu 1 and Menu 2 will skip the country code prompt and display:
  `[*] Skipping country code feature.`

### 3.7 Option 7: Check for LPMBox Updates
Checks the current LPMBox version and updates the tool if a newer version is available.

### 3.8 Option 8: Change LPMBox Language
Allows you to reselect your preferred UI language at any time.

### 3.9 Option 9: Developer YouTube
Opens the developer's YouTube channel with guides and tools related to Lenovo tablets (ZUI / ZUXOS).

### 3.10 Exit
Press `x` to exit the program.

---

## 4. Quick Start (How to Use)

### 4.1 Download & Extract
Download the LPMBox release archive and **extract** it.

### 4.2 Install Drivers (Important)
You can install the MTK driver in either of the following ways:
* Use **Option 4: MTK Driver Download** from the LPMBox main menu
* Or manually download and install from:  
  [https://mtkdriver.com/](https://mtkdriver.com/)

**LPMBox detects whether the MediaTek driver is installed on your system.**

### 4.3 Prepare the `image/` Folder (Important)
Copy the official firmware **`image`** folder downloaded via Lenovo Software Fix into the LPMBox root directory.

Typical contents include:
* `image/`
* `image/flash.xml`
* `image/da.auth`
* `image/<platform>_Android_scatter.x` (e.g., `MT0000_Android_scatter.x`)
* `image/super.img`, `image/userdata.img`, `image/vendor.img`, etc.

> The exact `image/` structure depends on the firmware package.  
> Make sure you use **official firmware downloaded via Lenovo Software Fix**.

### 4.4 Run
Run lpmbox.exe and select the ROM task or device-management option in the graphical interface.

---

## 5. Requirements

### Recommended environment
* Windows 10/11 (64-bit x86-64)
* A stable USB cable/port (direct motherboard USB ports are recommended)
* **USB debugging (ADB) enabled and PC authorized**
* **MediaTek USB Port (Preloader) drivers installed**  
  (In Device Manager, it should appear as “MediaTek Preloader USB VCOM”)

### Required firmware files (provided by the user)
An official firmware/tool package for your device that includes:
* `flash.xml`
* `da.auth` (DA/Authentication file)
* `*_Android_scatter.x`
* Partition image files referenced by `flash.xml` (e.g., `*.img`)

---

## 6. ⚠️ Important: Disclaimer
This software is primarily intended for learning and research. The GPLv3 license grants software redistribution and commercial-use rights for code covered by that license; third-party components may have separate terms.
Firmware flashing and partition-level operations involve **serious risks**, including:
* Device **bricking** / boot failure
* **Data loss** (factory reset)
* Warranty void, region/service restrictions, and related issues

The author is not responsible for any damage or loss caused by using this tool.  
**You are solely responsible for all outcomes. Use at your own risk.**

---

## 7. Credits

### 7.1 Inspired by / Based on
* **LTBox** by **jjhitel** (and contributors)  
  [https://github.com/jjhitel/LTBox](https://github.com/jjhitel/LTBox)  
  (Licensed under CC BY-NC-SA 4.0 as stated in the LTBox README)

### 7.2 Special thanks
* **Anonymous[ㅇㅇ](https://gall.dcinside.com/board/lists?id=tabletpc)**: Thank you for sharing the LTBox project/files and making it possible to develop LPMBox.
* **[hitin911](https://xdaforums.com/m/hitin911.12861404/)**: For providing the method to decrypt `.x` to `.xml` and for guidance on modifying XML scripts.

---

## 8. Third-party
* [Android platform-tools](https://developer.android.com/tools/releases/platform-tools?hl=en) (ADB/Fastboot)
* [Python (embeddable)](https://www.python.org/downloads/windows/) / pip / open-source Python packages (e.g., [cryptography](https://pypi.org/project/cryptography/))
* [SP Flash Tool V6](https://spflashtools.com/windows/sp-flash-tool-v6-2404)

> LPMBox does **not** include Lenovo firmware.  
> Users must download official firmware via official channels (e.g., Lenovo Software Fix).

---

## 9. License

**LPMBox original source code: GNU General Public License version 3 (GPL-3.0-only).**

The full GPLv3 license is included in the root **LICENSE** file. This matches
GBST v2.0.0's declared source-code license.

- Source code is available to study, modify, redistribute, and use commercially
  under the GPLv3 requirements, including corresponding source availability.
- LPMBox and GBST are developed by dwas. LTBox is independent and not affiliated.
- Earlier LPMBox releases were published under CC BY-NC-SA 4.0. Their historical
  license text remains in THIRD_PARTY_LICENSES/PRIOR_RELEASE_CC_BY_NC_SA_4.0.txt.
- LTBox-derived code, icons, fonts, firmware, SP Flash Tool and other third-party
  materials retain their original rights. Verify provenance and obtain any
  necessary relicensing permission before distributing a GPLv3-only combination.
  GPLv3 project metadata alone does not override existing third-party licenses.

## 10. GBST v2.0.0-inspired architecture

This LPMBox revision ports the GBST v2.0.0 actual graphical shell and shared foundations while
**preserving LPMBox's ROM installation, readback, driver and recovery workers**.

- GBST-like Iced appearance preferences: system/light/dark, indigo/teal/rose,
  persistent at %LOCALAPPDATA%/dwas_kr/LPMBox/appearance.json.
- GBST v2.0.0 Material 3 shell primitives, exact pill-style navigation items,
  responsive sidebar, top page header, three dashboard cards, full-width footer,
  Settings cards, About screen, SVG links, GPLv3 dialog and ROM task timeline.
- Responsive 820x720 minimum main window, 1080px sidebar breakpoint,
  dark icon recoloring and theme-dependent Material 3 colors.
- File-backed 12-language locale catalogs (GBST-style UI, 228 legacy phrases,
  and 1,512 extracted fixed Korean-source text keys for each language)
  in crates/lpmbox-core/lang/. Dynamic device logs retain the legacy runtime
  translation fallback so firmware operation flow remains unchanged.
- Separate cautious, tested log and interrupted-download cleanup modules;
  firmware ZIP files, embedded tools, ADB keys and device backups are untouched.
- Exact GBST-style in-app GPLv3 scrollable license viewer and confirmation dialogs.
- Dedicated core ROM-flashing and device APIs are unchanged and stay outside the GUI port.
- This is a shared-architecture port, **not an import of GBST's Google APK installation
  workflow**. The two tools have different device-operation responsibilities.

See GBST_MIGRATION_AUDIT.md and GBST_SOURCE_INVENTORY_20261109.json for
file-by-file implementation status, remaining port limits and screenshot tests.
