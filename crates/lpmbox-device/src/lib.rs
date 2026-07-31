use adb_client::usb::{find_all_connected_adb_devices, ADBUSBDevice};
use adb_client::{ADBDeviceExt, RebootType};
use lpmbox_core::{app_paths, LpmError, PreloaderDetectResult, Result, RomRegion};
use nusb::transfer::{Buffer, Bulk, In, Out};
use nusb::Endpoint;
use rsa::pkcs8::{EncodePrivateKey, LineEnding};
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub const ADB_UNAUTHORIZED_GUIDE: &str =
    "[안내] PC(노트북)와 연결한 태블릿에 잠금 해제 → 메세지 창 왼쪽 중간 체크 박스 체크 → 오른쪽 하단 Allow(허용)를 터치해주세요.";

const PRELOADER_TOKENS: &[&str] = &[
    "MediaTek PreLoader USB VCOM (Android)",
    "MediaTek PreLoader USB VCOM",
    "MediaTek PreLoader USB",
    "MediaTek PreLoader",
    "MediaTek",
    "PreLoader",
];

const PRELOADER_POLL_INTERVAL_MS: u64 = 500;

const ADB_SERVER_ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037);
const ADB_SERVER_PROBE_TIMEOUT: Duration = Duration::from_millis(150);
const ADB_WAIT_TIMEOUT: Duration = Duration::from_secs(120);
const SLOT_A_ADB_WAIT_TIMEOUT: Duration = Duration::from_secs(30);
const ADB_POLL_INTERVAL: Duration = Duration::from_secs(1);
const ADB_CONNECT_RETRY_ATTEMPTS: usize = 3;
const ADB_CONNECT_RETRY_BACKOFF: Duration = Duration::from_millis(150);
const SPINNER_FRAMES: [&str; 4] = ["│", "╱", "━", "╲"];
const INTERNAL_SPINNER_PREFIX: &str = "\u{001e}LPMBOX_SPINNER\u{001f}";

const FASTBOOT_USB_CLASS: u8 = 0xFF;
const FASTBOOT_USB_SUBCLASS: u8 = 0x42;
const FASTBOOT_USB_PROTOCOL: u8 = 0x03;

#[derive(Debug, Clone)]
pub struct AdbDeviceProbe {
    pub android_version: Option<String>,
    pub android_major: Option<u32>,
    pub platform: String,
    pub model: String,
    pub region: RomRegion,
    pub current_slot: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DashboardDeviceInfo {
    pub connected: bool,
    pub country_code: String,
    pub locale: String,
    pub serial_number: String,
    pub original_rom: String,
    pub model_name: String,
    pub product_device: String,
    pub platform: String,
    pub installed_rom: String,
    pub firmware_version: String,
    pub widevine_level: String,
    pub android_version: String,
    pub slot_suffix: String,
    pub hardware_info: String,
    pub battery_level: Option<u8>,
    pub uptime_display: String,
    pub ap_chipset: String,
    pub ota_status: String,
}

impl Default for DashboardDeviceInfo {
    fn default() -> Self {
        Self {
            connected: false,
            country_code: "알 수 없음".to_string(),
            locale: "알 수 없음".to_string(),
            serial_number: "알 수 없음".to_string(),
            original_rom: "알 수 없음".to_string(),
            model_name: "알 수 없음".to_string(),
            product_device: "알 수 없음".to_string(),
            platform: "알 수 없음".to_string(),
            installed_rom: "알 수 없음".to_string(),
            firmware_version: "알 수 없음".to_string(),
            widevine_level: "알 수 없음".to_string(),
            android_version: "알 수 없음".to_string(),
            slot_suffix: "알 수 없음".to_string(),
            hardware_info: "알 수 없음".to_string(),
            battery_level: None,
            uptime_display: "알 수 없음".to_string(),
            ap_chipset: "알 수 없음".to_string(),
            ota_status: "알 수 없음".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
enum AdbUsbState {
    Ready,
    Unauthorized,
    ServerBlocking,
    NoDevice,
    Error(String),
}

pub fn terminate_adb_fastboot_processes() {
    if cfg!(windows) {
        let _ = hidden_command_output("taskkill", &["/F", "/IM", "adb.exe"]);
        let _ = hidden_command_output("taskkill", &["/F", "/IM", "fastboot.exe"]);
    }
}

pub fn probe_adb_device_for_convert_wipe<F>(mut on_log: F) -> Result<AdbDeviceProbe>
    where
        F: FnMut(String),
    {
        prepare_adb_usb_environment();

    on_log(spinner_log(
        "adb_detect_start",
        "[ADB] 기기 감지 │",
    ));

    on_log(ADB_UNAUTHORIZED_GUIDE.to_string());

    wait_for_adb_device_ready(&mut on_log)?;

    on_log(spinner_log(
        "adb_detect_start",
        "[ADB] 기기 감지 완료",
    ));

    let android_version = empty_to_none(adb_shell_checked("getprop ro.build.version.release")?);
    let android_major = android_version
        .as_deref()
        .and_then(|value| value.split('.').next())
        .and_then(|value| value.parse::<u32>().ok());

    if let Some(major) = android_major {
        on_log(format!("[ADB] Android 버전: {major}"));

        if major <= 14 {
            return Err(LpmError::Adb(format!(
                "Android {major} 기기는 1번 설치 흐름에서 차단됩니다. Android 15 이상에서만 진행합니다."
            )));
        }
    } else {
        on_log("[ADB] Android 버전 감지 실패".to_string());
    }

    let platform = adb_shell_checked("getprop ro.vendor.mediatek.platform")?
        .trim()
        .to_string();

    if platform.is_empty() || !platform.to_uppercase().starts_with("MT") {
        return Err(LpmError::Adb(format!(
            "MediaTek platform 감지 실패 또는 MTK 기기가 아닙니다: {platform}"
        )));
    }

    on_log(format!("[ADB] platform: {platform}"));

    let mut model = adb_shell_checked("getprop ro.product.model")?
        .trim()
        .to_string();

    if model.is_empty() {
        model = adb_shell_checked("getprop ro.vendor.config.lgsi.hw.version")
            .unwrap_or_default()
            .trim()
            .to_string();
    }

    if model.is_empty() {
        model = "UNKNOWN".to_string();
    }

    on_log(format!("[ADB] model: {model}"));

    let region_raw = adb_shell_checked("getprop ro.config.zui.region")?;
    let region = normalize_region(&region_raw);

    on_log(format!("[ADB] region: {}", region_label(region)));

    let current_slot = adb_get_slot_suffix_checked().ok().flatten();

    Ok(AdbDeviceProbe {
        android_version,
        android_major,
        platform,
        model,
        region,
        current_slot,
    })
}

pub fn run_current_slot_a_stage<F>(on_log: F) -> Result<()>
where
    F: FnMut(String),
{
    run_current_slot_a_stage_with_adb_timeout(SLOT_A_ADB_WAIT_TIMEOUT, on_log)
}

pub fn run_current_slot_a_stage_with_adb_timeout<F>(
    adb_timeout: Duration,
    mut on_log: F,
) -> Result<()>
where
    F: FnMut(String),
{
    on_log(spinner_log(
        "adb_slot_detect",
        "[ADB] 기기를 감지하고 있습니다... │",
    ));

    wait_for_adb_device_ready_until(&mut on_log, adb_timeout)?;

    on_log(spinner_log(
        "adb_slot_detect",
        "[ADB] 기기 감지 완료",
    ));

    on_log(spinner_log(
        "adb_slot_set",
        "[ADB] ADB 명령어로 Slot 설정 중... │",
    ));

    let mut bootctl_ok = false;

    for attempt in 1..=10 {
        match adb_shell_checked("bootctl set-active-boot-slot 0") {
            Ok(_) => {
                bootctl_ok = true;
                break;
            }
            Err(err) => {
                if attempt == 1 || attempt == 10 {
                    on_log(format!("[ADB] Slot 설정 재시도 {attempt}/10 실패: {err}"));
                }

                if attempt < 10 {
                    thread::sleep(Duration::from_secs(2));
                }
            }
        }
    }

    if bootctl_ok {
        on_log(spinner_log(
            "adb_slot_set",
            "[ADB] ADB 명령어로 Slot 설정 완료",
        ));
    } else {
        on_log(spinner_log(
            "adb_slot_set",
            "[ADB] ADB 명령어 Slot 설정 실패, Fastboot Slot 설정으로 계속 진행합니다.",
        ));
    }

    on_log("[ADB] bootloader 모드 설정".to_string());
    adb_reboot_checked("bootloader")?;

    thread::sleep(Duration::from_secs(2));

    on_log(spinner_log(
        "fastboot_detect",
        "[Fastboot] 기기 감지중... │",
    ));
    wait_for_fastboot_device(Duration::from_secs(60))?;
    on_log(spinner_log(
        "fastboot_detect",
        "[Fastboot] 기기 감지 완료",
    ));

    let current_slot = fastboot_get_current_slot().ok().flatten();

    if let Some(slot) = &current_slot {
        on_log(format!("[Fastboot] current-slot: {slot}"));
    } else {
        on_log("[Fastboot] current-slot 감지 실패, A로 설정합니다.".to_string());
    }

    run_fastboot_set_active_a_sequence(&mut on_log)?;

    on_log(spinner_log(
        "fastboot_reboot_bootloader",
        "[Fastboot] bootloader 재진입 후 Fastboot 재감지 중... │",
    ));

    let _ = fastboot_reboot_bootloader();

    thread::sleep(Duration::from_secs(2));

    wait_for_fastboot_device(Duration::from_secs(60))?;

    on_log(spinner_log(
        "fastboot_reboot_bootloader",
        "[Fastboot] bootloader 재진입 후 Fastboot 재감지 완료",
    ));

    run_fastboot_set_active_a_sequence(&mut on_log)?;

    let final_slot = fastboot_get_current_slot()?.unwrap_or_else(|| "UNKNOWN".to_string());

    on_log(format!("[Fastboot] current-slot 재확인: {final_slot}"));

    if final_slot != "_a" {
        let _ = fastboot_reboot();
        return Err(LpmError::Fastboot(format!(
            "slot A 설정 실패 / 최종 slot: {final_slot}"
        )));
    }

    on_log("[Fastboot] 확인 완료".to_string());
    on_log(spinner_log(
        "fastboot_stabilize",
        "[Fastboot] 안정화를 위해 5초 대기합니다... │",
    ));

    thread::sleep(Duration::from_secs(5));

    on_log(spinner_log(
        "fastboot_stabilize",
        "[Fastboot] 안정화 대기 완료",
    ));

    Ok(())
}

pub fn trigger_rom_install_reboot_commands<F>(mut on_log: F)
where
    F: FnMut(String),
{
    on_log(spinner_log(
        "reboot_device",
        "[ADB] adb reboot 1회 실행 중... │",
    ));

    let adb_reboot_ok = adb_reboot_checked("system").is_ok();

    on_log(spinner_log(
        "reboot_device",
        if adb_reboot_ok {
            "[ADB] adb reboot 1회 실행 완료"
        } else {
            "[ADB] adb reboot 1회 실행 실패, Fastboot 명령을 계속 시도합니다."
        },
    ));

    on_log(spinner_log(
        "reboot_device",
        "[Fastboot] fastboot reboot 1회 실행 중... │",
    ));

    let fastboot_reboot_ok = fastboot_reboot().is_ok();

    on_log(spinner_log(
        "reboot_device",
        if fastboot_reboot_ok {
            "[Fastboot] fastboot reboot 1회 실행 완료"
        } else {
            "[Fastboot] fastboot reboot 1회 실행 실패, PreLoader 감지를 계속합니다."
        },
    ));
}

pub fn disable_ota_updates<F>(mut on_log: F) -> Result<()>
where
    F: FnMut(String),
{
    prepare_adb_usb_environment();

    on_log(spinner_log("ota_adb_detect", "[OTA] USB ADB 기기 감지 중... │"));
    on_log(ADB_UNAUTHORIZED_GUIDE.to_string());
    wait_for_adb_device_ready(&mut on_log)?;
    on_log(spinner_log("ota_adb_detect", "[OTA] USB ADB 기기 감지 완료"));

    on_log("[OTA] 업데이트 자동 동작을 비활성화합니다.".to_string());

    let settings = [
        ("global", "ota_disable_automatic_update", "1"),
        ("global", "setup_wizard_privacy_auto_update", "0"),
        ("global", "setup_wizard_privacy_ota_key", "0"),
        ("system", "ota_network_permission", "0"),
        ("secure", "lenovo_ota_new_version_found", "0"),
    ];

    for (scope, key, value) in settings {
        let command = format!("settings put {scope} {key} {value}");

        match adb_shell_checked(&command) {
            Ok(_) => on_log(format!("[OTA] 설정 적용: {scope} {key}={value}")),
            Err(err) => on_log(format!("[OTA] 설정 적용 실패: {scope} {key} / {err}")),
        }
    }

    on_log("[OTA] 업데이트 관련 패키지를 비활성화합니다.".to_string());

    for package in ["com.lenovo.ota", "com.tblenovo.lenovowhatsnew", "com.lenovo.tbengine"] {
        let uninstall_command = format!("pm uninstall -k --user 0 {package}");

        match adb_shell_checked(&uninstall_command) {
            Ok(output) => {
                if output.trim().is_empty() {
                    on_log(format!("[OTA] 패키지 비활성화: {package}"));
                } else {
                    on_log(format!("[OTA] 패키지 비활성화: {package} / {output}"));
                }
            }
            Err(err) => on_log(format!("[OTA] 패키지 비활성화 확인 필요: {package} / {err}")),
        }
    }

    let _ = kill_adb_server_direct();
    Ok(())
}

pub fn enable_ota_updates<F>(mut on_log: F) -> Result<()>
where
    F: FnMut(String),
{
    prepare_adb_usb_environment();

    on_log(spinner_log("ota_adb_detect", "[OTA] USB ADB 기기 감지 중... │"));
    on_log(ADB_UNAUTHORIZED_GUIDE.to_string());
    wait_for_adb_device_ready(&mut on_log)?;
    on_log(spinner_log("ota_adb_detect", "[OTA] USB ADB 기기 감지 완료"));

    let region_raw = adb_shell_checked("getprop ro.config.zui.region").unwrap_or_default();
    let region = region_raw.trim().to_ascii_uppercase();

    if !region.is_empty() {
        on_log(format!("[OTA] 기기에 설치된 ROM 타입: {region}"));
    }

    if region == "ROW" {
        on_log("[OTA] ROW(글로벌롬)은 OTA 활성화 작업을 건너뜁니다.".to_string());
        let _ = kill_adb_server_direct();
        return Ok(());
    }

    on_log("[OTA] 업데이트 관련 패키지를 복원합니다.".to_string());

    let packages = [
        "com.zui.homesettings",
        "com.lenovo.tbengine",
        "com.lenovo.ue.device",
        "com.lenovo.ota",
        "com.zui.safecenter",
    ];

    for package in packages {
        let install_command = format!("cmd package install-existing --user 0 {package}");

        match adb_shell_checked(&install_command) {
            Ok(output) => {
                if output.trim().is_empty() {
                    on_log(format!("[OTA] 패키지 복원: {package}"));
                } else {
                    on_log(format!("[OTA] 패키지 복원: {package} / {output}"));
                }
            }
            Err(err) => on_log(format!("[OTA] 패키지 복원 확인 필요: {package} / {err}")),
        }
    }

    for package in packages {
        let enable_command = format!("pm enable --user 0 {package}");

        match adb_shell_checked(&enable_command) {
            Ok(output) => {
                if output.trim().is_empty() {
                    on_log(format!("[OTA] 패키지 활성화: {package}"));
                } else {
                    on_log(format!("[OTA] 패키지 활성화: {package} / {output}"));
                }
            }
            Err(err) => on_log(format!("[OTA] 패키지 활성화 확인 필요: {package} / {err}")),
        }
    }

    on_log("[OTA] 설정 → 시스템 업데이트에서 OTA 상태를 확인해주세요.".to_string());

    let _ = kill_adb_server_direct();
    Ok(())
}

fn prepare_adb_usb_environment() {
    configure_stable_adb_vendor_keys();
    terminate_adb_fastboot_processes();

    if adb_server_running() {
        let _ = kill_adb_server_direct();
    }
}

fn configure_stable_adb_vendor_keys() {
    let Ok(key_path) = AdbManager::ensure_key_path() else {
        return;
    };

    let Some(key_dir) = key_path.parent() else {
        return;
    };

    unsafe {
        std::env::set_var("ADB_VENDOR_KEYS", key_dir);
    }
}

fn adb_usb_candidate_exists() -> Result<bool> {
    let devices = find_all_connected_adb_devices()
        .map_err(|err| LpmError::Adb(format!("ADB USB 기기 검색 실패: {err}")))?;

    Ok(!devices.is_empty())
}

fn wait_for_adb_device_ready<F>(on_log: &mut F) -> Result<()>
where
    F: FnMut(String),
{
    if !adb_usb_candidate_exists()? {
        return Err(LpmError::Adb(
            "NO_USB_ADB_DEVICE: 태블릿 확인에 실패했습니다, 올바른 데이터 케이블을 사용해주세요."
                .to_string(),
        ));
    }

    wait_for_adb_device_ready_until(on_log, ADB_WAIT_TIMEOUT)
}

fn wait_for_adb_device_ready_until<F>(on_log: &mut F, timeout: Duration) -> Result<()>
where
    F: FnMut(String),
{
    let deadline = Instant::now() + timeout;
    let mut spinner_index = 0usize;
    let mut current_state = "ADB USB 기기 감지 대기 중입니다.".to_string();

    loop {
        let frame = SPINNER_FRAMES[spinner_index % SPINNER_FRAMES.len()];
        spinner_index = spinner_index.wrapping_add(1);

        on_log(spinner_log(
            "adb_usb",
            &format!("[ADB] USB ADB 권한 요청/감지 중... {frame}"),
        ));

        if adb_usb_candidate_exists().unwrap_or(false) {
            current_state = match probe_adb_usb_state_once() {
                AdbUsbState::Ready => {
                    on_log(spinner_log("adb_usb", "[ADB] USB ADB 연결 완료"));
                    return Ok(());
                }

                AdbUsbState::NoDevice => "ADB USB 기기 감지 대기 중입니다.".to_string(),

                AdbUsbState::Unauthorized => "ADB unauthorized 상태입니다.".to_string(),

                AdbUsbState::ServerBlocking => {
                    let _ = kill_adb_server_direct();
                    "외부 adb server가 USB ADB 인터페이스를 점유 중입니다.".to_string()
                }

                AdbUsbState::Error(err) => err,
            };
        }

        if Instant::now() >= deadline {
            if current_state.contains("감지 대기") {
                return Err(LpmError::Adb(
                    "NO_USB_ADB_DEVICE: 태블릿 확인에 실패했습니다, 올바른 데이터 케이블을 사용해주세요."
                        .to_string(),
                ));
            }

            return Err(LpmError::Adb(format!(
                "ADB_UNAUTHORIZED_RETRY: ADB 기기 감지 시간 초과. 마지막 상태: {current_state}"
            )));
        }

        thread::sleep(ADB_POLL_INTERVAL);
    }
}

fn probe_adb_usb_state_once() -> AdbUsbState {
    let mut adb = AdbManager::new();

    match adb.check_device_state() {
        Ok(state) => state,
        Err(err) => AdbUsbState::Error(err.to_string()),
    }
}

fn spinner_log(key: &str, message: &str) -> String {
    format!("{INTERNAL_SPINNER_PREFIX}{key}|{message}")
}

fn adb_shell_checked(command: &str) -> Result<String> {
    let mut adb = AdbManager::new();

    match adb.check_device_state()? {
        AdbUsbState::Ready => adb.shell(command),
        AdbUsbState::Unauthorized => Err(LpmError::Adb(ADB_UNAUTHORIZED_GUIDE.to_string())),
        AdbUsbState::ServerBlocking => Err(LpmError::Adb(
            "외부 adb server가 USB ADB 인터페이스를 점유 중입니다.".to_string(),
        )),
        AdbUsbState::NoDevice => Err(LpmError::Adb(
            "USB ADB 기기를 찾지 못했습니다.".to_string(),
        )),
        AdbUsbState::Error(err) => Err(LpmError::Adb(err)),
    }
}

fn adb_get_slot_suffix_checked() -> Result<Option<String>> {
    let value = adb_shell_checked("getprop ro.boot.slot_suffix")?;

    match value.trim() {
        "_a" | "a" => Ok(Some("_a".to_string())),
        "_b" | "b" => Ok(Some("_b".to_string())),
        _ => Ok(None),
    }
}

fn adb_reboot_checked(target: &str) -> Result<()> {
    let mut adb = AdbManager::new();

    match adb.check_device_state()? {
        AdbUsbState::Ready => adb.reboot(target),
        AdbUsbState::Unauthorized => Err(LpmError::Adb(ADB_UNAUTHORIZED_GUIDE.to_string())),
        AdbUsbState::ServerBlocking => Err(LpmError::Adb(
            "외부 adb server가 USB ADB 인터페이스를 점유 중입니다.".to_string(),
        )),
        AdbUsbState::NoDevice => Err(LpmError::Adb(
            "USB ADB 기기를 찾지 못했습니다.".to_string(),
        )),
        AdbUsbState::Error(err) => Err(LpmError::Adb(err)),
    }
}

struct AdbManager {
    device: Option<ADBUSBDevice>,
    serial: Option<String>,
}

impl AdbManager {
    fn new() -> Self {
        Self {
            device: None,
            serial: None,
        }
    }

    fn ensure_key_path() -> Result<PathBuf> {
        let path = app_paths::adb_key_path();

        if path.exists() {
            return Ok(path);
        }

        let parent = path.parent().ok_or_else(|| {
            LpmError::Adb(format!("ADB key parent 경로가 없습니다: {}", path.display()))
        })?;

        std::fs::create_dir_all(parent)?;

        let private_key = rsa::RsaPrivateKey::new(&mut rsa::rand_core::OsRng, 2048)
            .map_err(|err| LpmError::Adb(format!("ADB RSA key 생성 실패: {err}")))?;

        let pem = private_key
            .to_pkcs8_pem(LineEnding::LF)
            .map_err(|err| LpmError::Adb(format!("ADB RSA key PEM 변환 실패: {err}")))?;

        std::fs::write(&path, pem.as_bytes())?;

        Ok(path)
    }

    fn connect_device(&mut self) -> Result<&mut ADBUSBDevice> {
        if self.device.is_none() {
            let key_path = Self::ensure_key_path()?;
            let mut last_error: Option<String> = None;

            for attempt in 0..ADB_CONNECT_RETRY_ATTEMPTS {
                match ADBUSBDevice::autodetect_with_custom_private_key(key_path.clone()) {
                    Ok(mut device) => {
                        let mut stdout = Vec::new();

                        let _ = device.shell_command(
                            &"getprop ro.serialno",
                            Some(&mut stdout as &mut dyn Write),
                            None,
                        );

                        let serial = String::from_utf8_lossy(&stdout).trim().to_string();

                        if !serial.is_empty() {
                            self.serial = Some(serial);
                        }

                        self.device = Some(device);

                        return Ok(self.device.as_mut().expect("ADB device just stored"));
                    }

                    Err(err) => {
                        last_error = Some(err.to_string());

                        if attempt + 1 < ADB_CONNECT_RETRY_ATTEMPTS {
                            thread::sleep(ADB_CONNECT_RETRY_BACKOFF);
                        }
                    }
                }
            }

            return Err(LpmError::Adb(
                last_error.unwrap_or_else(|| "ADB USB 직접 연결 실패".to_string()),
            ));
        }

        Ok(self.device.as_mut().expect("ADB device cached"))
    }

    fn drop_device(&mut self) {
        self.device = None;
    }

    fn check_device_state(&mut self) -> Result<AdbUsbState> {
        let devices = find_all_connected_adb_devices()
            .map_err(|err| LpmError::Adb(format!("ADB USB 기기 검색 실패: {err}")))?;

        if devices.is_empty() {
            self.drop_device();
            return Ok(AdbUsbState::NoDevice);
        }

        match self.connect_device() {
            Ok(_) => Ok(AdbUsbState::Ready),

            Err(_) => {
                self.drop_device();

                if adb_server_running() {
                    Ok(AdbUsbState::ServerBlocking)
                } else {
                    Ok(AdbUsbState::Unauthorized)
                }
            }
        }
    }

    fn shell(&mut self, command: &str) -> Result<String> {
        let device = self.connect_device()?;

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let result = device.shell_command(
            &command,
            Some(&mut stdout as &mut dyn Write),
            Some(&mut stderr as &mut dyn Write),
        );

        match result {
            Ok(_) => Ok(String::from_utf8_lossy(&stdout).trim().to_string()),
            Err(err) => {
                self.drop_device();

                let stderr_text = String::from_utf8_lossy(&stderr).trim().to_string();

                if stderr_text.is_empty() {
                    Err(LpmError::Adb(format!("ADB shell 실패 `{command}`: {err}")))
                } else {
                    Err(LpmError::Adb(format!(
                        "ADB shell 실패 `{command}`: {err} / stderr={stderr_text}"
                    )))
                }
            }
        }
    }


    fn reboot(&mut self, target: &str) -> Result<()> {
        let reboot_type = match target {
            "bootloader" => RebootType::Bootloader,
            "recovery" => RebootType::Recovery,
            "sideload" => RebootType::Sideload,
            _ => RebootType::System,
        };

        let device = self.connect_device()?;

        let result = device
            .reboot(reboot_type)
            .map_err(|err| LpmError::Adb(format!("ADB reboot 실패: {err}")));

        self.drop_device();

        match result {
            Ok(_) => Ok(()),
            Err(err) => {
                let text = err.to_string().to_ascii_lowercase();

                if is_adbd_dropped_after_reboot(&text) {
                    Ok(())
                } else {
                    Err(err)
                }
            }
        }
    }
}

fn is_adbd_dropped_after_reboot(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();

    lower.contains("pipe")
        || lower.contains("broken pipe")
        || lower.contains("no device")
        || lower.contains("device disconnected")
        || lower.contains("unexpected eof")
        || lower.contains("end of file")
        || lower.contains("input/output error")
        || lower.contains("i/o error")
        || lower.contains("closed")
        || lower.contains("reset")
}

pub fn adb_server_running() -> bool {
    TcpStream::connect_timeout(&ADB_SERVER_ADDR.into(), ADB_SERVER_PROBE_TIMEOUT).is_ok()
}

pub fn kill_adb_server_direct() -> Result<()> {
    let mut stream = TcpStream::connect_timeout(&ADB_SERVER_ADDR.into(), Duration::from_secs(2))
        .map_err(|err| LpmError::Adb(format!("adb server 연결 실패: {err}")))?;

    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|err| LpmError::Adb(format!("adb server write timeout 설정 실패: {err}")))?;

    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|err| LpmError::Adb(format!("adb server read timeout 설정 실패: {err}")))?;

    let payload = b"host:kill";
    let header = format!("{:04x}", payload.len());

    stream
        .write_all(header.as_bytes())
        .and_then(|_| stream.write_all(payload))
        .map_err(|err| LpmError::Adb(format!("adb server kill 요청 실패: {err}")))?;

    let mut reply = [0u8; 4];

    match stream.read_exact(&mut reply) {
        Ok(()) => match &reply {
            b"OKAY" => Ok(()),
            b"FAIL" => Err(LpmError::Adb(
                "adb server가 host:kill 요청을 거부했습니다.".to_string(),
            )),
            other => Err(LpmError::Adb(format!(
                "adb server host:kill 응답이 예상과 다릅니다: {other:?}"
            ))),
        },

        Err(err)
            if matches!(
                err.kind(),
                std::io::ErrorKind::UnexpectedEof | std::io::ErrorKind::ConnectionReset
            ) =>
        {
            Ok(())
        }

        Err(err) => Err(LpmError::Adb(format!(
            "adb server host:kill 응답 읽기 실패: {err}"
        ))),
    }
}

struct NativeFastbootDevice {
    _interface: nusb::Interface,
    ep_in: Endpoint<Bulk, In>,
    ep_out: Endpoint<Bulk, Out>,
}

impl NativeFastbootDevice {
    fn open() -> Result<Self> {
        use nusb::MaybeFuture;

        let devices = nusb::list_devices()
            .wait()
            .map_err(|err| LpmError::Fastboot(format!("USB 기기 목록 읽기 실패: {err}")))?
            .collect::<Vec<_>>();

        for dev_info in devices {
            let Ok(device) = dev_info.open().wait() else {
                continue;
            };

            for config in device.configurations() {
                for iface in config.interfaces() {
                    for alt in iface.alt_settings() {
                        if alt.class() != FASTBOOT_USB_CLASS
                            || alt.subclass() != FASTBOOT_USB_SUBCLASS
                            || alt.protocol() != FASTBOOT_USB_PROTOCOL
                        {
                            continue;
                        }

                        let mut in_addr: u8 = 0;
                        let mut out_addr: u8 = 0;

                        for ep in alt.endpoints() {
                            match ep.direction() {
                                nusb::transfer::Direction::In => in_addr = ep.address(),
                                nusb::transfer::Direction::Out => out_addr = ep.address(),
                            }
                        }

                        if in_addr == 0 || out_addr == 0 {
                            continue;
                        }

                        let interface = device.claim_interface(iface.interface_number()).wait()
                            .map_err(|err| {
                                LpmError::Fastboot(format!("Fastboot USB 인터페이스 열기 실패: {err}"))
                            })?;

                        let ep_in = interface.endpoint::<Bulk, In>(in_addr).map_err(|err| {
                            LpmError::Fastboot(format!("Fastboot USB IN endpoint 열기 실패: {err}"))
                        })?;

                        let ep_out = interface.endpoint::<Bulk, Out>(out_addr).map_err(|err| {
                            LpmError::Fastboot(format!("Fastboot USB OUT endpoint 열기 실패: {err}"))
                        })?;

                        return Ok(Self {
                            _interface: interface,
                            ep_in,
                            ep_out,
                        });
                    }
                }
            }
        }

        Err(LpmError::Fastboot("Fastboot 기기 감지 실패".to_string()))
    }

    fn bulk_write(&mut self, data: Vec<u8>) -> Result<()> {
        self.ep_out.submit(Buffer::from(data));
        let completion = pollster::block_on(self.ep_out.next_complete());
        completion
            .status
            .map_err(|err| LpmError::Fastboot(format!("Fastboot USB write 실패: {err}")))?;
        Ok(())
    }

    fn bulk_read(&mut self) -> Result<Vec<u8>> {
        self.ep_in.submit(Buffer::new(4096));
        let completion = pollster::block_on(self.ep_in.next_complete());
        completion
            .status
            .map_err(|err| LpmError::Fastboot(format!("Fastboot USB read 실패: {err}")))?;

        let len = completion.actual_len;
        let mut out = completion.buffer.into_vec();
        out.truncate(len);
        Ok(out)
    }

    fn command(&mut self, command: &str) -> Result<String> {
        self.bulk_write(command.as_bytes().to_vec())?;

        loop {
            let data = self.bulk_read()?;

            if data.len() < 4 {
                return Err(LpmError::Fastboot("Fastboot 응답이 너무 짧습니다.".to_string()));
            }

            let status = std::str::from_utf8(&data[..4]).unwrap_or("");
            let payload = std::str::from_utf8(&data[4..]).unwrap_or("").trim();

            match status {
                "OKAY" => return Ok(payload.to_string()),
                "FAIL" => {
                    return Err(LpmError::Fastboot(format!(
                        "Fastboot 명령 실패 `{command}`: {payload}"
                    )));
                }
                "INFO" => continue,
                "DATA" => return Ok(payload.to_string()),
                _ => {
                    return Err(LpmError::Fastboot(format!(
                        "Fastboot 알 수 없는 응답 `{status}` / 명령 `{command}`"
                    )));
                }
            }
        }
    }

    fn getvar(&mut self, variable: &str) -> Result<String> {
        self.command(&format!("getvar:{variable}"))
    }

    fn get_slot_suffix(&mut self) -> Result<Option<String>> {
        match self.getvar("current-slot") {
            Ok(slot) => match slot.trim() {
                "a" | "_a" => Ok(Some("_a".to_string())),
                "b" | "_b" => Ok(Some("_b".to_string())),
                _ => Ok(None),
            },
            Err(_) => Ok(None),
        }
    }

    fn set_active_a(&mut self) -> Result<()> {
    self.command("set_active:a").map(|_| ())
    }

    fn set_active_a_compat(&mut self) -> Result<()> {
    self.command("set_active:a").map(|_| ())
    }

    fn set_active_a_legacy(&mut self) -> Result<()> {
    self.command("set_active:a").map(|_| ())
    }

    fn reboot(&mut self) -> Result<()> {
        self.command("reboot").map(|_| ())
    }

    fn reboot_bootloader(&mut self) -> Result<()> {
        self.command("reboot-bootloader").map(|_| ())
    }
}

fn hidden_command_output(program: &str, args: &[&str]) -> Option<Output> {
    let mut command = Command::new(program);
    command.args(args);

    #[cfg(windows)]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command.output().ok()
}

fn wait_for_fastboot_device(timeout: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;

    loop {
        if fastboot_has_device() {
            thread::sleep(Duration::from_secs(2));

            if fastboot_has_device() {
                return Ok(());
            }

            thread::sleep(Duration::from_secs(2));
        } else {
            thread::sleep(Duration::from_secs(2));
        }

        if Instant::now() >= deadline {
            return Err(LpmError::Fastboot("Fastboot 기기 감지 실패".to_string()));
        }
    }
}

fn fastboot_has_device() -> bool {
    NativeFastbootDevice::open().is_ok()
}

fn fastboot_get_current_slot() -> Result<Option<String>> {
    let mut device = NativeFastbootDevice::open()?;
    device.get_slot_suffix()
}

fn fastboot_reboot() -> Result<()> {
    let mut device = NativeFastbootDevice::open()?;
    device.reboot()
}

fn fastboot_reboot_bootloader() -> Result<()> {
    let mut device = NativeFastbootDevice::open()?;
    device.reboot_bootloader()
}

fn run_fastboot_set_active_a_sequence<F>(on_log: &mut F) -> Result<()>
where
    F: FnMut(String),
{
    on_log(spinner_log(
        "fastboot_slot_a",
        "[Fastboot] slot A 설정 중... │",
    ));

    NativeFastbootDevice::open().and_then(|mut device| device.set_active_a())?;
    thread::sleep(Duration::from_secs(1));

    NativeFastbootDevice::open().and_then(|mut device| device.set_active_a_compat())?;
    thread::sleep(Duration::from_secs(1));

    NativeFastbootDevice::open().and_then(|mut device| device.set_active_a_legacy())?;
    thread::sleep(Duration::from_secs(1));

    on_log(spinner_log(
        "fastboot_slot_a",
        "[Fastboot] slot A 설정 완료",
    ));

    Ok(())
}

fn normalize_region(value: &str) -> RomRegion {
    match value.trim().to_uppercase().as_str() {
        "PRC" | "CN" => RomRegion::Prc,
        "ROW" => RomRegion::Row,
        _ => RomRegion::Unknown,
    }
}

fn region_label(region: RomRegion) -> &'static str {
    match region {
        RomRegion::Prc => "PRC(중국 내수롬)",
        RomRegion::Row => "ROW(글로벌롬)",
        RomRegion::Unknown => "Unknown",
    }
}

fn empty_to_none(value: String) -> Option<String> {
    let value = value.trim().to_string();

    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

pub fn read_dashboard_device_info() -> Result<DashboardDeviceInfo> {
    let mut adb = AdbManager::new();

    ensure_dashboard_adb_ready(&mut adb)?;

    let country_code = dashboard_getprop(&mut adb, "gsm.product.countrycode");
    let locale = dashboard_getprop(&mut adb, "persist.sys.locale");

    let mut serial_number = dashboard_getprop(&mut adb, "ro.odm.lenovo.gsn");
    if serial_number.is_empty() {
        serial_number = dashboard_getprop(&mut adb, "ro.boot.sn");
    }
    if serial_number.is_empty() {
        serial_number = dashboard_getprop(&mut adb, "ro.boot.gsn");
    }
    if serial_number.is_empty() {
        serial_number = dashboard_getprop(&mut adb, "ro.serialno");
    }

    let original_rom_raw = dashboard_getprop(&mut adb, "ro.boot.region");
    let product_display = dashboard_getprop(&mut adb, "ro.product.display");
    let product_device = dashboard_getprop(&mut adb, "ro.product.device");
    let platform = dashboard_getprop(&mut adb, "ro.vendor.mediatek.platform");
    let installed_rom_raw = dashboard_getprop(&mut adb, "ro.config.zui.region");
    let build_display_id = dashboard_getprop(&mut adb, "ro.build.display.id");

    let widevine_prop = dashboard_getprop(&mut adb, "sys.lenovo.widevine_security_level");
    let android_version = dashboard_getprop(&mut adb, "ro.build.version.release");
    let slot_suffix = dashboard_getprop(&mut adb, "ro.boot.slot_suffix");
    let hw_board_id = dashboard_getprop(&mut adb, "ro.boot.hwboardid");

    let ap_chipset_raw = dashboard_getprop(&mut adb, "ro.vendor.config.lgsi.cpuinfo");

    let drm_widevine_dump = adb
        .shell("dumpsys android.hardware.drm.IDrmFactory/widevine")
        .unwrap_or_default();

    let battery_level = dashboard_read_battery_level(&mut adb);
    let uptime_display = dashboard_read_uptime_display(&mut adb);
    let has_ota_package = dashboard_has_package(&mut adb, "com.lenovo.ota");

    Ok(DashboardDeviceInfo {
        connected: true,
        country_code: dashboard_display_value(&country_code),
        locale: dashboard_display_value(&locale),
        serial_number: dashboard_display_value(&serial_number),
        original_rom: dashboard_rom_label(&original_rom_raw),
        model_name: dashboard_model_label(&product_display, &product_device),
        product_device: dashboard_display_value(&product_device),
        platform: dashboard_display_value(&platform),
        installed_rom: dashboard_rom_label(&installed_rom_raw),
        firmware_version: dashboard_firmware_version_label(&build_display_id),
        widevine_level: dashboard_widevine_label(&widevine_prop, &drm_widevine_dump),
        android_version: dashboard_display_value(&android_version),
        slot_suffix: dashboard_slot_label(&slot_suffix),
        hardware_info: dashboard_hwboard_label(&hw_board_id),
        battery_level,
        uptime_display,
        ap_chipset: dashboard_ap_chipset_label(&ap_chipset_raw, &platform),
        ota_status: dashboard_ota_status_label(&installed_rom_raw, has_ota_package),
    })
}


fn ensure_dashboard_adb_ready(adb: &mut AdbManager) -> Result<()> {
    match adb.check_device_state()? {
        AdbUsbState::Ready => Ok(()),

        AdbUsbState::ServerBlocking => {
            let _ = kill_adb_server_direct();
            thread::sleep(Duration::from_millis(200));

            match adb.check_device_state()? {
                AdbUsbState::Ready => Ok(()),
                AdbUsbState::Unauthorized => Err(LpmError::Adb(ADB_UNAUTHORIZED_GUIDE.to_string())),
                AdbUsbState::ServerBlocking => Err(LpmError::Adb(
                    "외부 adb server가 USB ADB 인터페이스를 점유 중입니다.".to_string(),
                )),
                AdbUsbState::NoDevice => Err(LpmError::Adb(
                    "USB ADB 기기를 찾지 못했습니다.".to_string(),
                )),
                AdbUsbState::Error(err) => Err(LpmError::Adb(err)),
            }
        }

        AdbUsbState::Unauthorized => Err(LpmError::Adb(ADB_UNAUTHORIZED_GUIDE.to_string())),

        AdbUsbState::NoDevice => Err(LpmError::Adb(
            "USB ADB 기기를 찾지 못했습니다.".to_string(),
        )),

        AdbUsbState::Error(err) => Err(LpmError::Adb(err)),
    }
}

fn dashboard_getprop(adb: &mut AdbManager, name: &str) -> String {
    adb.shell(&format!("getprop {name}"))
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn dashboard_has_package(adb: &mut AdbManager, package_name: &str) -> bool {
    let output = adb
        .shell(&format!("pm list packages {package_name}"))
        .unwrap_or_default();

    output.lines().any(|line| {
        let line = line.trim();
        line == format!("package:{package_name}") || line.contains(package_name)
    })
}

fn dashboard_ap_chipset_label(ap_chipset_raw: &str, platform: &str) -> String {
    let ap_chipset_raw = ap_chipset_raw.trim();
    let platform = platform.trim();

    if !ap_chipset_raw.is_empty() {
        ap_chipset_raw.to_string()
    } else if !platform.is_empty() {
        platform.to_string()
    } else {
        "알 수 없음".to_string()
    }
}

fn dashboard_ota_status_label(installed_rom_raw: &str, has_ota_package: bool) -> String {
    match normalize_region(installed_rom_raw) {
        RomRegion::Row => {
            if has_ota_package {
                "차단 필요".to_string()
            } else {
                "차단 완료".to_string()
            }
        }
        RomRegion::Prc => {
            if has_ota_package {
                "사용 중".to_string()
            } else {
                "사용 하지 않음".to_string()
            }
        }
        RomRegion::Unknown => {
            if has_ota_package {
                "감지됨".to_string()
            } else {
                "알 수 없음".to_string()
            }
        }
    }
}

fn dashboard_display_value(value: &str) -> String {
    let value = value.trim();

    if value.is_empty() {
        "알 수 없음".to_string()
    } else {
        value.to_string()
    }
}

fn dashboard_widevine_label(prop_value: &str, drm_dump: &str) -> String {
    let prop = prop_value.trim().to_uppercase();

    let default_security_line = drm_dump
        .lines()
        .find(|line| line.to_lowercase().contains("default_security_level"))
        .unwrap_or("")
        .to_uppercase();

    let prop_l1 = prop == "1" || prop == "L1" || prop.contains("LEVEL_1");
    let prop_l3 = prop == "3" || prop == "L3" || prop.contains("LEVEL_3");

    let dump_l1 = default_security_line.contains("L1")
        || default_security_line.contains("LEVEL_1")
        || default_security_line.contains("HW_SECURE");

    let dump_l3 = default_security_line.contains("L3")
        || default_security_line.contains("LEVEL_3")
        || default_security_line.contains("SW_SECURE");

    if prop_l1 || dump_l1 {
        "L1".to_string()
    } else if prop_l3 || dump_l3 {
        "L3".to_string()
    } else if !prop.is_empty() || !default_security_line.is_empty() {
        "L3".to_string()
    } else {
        "알 수 없음".to_string()
    }
}

fn dashboard_slot_label(value: &str) -> String {
    let value = value.trim().trim_start_matches('_');

    if value.is_empty() {
        "알 수 없음".to_string()
    } else {
        value.to_uppercase()
    }
}

fn dashboard_hwboard_label(value: &str) -> String {
    let value = value.trim();

    if value.is_empty() {
        return "알 수 없음".to_string();
    }

    if let Some(pair) = extract_hw_memory_storage_pair(value) {
        return pair;
    }

    "알 수 없음".to_string()
}

fn extract_hw_memory_storage_pair(value: &str) -> Option<String> {
    let normalized = value.replace('-', "_").replace('.', "_");

    for token in normalized.split('_') {
        if let Some((ram, storage)) = token.split_once('+') {
            let ram = ram.trim();
            let storage = storage.trim();

            let valid_ram = ["6", "8", "12", "16"].contains(&ram);
            let valid_storage = ["128", "256", "512"].contains(&storage);

            if valid_ram && valid_storage {
                return Some(format!("{ram}+{storage}"));
            }
        }
    }

    None
}

fn dashboard_read_battery_level(adb: &mut AdbManager) -> Option<u8> {
    let output = adb.shell("dumpsys battery").ok()?;

    for line in output.lines() {
        let line = line.trim();

        if let Some(value) = line.strip_prefix("level:") {
            if let Ok(level) = value.trim().parse::<u8>() {
                return Some(level.min(100));
            }
        }
    }

    None
}

fn dashboard_read_uptime_display(adb: &mut AdbManager) -> String {
    let output = adb.shell("cat /proc/uptime").unwrap_or_default();

    let first_value = output
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim();

    let Ok(total_seconds) = first_value.parse::<f64>() else {
        return "알 수 없음".to_string();
    };

    format_uptime_seconds_display(total_seconds)
}

fn format_uptime_seconds_display(total_seconds: f64) -> String {
    let safe_seconds = if total_seconds.is_finite() && total_seconds >= 0.0 {
        total_seconds
    } else {
        0.0
    };

    let whole_seconds = safe_seconds.floor() as u64;

    let days = whole_seconds / 86_400;
    let hours = (whole_seconds % 86_400) / 3_600;
    let minutes = (whole_seconds % 3_600) / 60;

    format!("{days}일 {hours}시간 {minutes}분")
}

fn dashboard_model_label(product_display: &str, product_device: &str) -> String {
    let product_display = product_display.trim();
    let product_device = product_device.trim();

    match (product_display.is_empty(), product_device.is_empty()) {
        (true, true) => "알 수 없음".to_string(),
        (false, true) => product_display.to_string(),
        (true, false) => product_device.to_string(),
        (false, false) => {
            if product_display.contains(product_device) {
                product_display.to_string()
            } else {
                format!("{product_display} ({product_device})")
            }
        }
    }
}

fn dashboard_rom_label(value: &str) -> String {
    match normalize_region(value) {
        RomRegion::Prc => "PRC(중국 내수롬)".to_string(),
        RomRegion::Row => "ROW(글로벌롬)".to_string(),
        RomRegion::Unknown => {
            let value = value.trim();

            if value.is_empty() {
                "알 수 없음".to_string()
            } else {
                value.to_string()
            }
        }
    }
}

fn dashboard_firmware_version_label(build_display_id: &str) -> String {
    let build_display_id = build_display_id.trim();

    if build_display_id.is_empty() {
        return "알 수 없음".to_string();
    }

    extract_zui_version_segment(build_display_id)
        .unwrap_or_else(|| build_display_id.to_string())
}

fn extract_zui_version_segment(value: &str) -> Option<String> {
    for prefix in ["ZUI_", "ZUXOS_", "ZUX1_"] {
        if let Some(start) = value.find(prefix) {
            let tail = &value[start..];

            if let Some(end) = tail.find("_ST") {
                return Some(tail[..end + 3].to_string());
            }
        }
    }

    None
}



pub fn detect_preloader_once() -> PreloaderDetectResult {
    let checked_tokens = preloader_tokens();

    if !cfg!(windows) {
        return PreloaderDetectResult {
            detected: false,
            display_name: None,
            method: Some("Windows 전용 감지".to_string()),
            checked_tokens,
        };
    }

    let methods: &[(&str, &str, &[&str])] = &[
        (
            "pnputil /enum-devices /connected",
            "pnputil",
            &["/enum-devices", "/connected"],
        ),
        (
            "wmic Win32_PnPEntity ConfigManagerErrorCode=0",
            "wmic",
            &[
                "path",
                "Win32_PnPEntity",
                "where",
                "ConfigManagerErrorCode=0",
                "get",
                "Name",
                "/value",
            ],
        ),
    ];

    for (_method_name, program, args) in methods {
        if let Some(display_name) = run_and_find_preloader(program, args) {
            return PreloaderDetectResult {
                detected: true,
                display_name: Some(display_name),
                method: None,
                checked_tokens,
            };
        }
    }

    PreloaderDetectResult {
        detected: false,
        display_name: None,
        method: None,
        checked_tokens,
    }
}

pub fn detect_preloader_until_timeout(timeout_secs: u64) -> PreloaderDetectResult {
    if !cfg!(windows) {
        return detect_preloader_once();
    }

    let deadline = Instant::now() + Duration::from_secs(timeout_secs);

    loop {
        let result = detect_preloader_once();

        if result.detected {
            return result;
        }

        if Instant::now() >= deadline {
            return PreloaderDetectResult {
                detected: false,
                display_name: None,
                method: Some(format!("{timeout_secs}초 제한 시간 초과")),
                checked_tokens: preloader_tokens(),
            };
        }

        thread::sleep(Duration::from_millis(PRELOADER_POLL_INTERVAL_MS));
    }
}

fn preloader_tokens() -> Vec<String> {
    PRELOADER_TOKENS
        .iter()
        .map(|token| (*token).to_string())
        .collect()
}

fn run_and_find_preloader(program: &str, args: &[&str]) -> Option<String> {
    let output = hidden_command_output(program, args)?;

    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stderr));

    find_matching_line(&text)
}

fn find_matching_line(text: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        if contains_preloader_token(line) {
            return Some(clean_device_line(line));
        }
    }

    None
}

fn contains_preloader_token(line: &str) -> bool {
    let upper_line = line.to_uppercase();

    PRELOADER_TOKENS.iter().any(|token| {
        let upper_token = token.to_uppercase();
        upper_line.contains(&upper_token)
    })
}

fn clean_device_line(line: &str) -> String {
    let trimmed = line.trim();

    if let Some(value) = trimmed.strip_prefix("Name=") {
        return value.trim().to_string();
    }

    trimmed.to_string()
}

const MTK_DRIVER_URL: &str = "https://mtkdriver.com/wp-content/uploads/MTK-Driver-v5.1632.zip";
const MTK_DRIVER_PACKAGE_DIR_NAME: &str = "MTK-Driver-v5.1632";
const MTK_DRIVER_INSTALLER_FILE_NAME: &str = "MTK-Driver-Setup.exe";
const MTK_DRIVER_REVISION_MARKER: &str = ".lpmbox_mtk_driver_v5.1632";
#[cfg(windows)]
const MTK_DRIVER_REGISTRY_KEY: &str = r"HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\MediaTek SP Driver_is1";
#[cfg(windows)]
static MTK_DRIVER_PACKAGE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

const VC_REDIST_X86_URL: &str = "https://download.visualstudio.microsoft.com/download/pr/57eef8ae-a341-46c3-b0bc-c041027b54cd/F0BAB33A302B3CDB2E11113760D016F54FD3D2632C65BA7834FAC4F0ABD7F1A3/VC_redist.x86.exe";
const VC_REDIST_X64_URL: &str = "https://download.visualstudio.microsoft.com/download/pr/ebdab8e5-1d7b-4d9f-a11b-cbb1720c3b12/843068991DAAA1F73AD9F6239BCE4D0F6A07A51F18C37EA2A867E9BECA71295C/VC_redist.x64.exe";
#[cfg(windows)]
const VC_RUNTIME_X86_REGISTRY_KEY: &str = r"HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\{9a6ce18d-11c0-4452-aa35-9f2b8437c686}";
#[cfg(windows)]
const VC_RUNTIME_X64_REGISTRY_KEY: &str = r"HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\{0e3bb569-69d6-4c34-bff9-c2f81db5e5f0}";
#[cfg(windows)]
static VC_RUNTIME_PACKAGE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VcRuntimeStatus {
    pub x86_installed: bool,
    pub x64_installed: bool,
}

impl VcRuntimeStatus {
    pub fn all_installed(self) -> bool {
        self.x86_installed && self.x64_installed
    }
}

pub fn check_vc_runtime_installed() -> Result<VcRuntimeStatus> {
    check_vc_runtime_installed_impl()
}

#[cfg(not(windows))]
fn check_vc_runtime_installed_impl() -> Result<VcRuntimeStatus> {
    Ok(VcRuntimeStatus {
        x86_installed: true,
        x64_installed: true,
    })
}

#[cfg(windows)]
fn check_vc_runtime_installed_impl() -> Result<VcRuntimeStatus> {
    let x86_key = powershell_single_quote_escape(VC_RUNTIME_X86_REGISTRY_KEY);
    let x64_key = powershell_single_quote_escape(VC_RUNTIME_X64_REGISTRY_KEY);
    let script = format!(
        r#"$ErrorActionPreference = 'SilentlyContinue'
$roots = @(
    'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*',
    'HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*'
)
$items = foreach ($root in $roots) {{ Get-ItemProperty -Path $root -ErrorAction SilentlyContinue }}
function Test-LpmVcRegistry([string]$arch, [string]$exactKey) {{
    if (Test-Path $exactKey) {{ return $true }}
    foreach ($item in $items) {{
        $name = [string]$item.DisplayName
        $version = [string]$item.DisplayVersion
        if ($name -match '(?i)^Microsoft Visual C\+\+' -and
            $name -match "(?i)$arch" -and
            $name -match '(?i)(Redistributable|Minimum Runtime|Additional Runtime)' -and
            ($version -eq '' -or $version -match '^14\.')) {{
            return $true
        }}
    }}
    return $false
}}
$is64 = [Environment]::Is64BitOperatingSystem
$x86DllDir = if ($is64) {{ Join-Path $env:WINDIR 'SysWOW64' }} else {{ Join-Path $env:WINDIR 'System32' }}
$x64DllDir = if ($is64 -and -not [Environment]::Is64BitProcess) {{ Join-Path $env:WINDIR 'Sysnative' }} else {{ Join-Path $env:WINDIR 'System32' }}
$x86Registry = Test-LpmVcRegistry 'x86' '{x86_key}'
$x64Registry = if ($is64) {{ Test-LpmVcRegistry 'x64' '{x64_key}' }} else {{ $true }}
$x86Dll = (Test-Path (Join-Path $x86DllDir 'VCRUNTIME140.dll')) -and (Test-Path (Join-Path $x86DllDir 'MSVCP140.dll'))
$x64Dll = if ($is64) {{ (Test-Path (Join-Path $x64DllDir 'VCRUNTIME140.dll')) -and (Test-Path (Join-Path $x64DllDir 'MSVCP140.dll')) }} else {{ $true }}
if ($x86Registry -and $x86Dll) {{ Write-Output 'LPMBOX_VC_X86_FOUND' }}
if ($x64Registry -and $x64Dll) {{ Write-Output 'LPMBOX_VC_X64_FOUND' }}
"#
    );

    let output = hidden_command_output(
        "powershell",
        &["-NoProfile", "-NonInteractive", "-Command", &script],
    )
    .ok_or_else(|| {
        LpmError::InvalidFirmwareFolder(
            "Microsoft Visual C++ 런타임 설치 여부 확인 명령을 실행하지 못했습니다."
                .to_string(),
        )
    })?;

    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stderr));

    Ok(VcRuntimeStatus {
        x86_installed: text.contains("LPMBOX_VC_X86_FOUND"),
        x64_installed: text.contains("LPMBOX_VC_X64_FOUND"),
    })
}

pub fn install_vc_runtimes<F>(mut on_log: F) -> Result<()>
where
    F: FnMut(String),
{
    install_vc_runtimes_impl(&mut on_log)
}

#[cfg(not(windows))]
fn install_vc_runtimes_impl<F>(on_log: &mut F) -> Result<()>
where
    F: FnMut(String),
{
    on_log(
        "[Runtime] Microsoft Visual C++ 런타임 설치는 Windows에서만 지원됩니다."
            .to_string(),
    );
    Err(LpmError::InvalidFirmwareFolder(
        "Microsoft Visual C++ 런타임 설치는 Windows에서만 지원됩니다.".to_string(),
    ))
}

#[cfg(windows)]
fn install_vc_runtimes_impl<F>(on_log: &mut F) -> Result<()>
where
    F: FnMut(String),
{
    let lock = VC_RUNTIME_PACKAGE_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().map_err(|_| {
        LpmError::InvalidFirmwareFolder(
            "Microsoft Visual C++ 런타임 설치 잠금을 가져오지 못했습니다.".to_string(),
        )
    })?;

    let initial = check_vc_runtime_installed_impl()?;
    if initial.all_installed() {
        on_log("[Runtime] Microsoft Visual C++ x86/x64 런타임이 이미 설치되어 있습니다.".to_string());
        return Ok(());
    }

    std::fs::create_dir_all(app_paths::tool_download_dir())?;
    let mut restart_required = false;

    if !initial.x86_installed {
        let installer = app_paths::vc_redist_x86_path();
        on_log("[Runtime] Microsoft Visual C++ x86 설치 파일을 다운로드 합니다.".to_string());
        download_vc_redist(VC_REDIST_X86_URL, &installer)?;
        on_log(format!(
            "[Runtime] Microsoft Visual C++ x86 설치 파일을 실행합니다: {}",
            installer.display()
        ));
        restart_required |= run_vc_redist_elevated(&installer)?;
    }

    let after_x86 = check_vc_runtime_installed_impl()?;

    if !after_x86.x64_installed {
        let installer = app_paths::vc_redist_x64_path();
        on_log("[Runtime] Microsoft Visual C++ x64 설치 파일을 다운로드 합니다.".to_string());
        download_vc_redist(VC_REDIST_X64_URL, &installer)?;
        on_log(format!(
            "[Runtime] Microsoft Visual C++ x64 설치 파일을 실행합니다: {}",
            installer.display()
        ));
        restart_required |= run_vc_redist_elevated(&installer)?;
    }

    thread::sleep(Duration::from_millis(500));
    let final_status = check_vc_runtime_installed_impl()?;

    if !final_status.all_installed() {
        let mut missing = Vec::new();
        if !final_status.x86_installed {
            missing.push("x86");
        }
        if !final_status.x64_installed {
            missing.push("x64");
        }

        return Err(LpmError::InvalidFirmwareFolder(format!(
            "Microsoft Visual C++ 런타임 설치 완료를 확인하지 못했습니다: {}. 설치 창을 완료한 뒤 다시 시도해주세요.",
            missing.join(", ")
        )));
    }

    on_log("[Runtime] Microsoft Visual C++ x86/x64 런타임 설치 감지 완료".to_string());

    if restart_required {
        on_log(
            "[Runtime] Microsoft Visual C++ 런타임 설치가 완료되었습니다. Windows 재시작이 필요할 수 있습니다."
                .to_string(),
        );
    }

    Ok(())
}

#[cfg(windows)]
fn download_vc_redist(url: &str, out_path: &Path) -> Result<()> {
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let part_path = out_path.with_file_name(format!(
        "{}.part",
        out_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("VC_redist.exe")
    ));

    if part_path.exists() {
        std::fs::remove_file(&part_path)?;
    }

    let response = ureq::get(url)
        .set("User-Agent", "Mozilla/5.0")
        .call()
        .map_err(|err| {
            LpmError::InvalidFirmwareFolder(format!(
                "Microsoft Visual C++ 런타임 다운로드 실패: {err}"
            ))
        })?;

    if !(200..300).contains(&response.status()) {
        return Err(LpmError::InvalidFirmwareFolder(format!(
            "Microsoft Visual C++ 런타임 다운로드 HTTP 오류: {}",
            response.status()
        )));
    }

    let mut reader = response.into_reader();
    let mut file = std::fs::File::create(&part_path)?;
    std::io::copy(&mut reader, &mut file)?;
    drop(file);

    let size = std::fs::metadata(&part_path)?.len();
    if size < 1024 * 1024 {
        let _ = std::fs::remove_file(&part_path);
        return Err(LpmError::InvalidFirmwareFolder(format!(
            "Microsoft Visual C++ 런타임 설치 파일 크기가 올바르지 않습니다: {} bytes",
            size
        )));
    }

    let mut header = [0_u8; 2];
    std::fs::File::open(&part_path)?.read_exact(&mut header)?;
    if header != *b"MZ" {
        let _ = std::fs::remove_file(&part_path);
        return Err(LpmError::InvalidFirmwareFolder(
            "Microsoft Visual C++ 런타임 설치 파일 형식이 올바르지 않습니다."
                .to_string(),
        ));
    }

    if out_path.exists() {
        std::fs::remove_file(out_path)?;
    }
    std::fs::rename(&part_path, out_path)?;

    Ok(())
}

#[cfg(windows)]
fn run_vc_redist_elevated(path: &Path) -> Result<bool> {
    let path_text = powershell_single_quote_escape(&path.to_string_lossy());
    let script = format!(
        "try {{ $p = Start-Process -FilePath '{path_text}' -ArgumentList @('/install','/passive','/norestart') -Verb RunAs -Wait -PassThru -ErrorAction Stop; if ($null -eq $p.ExitCode) {{ exit 1 }} else {{ exit $p.ExitCode }} }} catch {{ exit 1223 }}"
    );

    let output = hidden_powershell(&script)?;
    match output.status.code().unwrap_or(-1) {
        0 | 1638 => Ok(false),
        1641 | 3010 => Ok(true),
        1223 => Err(LpmError::InvalidFirmwareFolder(
            "Microsoft Visual C++ 런타임 설치가 관리자 권한 확인 창에서 취소되었습니다."
                .to_string(),
        )),
        other => Err(LpmError::InvalidFirmwareFolder(format!(
            "Microsoft Visual C++ 런타임 설치 명령이 실패했습니다. exit_code={other}"
        ))),
    }
}

pub fn check_mtk_driver_installed() -> Result<bool> {
    check_mtk_driver_installed_impl()
}

#[cfg(not(windows))]
fn check_mtk_driver_installed_impl() -> Result<bool> {
    Ok(true)
}

#[cfg(windows)]
fn check_mtk_driver_installed_impl() -> Result<bool> {
    let key = powershell_single_quote_escape(MTK_DRIVER_REGISTRY_KEY);
    let script = format!(
        "$ErrorActionPreference = 'SilentlyContinue'; if (Test-Path '{key}') {{ Write-Output 'LPMBOX_MTK_DRIVER_FOUND' }}"
    );

    let output = hidden_command_output(
        "powershell",
        &["-NoProfile", "-NonInteractive", "-Command", &script],
    )
    .ok_or_else(|| {
        LpmError::InvalidFirmwareFolder(
            "MTK 드라이버 설치 여부 확인 명령을 실행하지 못했습니다.".to_string(),
        )
    })?;

    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stderr));

    Ok(text.contains("LPMBOX_MTK_DRIVER_FOUND"))
}

pub fn prepare_mtk_driver_package<F>(mut on_log: F) -> Result<()>
where
    F: FnMut(String),
{
    prepare_mtk_driver_package_impl(&mut on_log)
}

#[cfg(not(windows))]
fn prepare_mtk_driver_package_impl<F>(on_log: &mut F) -> Result<()>
where
    F: FnMut(String),
{
    on_log("[Driver] MTK 드라이버 파일 준비는 Windows에서만 지원됩니다.".to_string());
    Err(LpmError::InvalidFirmwareFolder(
        "MTK 드라이버 파일 준비는 Windows에서만 지원됩니다.".to_string(),
    ))
}

#[cfg(windows)]
fn prepare_mtk_driver_package_impl<F>(on_log: &mut F) -> Result<()>
where
    F: FnMut(String),
{
    let lock = MTK_DRIVER_PACKAGE_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().map_err(|_| {
        LpmError::InvalidFirmwareFolder(
            "MTK 드라이버 파일 준비 잠금을 가져오지 못했습니다.".to_string(),
        )
    })?;

    let driver_dir = app_paths::mtk_driver_dir();
    let zip_path = app_paths::mtk_driver_zip_path();
    let revision_marker = driver_dir.join(MTK_DRIVER_REVISION_MARKER);

    if revision_marker.is_file() && find_mtk_driver_installer(&driver_dir).is_some() {
        on_log(format!(
            "[Driver] MTK 드라이버 설치 파일 준비 완료: {}",
            driver_dir.display()
        ));
        return Ok(());
    }

    if let Some(parent) = zip_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if driver_dir.exists() {
        std::fs::remove_dir_all(&driver_dir)?;
    }

    std::fs::create_dir_all(&driver_dir)?;

    if zip_path.exists() {
        std::fs::remove_file(&zip_path)?;
    }

    on_log("[Driver] MTK 드라이버 파일을 다운로드 합니다.".to_string());
    download_mtk_driver_zip(MTK_DRIVER_URL, &zip_path)?;

    on_log(format!(
        "[Driver] MTK 드라이버 압축을 {} 경로에 해제합니다.",
        driver_dir.display()
    ));
    extract_mtk_driver_zip_file(&zip_path, &driver_dir)?;

    let installer = driver_dir.join(MTK_DRIVER_INSTALLER_FILE_NAME);
    if !installer.is_file() && find_mtk_driver_installer(&driver_dir).is_none() {
        return Err(LpmError::FileNotFound(format!(
            "MTK 드라이버 설치 파일을 찾지 못했습니다: {}",
            installer.display()
        )));
    }

    std::fs::write(&revision_marker, b"5.1632")?;

    on_log(format!(
        "[Driver] MTK 드라이버 설치 파일 준비 완료: {}",
        driver_dir.display()
    ));

    Ok(())
}

pub fn install_mtk_driver<F>(mut on_log: F) -> Result<()>
where
    F: FnMut(String),
{
    install_mtk_driver_impl(&mut on_log)
}

#[cfg(not(windows))]
fn install_mtk_driver_impl<F>(on_log: &mut F) -> Result<()>
where
    F: FnMut(String),
{
    on_log("[Driver] MTK 드라이버 설치는 Windows에서만 지원됩니다.".to_string());
    Err(LpmError::InvalidFirmwareFolder(
        "MTK 드라이버 설치는 Windows에서만 지원됩니다.".to_string(),
    ))
}

#[cfg(windows)]
fn install_mtk_driver_impl<F>(on_log: &mut F) -> Result<()>
where
    F: FnMut(String),
{
    let driver_dir = app_paths::mtk_driver_dir();

    prepare_mtk_driver_package_impl(on_log)?;

    let installer = driver_dir.join(MTK_DRIVER_INSTALLER_FILE_NAME);
    let installer = if installer.is_file() {
        installer
    } else {
        find_mtk_driver_installer(&driver_dir).ok_or_else(|| {
            LpmError::FileNotFound(format!(
                "MTK 드라이버 설치 파일을 찾지 못했습니다: {}",
                driver_dir.display()
            ))
        })?
    };

    on_log(format!(
        "[Driver] MTK 드라이버 설치 파일을 실행합니다: {}",
        installer.display()
    ));
    run_elevated_file(&installer)?;

    if check_mtk_driver_installed_impl()? {
        on_log("[Driver] MTK 드라이버 설치 감지 완료".to_string());
        Ok(())
    } else {
        Err(LpmError::InvalidFirmwareFolder(
            "MTK 드라이버 설치 완료를 확인하지 못했습니다. 설치 창을 완료한 뒤 다시 시도해주세요.".to_string(),
        ))
    }
}

#[cfg(windows)]
fn download_mtk_driver_zip(url: &str, out_path: &Path) -> Result<()> {
    let response = ureq::get(url)
        .set("User-Agent", "Mozilla/5.0")
        .call()
        .map_err(|err| LpmError::InvalidFirmwareFolder(format!("MTK 드라이버 다운로드 실패: {err}")))?;

    if !(200..300).contains(&response.status()) {
        return Err(LpmError::InvalidFirmwareFolder(format!(
            "MTK 드라이버 다운로드 HTTP 오류: {}",
            response.status()
        )));
    }

    let mut reader = response.into_reader();
    let mut file = std::fs::File::create(out_path)?;
    std::io::copy(&mut reader, &mut file)?;

    let size = std::fs::metadata(out_path)?.len();

    if size == 0 {
        return Err(LpmError::InvalidFirmwareFolder(format!(
            "MTK 드라이버 ZIP 파일 크기가 0입니다: {}",
            out_path.display()
        )));
    }

    Ok(())
}

#[cfg(windows)]
fn extract_mtk_driver_zip_file(zip_path: &Path, target_dir: &Path) -> Result<()> {
    let file = std::fs::File::open(zip_path)?;

    let mut archive = zip::ZipArchive::new(file).map_err(|err| {
        LpmError::InvalidFirmwareFolder(format!(
            "MTK 드라이버 ZIP 열기 실패: {} / {err}",
            zip_path.display()
        ))
    })?;

    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(|err| {
            LpmError::InvalidFirmwareFolder(format!(
                "MTK 드라이버 ZIP 항목 읽기 실패: {} / {err}",
                zip_path.display()
            ))
        })?;

        let Some(enclosed_name) = file.enclosed_name().map(|path| path.to_path_buf()) else {
            continue;
        };

        let relative_path = normalize_mtk_driver_zip_entry_path(&enclosed_name);

        if relative_path.as_os_str().is_empty() {
            continue;
        }

        let out_path = target_dir.join(relative_path);

        if file.is_dir() {
            std::fs::create_dir_all(&out_path)?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut out_file = std::fs::File::create(&out_path)?;
        std::io::copy(&mut file, &mut out_file)?;
    }

    Ok(())
}

#[cfg(windows)]
fn normalize_mtk_driver_zip_entry_path(path: &Path) -> PathBuf {
    let mut normalized = path.to_path_buf();

    loop {
        let mut components = normalized.components();

        let Some(first) = components.next() else {
            return PathBuf::new();
        };

        if first
            .as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case(MTK_DRIVER_PACKAGE_DIR_NAME)
            || first
                .as_os_str()
                .to_string_lossy()
                .eq_ignore_ascii_case("MTK-Driver")
        {
            normalized = components.as_path().to_path_buf();
            continue;
        }

        return normalized;
    }
}

#[cfg(windows)]
fn find_mtk_driver_installer(root: &Path) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    collect_files_by_extension(root, "exe", &mut candidates);

    let preferred_names = [
        MTK_DRIVER_INSTALLER_FILE_NAME,
        "MTK Driver Setup.exe",
        "DriverInstall.exe",
        "DriverInstaller.exe",
        "InstallDriver.exe",
        "MTK_Driver_Installer.exe",
        "setup.exe",
    ];

    for preferred in preferred_names {
        if let Some(path) = candidates.iter().find(|path| file_name_eq(path, preferred)) {
            return Some(path.clone());
        }
    }

    if let Some(path) = candidates.iter().find(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| {
                let lower = name.to_ascii_lowercase();
                lower.contains("install") || lower.contains("driver")
            })
            .unwrap_or(false)
    }) {
        return Some(path.clone());
    }

    candidates.into_iter().next()
}

#[cfg(windows)]
fn collect_files_by_extension(root: &Path, extension: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };

    for entry in entries.filter_map(|entry| entry.ok()) {
        let path = entry.path();

        if path.is_dir() {
            collect_files_by_extension(&path, extension, out);
            continue;
        }

        if path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.eq_ignore_ascii_case(extension))
            .unwrap_or(false)
        {
            out.push(path);
        }
    }
}

#[cfg(windows)]
fn file_name_eq(path: &Path, expected: &str) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.eq_ignore_ascii_case(expected))
        .unwrap_or(false)
}

#[cfg(windows)]
fn run_elevated_file(path: &Path) -> Result<()> {
    let path_text = powershell_single_quote_escape(&path.to_string_lossy());
    let script = format!(
        "try {{ $p = Start-Process -FilePath '{path_text}' -Verb RunAs -Wait -PassThru -ErrorAction Stop; if ($null -eq $p.ExitCode) {{ exit 1 }} else {{ exit $p.ExitCode }} }} catch {{ exit 1223 }}"
    );

    let output = hidden_powershell(&script)?;
    handle_elevated_exit(output.status.code().unwrap_or(-1))
}

#[cfg(windows)]
fn hidden_powershell(script: &str) -> Result<Output> {
    let mut command = Command::new("powershell");
    command
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg(script);

    command.creation_flags(CREATE_NO_WINDOW);

    command.output().map_err(LpmError::Io)
}

#[cfg(windows)]
fn handle_elevated_exit(code: i32) -> Result<()> {
    match code {
        0 => Ok(()),
        1223 => Err(LpmError::InvalidFirmwareFolder(
            "MTK 드라이버 설치가 관리자 권한 확인 창에서 취소되었습니다.".to_string(),
        )),
        other => Err(LpmError::InvalidFirmwareFolder(format!(
            "MTK 드라이버 설치 명령이 실패했습니다. exit_code={other}"
        ))),
    }
}

#[cfg(windows)]
fn powershell_single_quote_escape(value: &str) -> String {
    value.replace('\'', "''")
}