# LPMBox

![License: CC BY-NC-SA 4.0](https://img.shields.io/badge/License-CC%20BY--NC--SA%204.0-lightgrey.svg)

> This project follows the **LTBox documentation style** and is released under the **LTBox license model**.
>
> * [LTBox](https://github.com/jjhitel/LTBox), [LTBox License](https://github.com/jjhitel/LTBox?tab=License-1-ov-file)
> * Inspired by LTBox and adapted for **MediaTek-based Lenovo tablet firmware workflows**.
> * **This project is not affiliated with or endorsed by the LTBox developer. I only received permission to develop LPMBox.**
> * Based on LTBox (CC BY-NC-SA 4.0); modified and extended by **돠스 (dwas)** for MTK Lenovo firmware workflows.
> * **NonCommercial:** Do not sell this project, offer paid access, or use it primarily for commercial advantage or monetary compensation.

---

## 1. Developer YouTube
* Maintained by: **돠스 (dwas)**
* [YouTube](http://www.youtube.com/@dwas_KR?sub_confirmation=1)

---

## 2. Overview

**LPMBox** is a helper tool designed to make PRC (CN) / ROW (Global) firmware operations easier on **MediaTek (MTK)-based Lenovo tablets**.

### Languages
* English / 한국어 / Русский / 日本語

### Target models
* Lenovo Xiaoxin Pad Pro 12.7 2025 2nd (TB375FC)
* Lenovo Xiaoxin Pad 12.1 (TB365FC)
* Lenovo Xiaoxin Pad 11 (TB335FC)
* Other Lenovo tablets using MediaTek Dimensity chipsets

> ⚠️ Note: Behavior may vary depending on device model, ROM, and SoC/platform.  
> Always use **official firmware packages** intended for your specific device.

---

## 3. Menu (Functions)

### 3.1 Option 1: Switch PRC (CN) ↔ ROW (Global) Firmware [Data Wipe]
Performs a clean install (factory reset). **All data will be erased.**  
Use this option to freely switch between **PRC (CN)** and **ROW (Global)** firmware.

### 3.2 Option 2: Update ROW (Global) Firmware [Keep Data]
Updates **ROW (Global)** firmware while keeping your user data as much as possible.

### 3.3 Option 3: Disable OTA
Disables system OTA update checks, notifications, and related components.

### 3.4 Option 4: MediaTek Driver Download
Opens the MediaTek driver download page and shows the current driver status  
(e.g., **Installed / Not installed**).

### 3.5 Option 5: Check for LPMBox Updates
Checks for the LPMBox version and updates the tool if an update is available.

### 3.6 Option 6: Change LPMBox Language
Lets you reselect your preferred UI language at any time.

### 3.7 Option 7: Developer YouTube
Introduces guides and other useful Lenovo tablet tools (ZUI / ZUXOS).

### 3.8 Exit
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
Run `start.cmd` and select an option.

---

## 5. Requirements

### Recommended environment
* Windows 10/11 (32-bit, 64-bit)
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
This project is provided for **learning, research, and personal use only**.  
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
LPMBox follows the same license model as **LTBox**.

This work is licensed under the  
**Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International (CC BY-NC-SA 4.0)**.

For full details, see the `LICENSE` file or visit:  
[https://creativecommons.org/licenses/by-nc-sa/4.0/](https://creativecommons.org/licenses/by-nc-sa/4.0/)

> ⚠️ **Note**  
> Third-party tools or files used or downloaded by LPMBox (e.g., SP Flash Tool, platform-tools, firmware packages)  
> are subject to **their own licenses and distribution terms**. Please review and comply with them separately.

[![CC BY-NC-SA 4.0][cc-by-nc-sa-image]][cc-by-nc-sa]

[cc-by-nc-sa]: https://creativecommons.org/licenses/by-nc-sa/4.0/
[cc-by-nc-sa-image]: https://licensebuttons.net/l/by-nc-sa/4.0/88x31.png
