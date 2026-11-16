# LPMBox

**LPMBox v4.0.1** is an open-source Windows utility for firmware and device-management workflows on selected **Lenovo MediaTek tablets**.

[Download releases](https://github.com/dwas-KR/LPMBox/releases) · [Report an issue](https://github.com/dwas-KR/LPMBox/issues) · [Video guide](https://youtu.be/M3hUF-AVJbM)

## 🌐 LPMBOX Languages
English (en) / 한국어 (ko) / Русский (ru) / 日本語 (jp)
繁體中文 (CN/TW) / Tiếng Việt (vi) / Ελληνικά (el) / हिन्दी (hi)
ქართული (ka) / Nederlands (nl) / العربية (ar) / Español (es)

Setup and troubleshooting guides: [한국어](https://dwas.tistory.com/19) · [English](https://dwas.tistory.com/20) · [日本語](https://dwas.tistory.com/23) · [Русский](https://dwas.tistory.com/25) · [繁體中文](https://dwas.tistory.com/27)

## Features

- **ROM installation:** PRC (China) ↔ ROW (Global) firmware workflows; a clean installation erases user data.
- **ROW updates:** Firmware update workflows for supported device and version combinations.
- **Flashing safeguards:** Model, firmware, LK-image integrity and partition/slot checks.
- **Device tools:** ADB, Fastboot, MediaTek PreLoader and SP Flash Tool integration.
- **Desktop UI:** Multiple languages, operation history, logs and settings.

## Device families

| PRC (China) | ROW (Global) | Family |
| --- | --- | --- |
| TB375FC | TB373FU | Xiaoxin Pad Pro 2025 |
| TB335FC | TB336FU | Xiaoxin Pad 2025 |
| TB365FC | TB361FU | Xiaoxin Pad 12.1 |

## Getting started

1. **Configure Windows Security**
   - Open **Windows Security → Device security → Core isolation**.
   - Disable **Memory integrity** if required for MediaTek driver compatibility.

2. **Download and launch LPMBox**
   - Download the [latest release](https://github.com/dwas-KR/LPMBox/releases/latest).
   - Extract the archive and run `lpmbox.exe`.

3. **Download the official Lenovo firmware**
   - Download and install [Lenovo Software Fix](https://support.lenovo.com/us/en/downloads/ds101291).
   - Download the latest official firmware compatible with your exact tablet model.

4. **Select the firmware folder**
   - Open the **ROM Installation** tab in LPMBox.
   - Select the `image` folder from the downloaded firmware package.
   - LPMBox will automatically download and prepare the required components, including:
     - [MediaTek USB drivers](https://mtkdriver.com/)
     - [Microsoft Visual C++ Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170)
     - [SP Flash Tool V6](https://spflashtools.com/windows/sp-flash-tool-v6-2404/)
     - Other required dependencies

5. **Choose an operation**
   - Click **Next** and select the appropriate option:
     - **Firmware Installation** — Install a compatible firmware package.
     - **Firmware Update** — Update to a supported firmware version.
     - **Device Recovery** — Recover a supported device from firmware-related issues.
   - Follow the on-screen instructions to complete the process.

6. **Complete the setup**
   - Once the operation finishes successfully, your Lenovo tablet can run the selected **ROW (Global)** or **PRC (China)** firmware, depending on device compatibility.

> **Warning:** Firmware flashing may erase user data, cause boot failures, or affect warranty coverage. Back up important data before proceeding, verify firmware compatibility, and never disconnect your device during an active flashing operation.

## Build from source

**Requirements:** Windows 10/11 (x64), Rust (see [rust-toolchain.toml](rust-toolchain.toml)), Visual Studio C++ Build Tools and the Windows SDK.

```powershell
cargo build --release --locked -p lpmbox-gui
```

Output: `target/release/lpmbox.exe`

## Credits

Thanks to the following projects and contributors for sharing research, tools, information and technical guidance that helped shape LPMBox.

- **[LTBox(miner7222)](https://github.com/miner7222/LTBox)**
- **[TB375FC-LK-patch(ifg)](https://github.com/ifg/TB375FC-LK-patch)**
- **[hitin911](https://xdaforums.com/m/hitin911.12861404/)**
- **[mistq](https://cafe.naver.com/f-e/cafes/27738104/articles/705314?boardtype=L&menuid=611&referrerAllArticles=false)**
- **[Anonymousㅇㅇ](https://gall.dcinside.com/board/lists?id=tabletpc)**

## License

LPMBox's original source code is licensed under the **[GNU General Public License v3.0 (GPL-3.0-only)](LICENSE)**.

Third-party code and assets retain their original licenses. See [Third-party licenses and notices](THIRD_PARTY_LICENSES/README.md) for attribution and historical license information. The project's GPLv3 declaration does not override third-party rights or grant permission to redistribute firmware-derived LK images and manufacturer imagery.
