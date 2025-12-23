# LPMBox
![License: CC BY-NC-SA 4.0](https://img.shields.io/badge/License-CC%20BY--NC--SA%204.0-lightgrey.svg)

> This project follows the LTBox documentation style and is released under the LTBox license model.  
> - [LTBox](https://github.com/jjhitel/LTBox), [LTBox License](https://github.com/jjhitel/LTBox?tab=License-1-ov-file)  
> - Inspired by LTBox and adapted for MediaTek-based Lenovo tablet firmware workflows.  
> - **Not affiliated with or endorsed by the LTBox developer. I only received permission to develop LPMBox.**  
> - Based on LTBox (CC BY-NC-SA 4.0); modified and extended by **돠스 (dwas)** for MTK Lenovo firmware workflows.  
> - **NonCommercial:** Do not sell this project, offer paid access, or use it primarily for commercial advantage or monetary compensation.

---

## ⚠️ Important: Disclaimer

This project is provided for **learning, research, and personal use only**.  
Firmware flashing and partition-level operations involve **serious risks**, including:

- Device **bricking** / boot failure
- **Data loss** (factory reset)
- Warranty void, region/service restrictions, and related issues

The author is not responsible for any damage or loss caused by using this tool.  
**You are solely responsible for all outcomes. Use at your own risk.**

---

## 1. Credits

### 1.1 Inspired by / Based on
- **LTBox** by **jjhitel** (and contributors)  
  https://github.com/jjhitel/LTBox  
  Licensed under CC BY-NC-SA 4.0 (as stated in the LTBox README)

### 1.2 Special thanks
- **Anonymous[ㅇㅇ](https://gall.dcinside.com/board/lists?id=tabletpc)**: Thank you for sharing the LTBox project and files, and for making it possible to develop LPMBox.
- **[hitin911](https://xdaforums.com/m/hitin911.12861404/)**: For providing the method to decrypt `.x` to `.xml` and for guidance on modifying XML scripts.

### 1.3 Third-party
- [Android platform-tools](https://developer.android.com/tools/releases/platform-tools?hl=en) (ADB/Fastboot)
- [Python (embeddable)](https://www.python.org/downloads/windows/) / pip / open-source Python packages (e.g., [cryptography](https://pypi.org/project/cryptography/))
- [SP Flash Tool V6](https://spflashtools.com/windows/sp-flash-tool-v6-2404)

> LPMBox does **not** include Lenovo firmware.  
> Users must download official firmware via official channels (e.g., Lenovo Software Fix).

---

## 2. Contact / Issues
- Maintained by: **돠스 (dwas)**
- [YouTube](http://www.youtube.com/@dwas_KR?sub_confirmation=1)
- [Bug reports / feature requests](http://pf.kakao.com/_HHVmG)

---

## 3. Overview

**LPMBox** is a helper tool designed to make PRC (CN) / ROW (Global) firmware operations easier on **MediaTek (MTK)-based Lenovo tablets**.

### Key features
- **Option 1:** Install PRC (CN) / ROW (Global) firmware **[Data Wipe / Factory Reset]**
- **Option 2:** Upgrade ROW (Global) firmware **[Keep Data]**
- Supports changing the **country code in `proinfo`** (e.g., `CNXX → KRXX`)

### Languages
- English / 한국어 / Русский / 日本語

### Target models
- Lenovo Xiaoxin Pad Pro 12.7 2025 2nd (TB375FC)
- Lenovo Xiaoxin Pad 12.1 (TB365FC)
- Lenovo Xiaoxin Pad 11 (TB335FC)
- Other Lenovo tablets using MediaTek Dimensity chipsets

> ⚠️ Note: Behavior may vary depending on device model, ROM, and SoC/platform.  
> Always use **official firmware packages** intended for your specific device.

---

## 4. Quick Start (How to Use)

### 4.1 Download & Extract
Download the LPMBox release archive and **extract** it.

### 4.2 Install Drivers (Important)
Download and install the MTK driver from:
- https://mtkdriver.com/

### 4.3 Prepare the `image/` Folder (Important)
Copy the official firmware **`image`** folder downloaded via Lenovo Software Fix into the LPMBox root directory
- `image/`
- `image/flash.xml`
- `image/da.auth`
- `image/<platform>_Android_scatter.x` (e.g., `MT0000_Android_scatter.x`)
- `image/super.img`, `image/userdata.img`, `image/vendor.img`, etc.

> The exact `image/` structure depends on the firmware package.  
> Make sure you use **official firmware downloaded via Lenovo Software Fix**.

### 4.4 Run
Double-click `start.cmd` (**Administrator** privileges recommended if needed).

On first run (depending on your system):
- Python (embedded or system Python) and required components may be prepared automatically.
- Logs are created in the `logs/` folder.

---

## 5. Menu (Functions)

### 5.1 Option 1: Firmware Installation [Data Initialization]
**Purpose**  
Clean install / factory reset style installation (**all data will be erased**).

**Typical use cases**
- Installing **ROW (Global)** firmware onto a **PRC (CN)** device
- Reinstalling firmware from scratch to recover a device

**Workflow (summary)**
1. Check device/platform information via ADB (connected device + MTK platform verification)
2. Convert and prepare encrypted `scatter.x` → XML scatter file
3. Replace/prepare PRC (CN) boot-related images (`lk.img`, `dtbo.img`)
4. Launch SP Flash Tool and guide the user to **backup `proinfo` (Readback)**
5. Detect the readback `proinfo` file → optionally change the country code
6. Proceed with full flashing (including `userdata`, data will be wiped)

### 5.2 Option 2: Firmware Upgrade [Keep Data]
**Purpose**  
Upgrade while keeping user data **as much as possible**.

**Behavior**
- Modifies the XML scatter so that the **`userdata` partition is skipped** during flashing.
- System partitions are updated, but `/data` is not reflashed.

**Limitations & recommended flow**
- If the currently installed ROM is **not a ROW (Global) build**, the tool may block this option for safety.
- Recommended:
  1. First complete **Option 1** (clean installation to ROW).
  2. Then use **Option 2** for future firmware upgrades while keeping data.

### 5.3 Option 3: Disable OTA
Disables system OTA update checks, notifications, and related “What’s New” / recommendation components on the device.

### 5.4 Option 4: Developer YouTube
We introduce guide and tutorial videos for LPMBox, along with programs you can use on Lenovo tablets (Xiaoxin Pad, Y700, GT, Yoga Pad Pro, etc.) running ZUI and ZUXOS.

---

## 6. Requirements

### Recommended environment
- Windows 10/11 x64
- A stable USB cable/port (direct motherboard USB ports are recommended)
- **USB debugging (ADB) enabled and PC authorized**
- **MediaTek USB Port (Preloader) drivers installed**  
  (In Device Manager, it should appear as “MediaTek USB Port” or “MediaTek Preloader USB VCOM”)

### Required firmware files (provided by the user)
An official firmware/tool package for your device that includes:
- `flash.xml`
- `da.auth` (DA/Authentication file)
- `*_Android_scatter.x`
- Partition image files referenced by `flash.xml` (e.g., `*.img`)

---

## 7. License

LPMBox follows the same license model as **LTBox**.

This work is licensed under the  
**Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International (CC BY-NC-SA 4.0)**.

### Key points
- **Attribution**  
  You must give appropriate credit, provide a link to the license, and indicate if changes were made.
- **NonCommercial**  
  You may **not** use this work for commercial purposes.
- **ShareAlike**  
  If you remix, transform, or build upon this work, you must distribute your contributions under the **same license** (CC BY-NC-SA 4.0).

For full details, see the `LICENSE` file or visit:  
https://creativecommons.org/licenses/by-nc-sa/4.0/

> ⚠️ **Note**  
> Third-party tools or files used or downloaded by LPMBox (e.g., SP Flash Tool, platform-tools, firmware packages)  
> are subject to **their own licenses and distribution terms**. Please review and comply with them separately.

[![CC BY-NC-SA 4.0][cc-by-nc-sa-image]][cc-by-nc-sa]

[cc-by-nc-sa]: https://creativecommons.org/licenses/by-nc-sa/4.0/
[cc-by-nc-sa-image]: https://licensebuttons.net/l/by-nc-sa/4.0/88x31.png
