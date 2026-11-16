// Windows-only low-latency MediaTek PreLoader COM detection.
//
// The preloader is an ephemeral USB serial port; launching pnputil or
// WMIC for every sample takes long enough to miss its short lifetime.
// Use SetupAPI to enumerate *present* COM-port class devices in-process.
// Require BOTH the MediaTek PreLoader hardware ID and a valid COM port,
// not generic substrings like "MediaTek" that could match unrelated devices.
use std::ptr;
use windows_sys::core::GUID;
use windows_sys::Win32::Devices::DeviceAndDriverInstallation::{
    SetupDiGetClassDevsW, SetupDiEnumDeviceInfo, SetupDiGetDeviceRegistryPropertyW,
    SetupDiOpenDevRegKey, SetupDiDestroyDeviceInfoList, SP_DEVINFO_DATA,
    DIGCF_PRESENT, DIGCF_ALLCLASSES, SPDRP_CLASS, SPDRP_HARDWAREID,
    SPDRP_FRIENDLYNAME, SPDRP_DEVICEDESC,
    DICS_FLAG_GLOBAL, DIREG_DEV,
};
use windows_sys::Win32::System::Registry::{RegCloseKey, RegQueryValueExW, KEY_READ};

const PORTS_CLASS_GUID: GUID = GUID {
    data1: 0x4d36e978,
    data2: 0xe325,
    data3: 0x11ce,
    data4: [0xbf, 0xc1, 0x08, 0x00, 0x2b, 0xe1, 0x03, 0x18],
};
const PRELOADER_HARDWARE_ID: &str = "VID_0E8D&PID_2000";
// Historic MediaTek USB Port entries on this PC use PID_0003.
// This is BootROM, NOT a PreLoader port and must not authorize SPFT flashing.
const BROM_HARDWARE_ID: &str = "VID_0E8D&PID_0003";

#[derive(Default)]
pub(crate) struct PresentMtkPorts {
    pub preloader: Option<String>,
    pub brom: Option<String>,
}

pub(crate) fn matches_brom_hardware_id(hardware_ids: &str) -> bool {
    hardware_ids.to_ascii_uppercase().contains(BROM_HARDWARE_ID)
}

pub(crate) fn matches_preloader_hardware_id(hardware_ids: &str) -> bool {
    hardware_ids.to_ascii_uppercase().contains(PRELOADER_HARDWARE_ID)
}

pub(crate) fn matches_preloader_name(display_name: &str) -> bool {
    let name = display_name.trim().to_ascii_lowercase();
    [
        "mediatek preloader usb",
        "mediatek preloader",
        "preloader usb",
        "preloader",
        "mediatek usb",
        "mediatek",
    ].iter().any(|name_prefix| {
        name == *name_prefix
            || name.starts_with(&format!("{name_prefix} "))
            || name.starts_with(&format!("{name_prefix}("))
    })
}

fn decode_utf16(bytes: &[u8]) -> String {
    let words: Vec<u16> = bytes.chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0],pair[1]])).collect();
    String::from_utf16_lossy(&words)
}

fn extract_port_name(port: &str) -> Option<String> {
    let upper = port.trim_matches(|c: char| c.is_whitespace() || c == '\0').to_ascii_uppercase();
    let suffix = upper.strip_prefix("COM")?;
    if !suffix.is_empty() && suffix.bytes().all(|ch| ch.is_ascii_digit())
       && suffix.parse::<u16>().ok().is_some_and(|n| n > 0)
    {
        Some(upper)
    } else { None }
}

fn friendly_port_name(display_name: &str) -> Option<String> {
    let upper = display_name.to_ascii_uppercase();
    let pos = upper.rfind("(COM")?;
    let rest = upper.get(pos+1..)?.split(')').next()?;
    extract_port_name(rest)
}

fn device_property(
    set: isize,
    device: &SP_DEVINFO_DATA,
    property: u32,
) -> Option<String> {
    let mut buffer = vec![0u8; 2048];
    let mut datatype = 0u32;
    let mut length = 0u32;
    // SAFETY: The device set and SP_DEVINFO_DATA are valid for the current
    // SetupAPI enumeration, and buffer length is supplied correctly.
    let result = unsafe {
        SetupDiGetDeviceRegistryPropertyW(
            set, device, property, &mut datatype,
            buffer.as_mut_ptr(), buffer.len() as u32, &mut length,
        )
    };
    (result != 0).then(|| decode_utf16(&buffer[..(length as usize).min(buffer.len())]))
}

fn registry_port_name(set: isize, device: &SP_DEVINFO_DATA) -> Option<String> {
    // SAFETY: The device is part of the current present Ports class set.
    let key = unsafe { SetupDiOpenDevRegKey(set, device, DICS_FLAG_GLOBAL, 0, DIREG_DEV, KEY_READ) };
    if key as isize == -1 || key.is_null() { return None; }
    let value: Vec<u16> = "PortName".encode_utf16().chain(std::iter::once(0)).collect();
    let mut data = [0u8; 128];
    let mut kind = 0u32;
    let mut length = data.len() as u32;
    // SAFETY: Querying a read-only PortName REG_SZ on a valid registry handle.
    let status = unsafe {
        RegQueryValueExW(key, value.as_ptr(), ptr::null(), &mut kind,
            data.as_mut_ptr(), &mut length)
    };
    // SAFETY: We own the handle returned by SetupDiOpenDevRegKey.
    unsafe { RegCloseKey(key); }
    if status != 0 { return None; }
    extract_port_name(&decode_utf16(&data[..(length as usize).min(data.len())]))
}

pub(crate) fn scan_present_mtk_ports() -> PresentMtkPorts {
    // Enumerate only PRESENT serial ports. Historical non-present COM5/COM13
    // records must never be treated as an active flashing interface.
    let set = unsafe {
        SetupDiGetClassDevsW(&PORTS_CLASS_GUID, ptr::null(), ptr::null_mut(), DIGCF_PRESENT)
    };
    if set == -1 { return PresentMtkPorts::default(); }
    let mut result = PresentMtkPorts::default();
    let mut index = 0u32;
    loop {
        let mut dev: SP_DEVINFO_DATA = unsafe { std::mem::zeroed() };
        dev.cbSize = std::mem::size_of::<SP_DEVINFO_DATA>() as u32;
        if unsafe { SetupDiEnumDeviceInfo(set, index, &mut dev) } == 0 { break; }
        index += 1;
        let hwids = device_property(set, &dev, SPDRP_HARDWAREID).unwrap_or_default();
        let is_preloader = matches_preloader_hardware_id(&hwids);
        let is_brom = matches_brom_hardware_id(&hwids);
        if !is_preloader && !is_brom { continue; }

        let name = device_property(set, &dev, SPDRP_FRIENDLYNAME)
            .or_else(|| device_property(set, &dev, SPDRP_DEVICEDESC))
            .unwrap_or_default();
        let valid_name = if is_preloader {
            matches_preloader_name(&name)
        } else {
            let lower = name.to_ascii_lowercase();
            lower.contains("mediatek usb port") || lower.contains("mtk usb port")
        };
        if !valid_name { continue; }

        if let Some(port) = registry_port_name(set, &dev)
            .or_else(|| friendly_port_name(&name)) {
            // Windows may already include (COM5) in the friendly name.
            // Preserve a single port suffix in the diagnostic result.
            let label = name.trim().trim_end_matches('\0');
            let desc = if label.to_ascii_uppercase().contains(&format!("({port})")) {
                label.to_string()
            } else {
                format!("{label} ({port})")
            };
            if is_preloader {
                result.preloader = Some(desc);
                break;
            }
            // A BROM port cannot be used as a V6 SPFlashTool PreLoader port.
            result.brom = Some(desc);
        }
    }
    unsafe { SetupDiDestroyDeviceInfoList(set); }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn media_tek_brom_never_masquerades_as_preloader() {
        assert!(matches_brom_hardware_id("USB\\VID_0E8D&PID_0003"));
        assert!(!matches_preloader_hardware_id("USB\\VID_0E8D&PID_0003"));
        assert!(!matches_brom_hardware_id("USB\\VID_0E8D&PID_2000"));
        assert!(!matches_brom_hardware_id("USB\\VID_0E8D&PID_2001"));
    }
    #[test]
    fn expected_hardware_id_only() {
        assert!(matches_preloader_hardware_id("USB\\VID_0E8D&PID_2000"));
        assert!(!matches_preloader_hardware_id("USB\\VID_0E8D&PID_2001"));
        assert!(!matches_preloader_hardware_id("USB\\VID_0E8D&PID_0003"));
        assert!(!matches_preloader_hardware_id("USB\\VID_18D1&PID_2000"));
    }
    #[test]
    fn user_requested_names_with_strict_hardware_id_gate() {
        for name in [
            "MediaTek PreLoader USB", "MediaTek PreLoader", "PreLoader",
            "MediaTek", "MediaTek USB", "PreLoader USB",
            "MediaTek PreLoader USB VCOM (Android) (COM8)",
            "PreLoader USB VCOM Port (COM9)",
        ] { assert!(matches_preloader_name(name),"{name}"); }
        assert!(!matches_preloader_name("Qualcomm HS-USB QDLoader"));
        assert!(!matches_preloader_name("USB Serial Device"));
    }
    #[test]
    fn strict_com_port_names() {
        assert_eq!(extract_port_name("COM8").as_deref(), Some("COM8"));
        assert_eq!(extract_port_name("com123").as_deref(), Some("COM123"));
        assert_eq!(friendly_port_name("PreLoader USB VCOM (COM8)").as_deref(), Some("COM8"));
        for bad in ["COM", "COM0", "COM8-1", "COM8DEV", "LPT1"] {
            assert!(extract_port_name(bad).is_none(), "{bad}");
        }
    }
}

pub(crate) fn scan_present_lenovo_usb_model() -> Option<String> {
    // Windows Device Manager "Universal Serial Bus devices" is class USBDevice.
    // Exclude WPD/Portable Devices and Ports so a normal ADB/PreLoader device
    // cannot be mistaken for this alternate-USB warning condition.
    let set = unsafe {
        SetupDiGetClassDevsW(ptr::null(), ptr::null(), ptr::null_mut(),
            DIGCF_PRESENT | DIGCF_ALLCLASSES)
    };
    if set == -1 { return None; }
    let mut matched = None;
    let mut index = 0u32;
    loop {
        let mut dev: SP_DEVINFO_DATA = unsafe { std::mem::zeroed() };
        dev.cbSize = std::mem::size_of::<SP_DEVINFO_DATA>() as u32;
        if unsafe { SetupDiEnumDeviceInfo(set, index, &mut dev) } == 0 { break; }
        index += 1;
        let class = device_property(set, &dev, SPDRP_CLASS).unwrap_or_default();
        if !class.trim_matches('\0').trim().eq_ignore_ascii_case("USBDevice")
            && !class.trim_matches('\0').trim().eq_ignore_ascii_case("USB") {
            continue;
        }
        let name = device_property(set, &dev, SPDRP_FRIENDLYNAME)
            .or_else(|| device_property(set, &dev, SPDRP_DEVICEDESC))
            .unwrap_or_default();
        let upper = name.to_ascii_uppercase();
        if ["TB33FC", "TB335FC", "TB336FU", "TB365FC", "TB361FU", "TB375FC", "TB373FU"]
            .iter().any(|model| upper.contains(model))
        {
            matched = Some(name.trim_matches('\0').trim().to_string());
            break;
        }
    }
    unsafe { SetupDiDestroyDeviceInfoList(set); }
    matched
}
