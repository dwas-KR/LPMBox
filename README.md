# LPMBox

**LPMBox v4.0.1** is an open-source Windows utility for firmware and device-management workflows on selected **Lenovo MediaTek tablets**.

🌐 Languages & LPMBOX Error List and Solutions
English (en) / 한국어 (ko) / Русский (ru) / 日本語 (jp)
繁體中文 (CN/TW) / Tiếng Việt (vi) / Ελληνικά (el) / हिन्दी (hi)
ქართული (ka) / Nederlands (nl) / العربية (ar) / Español (es)

[Download releases](https://github.com/dwas-KR/LPMBox/releases) · [Report an issue](https://github.com/dwas-KR/LPMBox/issues) · [Video guide](https://youtu.be/M3hUF-AVJbM)

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

> **Compatibility is version-specific.** Listing a device does not guarantee that every ROM, region conversion or LK patch is supported. The application may block unverified configurations.

## Getting started

1. Download and extract the [latest release](https://github.com/dwas-KR/LPMBox/releases/latest).
2. Install the required MediaTek USB drivers. Enable USB debugging and authorize ADB when the selected operation requires it.
3. Obtain the **official firmware for your exact device model and region** through Lenovo's authorized channels; prepare its `image/` folder as instructed by the application.
4. Run `lpmbox.exe`, select the task and follow the on-screen checks.

> **Warning:** Firmware flashing can erase data, prevent a device from booting or affect warranty coverage. Back up important data first. Do not disconnect the device during an active operation.

## Build from source

**Requirements:** Windows 10/11 (x64), Rust (see [rust-toolchain.toml](rust-toolchain.toml)), Visual Studio C++ Build Tools and the Windows SDK.

```powershell
cargo build --release --locked -p lpmbox-gui
```

Output: `target/release/lpmbox.exe`

## Help

Setup and troubleshooting guides: [한국어](https://dwas.tistory.com/19) · [English](https://dwas.tistory.com/20) · [日本語](https://dwas.tistory.com/23) · [Русский](https://dwas.tistory.com/25) · [繁體中文](https://dwas.tistory.com/27)

## License and acknowledgments

LPMBox's original source code is licensed under **[GPL-3.0-only](LICENSE)**. Third-party software, icons, fonts and assets retain their own licenses; see [third-party notices](THIRD_PARTY_LICENSES/README.md). Earlier LPMBox license terms are preserved for historical reference and are not superseded for material governed by those terms. Redistributing firmware-derived LK images or manufacturer imagery requires separate rights verification.

Developed by [돠스 (dwas)](https://www.youtube.com/@dwas_KR). The project was developed with reference to [LTBox](https://github.com/jjhitel/LTBox) and is independent of its maintainers and Lenovo. Thanks to [community contributors](https://gall.dcinside.com/board/lists?id=tabletpc) and [hitin911](https://xdaforums.com/m/hitin911.12861404/) for shared research and technical guidance.
