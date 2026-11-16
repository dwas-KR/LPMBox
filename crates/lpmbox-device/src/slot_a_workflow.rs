//! Opt-in ROM-install preflight. The legacy slot routine remains unchanged.
use super::*;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

const PRELOADER_REQUIRED_APPEARANCES: u8 = 4;
const SLOT_CHANGE_RETRIES: usize = 3;
const SLOT_CHANGE_PAUSE: Duration = Duration::from_secs(2);
const MONITOR_TIMEOUT: Duration = Duration::from_secs(90);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotAStageRoute {
    /// Android is running from A. Preserve the original ADB reboot/PreLoader path.
    AlreadyA,
    /// Fastboot has rebooted the device and 4 distinct PreLoader arrivals were seen.
    /// Skip a second reboot; SP Flash Tool should now wait for the next USB arrival.
    RebootedAfterFourPreloaders,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BootArrival {
    FourPreloaders,
    AndroidReady(u8),
    AndroidUnauthorized(u8),
    UsbWrongMode(String),
    TimedOut(u8),
}

#[derive(Debug, Default)]
struct PreloaderEdges {
    was_present: bool,
    appearances: u8,
}

impl PreloaderEdges {
    fn observe(&mut self, present: bool) -> u8 {
        if present && !self.was_present {
            self.appearances = self.appearances.saturating_add(1);
        }
        self.was_present = present;
        self.appearances
    }
}

struct BootArrivalWatch {
    worker: thread::JoinHandle<BootArrival>,
    cancelled: Arc<AtomicBool>,
    armed: bool,
}

impl BootArrivalWatch {
    fn start() -> Self {
        let (ready_tx, ready_rx) = mpsc::sync_channel::<()>(1);
        let cancelled = Arc::new(AtomicBool::new(false));
        let stopped = Arc::clone(&cancelled);
        let worker = thread::spawn(move || {
            let mut edges = PreloaderEdges::default();
            let start = Instant::now();
            let mut next_adb_probe = start + Duration::from_secs(1);
            let _ = ready_tx.send(());
            while !stopped.load(Ordering::Relaxed) && start.elapsed() < MONITOR_TIMEOUT {
                // Only PRESENT COM arrivals count. Multiple polls during a single
                // appearance are one event, not four.
                let detected = detect_preloader_once().detected;
                if edges.observe(detected) >= PRELOADER_REQUIRED_APPEARANCES {
                    return BootArrival::FourPreloaders;
                }
                #[cfg(windows)]
                if let Some(model) = preloader_windows::scan_present_lenovo_usb_model() {
                    return BootArrival::UsbWrongMode(model);
                }
                if Instant::now() >= next_adb_probe {
                    next_adb_probe = Instant::now() + Duration::from_secs(1);
                    match probe_adb_usb_state_once() {
                        AdbUsbState::Ready => return BootArrival::AndroidReady(edges.appearances),
                        AdbUsbState::Unauthorized => {
                            return BootArrival::AndroidUnauthorized(edges.appearances)
                        }
                        _ => {}
                    }
                }
                thread::sleep(Duration::from_millis(PRELOADER_POLL_INTERVAL_MS));
            }
            BootArrival::TimedOut(edges.appearances)
        });
        let armed = ready_rx.recv_timeout(Duration::from_millis(1600)).is_ok();
        Self { worker, cancelled, armed }
    }

    fn abort(self) {
        self.cancelled.store(true, Ordering::Relaxed);
        let _ = self.worker.join();
    }

    fn wait(self) -> BootArrival {
        self.worker.join().unwrap_or(BootArrival::TimedOut(0))
    }
}

/// ADB getprop is *read-only*: it identifies the slot actually running now.
/// Changes require the boot control HAL (when authorized) or Fastboot; writing
/// ro.boot.slot_suffix is neither supported nor a substitute for boot metadata.
pub fn run_current_slot_a_stage_v2<F>(mut on_log: F) -> Result<SlotAStageRoute>
where
    F: FnMut(String),
{
    on_log("[슬롯] Android ADB에서 현재 실행 중인 슬롯을 확인합니다.".into());
    wait_for_adb_device_ready_until(&mut on_log, SLOT_A_ADB_WAIT_TIMEOUT)?;
    let current = adb_get_slot_suffix_checked()?.ok_or_else(|| {
        LpmError::Adb("ro.boot.slot_suffix가 _a/_b로 확인되지 않았습니다. 슬롯 변경을 중단합니다.".into())
    })?;
    on_log(format!("[ADB] ro.boot.slot_suffix={current} (읽기 전용)"));
    if current == "_a" {
        on_log("[슬롯] 현재 실행 슬롯이 A이므로 Fastboot 전환 없이 기존 ROM 설치 흐름을 진행합니다.".into());
        return Ok(SlotAStageRoute::AlreadyA);
    }
    if current != "_b" {
        return Err(LpmError::Adb(format!("예상하지 못한 슬롯 값: {current}")));
    }

    for attempt in 1..=SLOT_CHANGE_RETRIES {
        on_log(format!("[슬롯] B→A 전환 시도 {attempt}/{SLOT_CHANGE_RETRIES}: adb reboot bootloader"));
        adb_reboot_checked("bootloader")?;
        thread::sleep(SLOT_CHANGE_PAUSE);
        wait_for_fastboot_device(Duration::from_secs(60))?;

        let mut bootloader = NativeFastbootDevice::open()?;
        let slots = bootloader.getvar("slot-count").unwrap_or_default();
        let has_slot = bootloader.getvar("has-slot:boot").unwrap_or_default();
        slot_support_diagnostic(&slots, &has_slot)?;
        on_log(format!("[Fastboot] slot-count={}, has-slot:boot={}", slots.trim(), has_slot.trim()));
        // These are the on-wire equivalents of:
        //   fastboot --set-active=a
        //   fastboot -aa
        // Do not depend on an external platform-tools path in a packaged EXE.
        on_log("[Fastboot] fastboot --set-active=a (USB 프로토콜 set_active:a)".into());
        bootloader.set_active_a()?;
        thread::sleep(SLOT_CHANGE_PAUSE);
        on_log("[Fastboot] fastboot -aa (USB 프로토콜 set_active:a)".into());
        bootloader.set_active_a()?;
        thread::sleep(SLOT_CHANGE_PAUSE);
        on_log("[Fastboot] fastboot oem cdms fix".into());
        match bootloader.command("oem cdms fix") {
            Ok(info) => on_log(format!("[Fastboot] cdms fix 응답: {info}")),
            Err(err) => {
                return Err(LpmError::Fastboot(format!(
                    "OEM cdms fix 실패. 슬롯 상태를 보장할 수 없어 플래싱 중단: {err}"
                )));
            }
        }
        thread::sleep(SLOT_CHANGE_PAUSE);

        // Must arm the Windows SetupAPI observer BEFORE issuing the reboot.
        let watcher = BootArrivalWatch::start();
        if !watcher.armed {
            watcher.abort();
            return Err(LpmError::Fastboot("PreLoader 4회 감시 초기화 실패".into()));
        }
        on_log("[Fastboot] fastboot reboot + PreLoader COM 포트 4회 출현 감시 시작".into());
        if let Err(err) = bootloader.reboot() {
            watcher.abort();
            return Err(err);
        }
        drop(bootloader);

        match watcher.wait() {
            BootArrival::FourPreloaders => {
                on_log("[PreLoader] 서로 구분되는 USB 연결 4회 감지 완료".into());
                // The user explicitly requires an ADB check after the fourth event.
                match probe_adb_usb_state_once() {
                    AdbUsbState::Ready => {
                        on_log("[ADB] PreLoader 4회 이후 ADB가 다시 연결됨. 슬롯 재점검을 위해 재시도합니다.".into());
                        if attempt == SLOT_CHANGE_RETRIES { break; }
                    }
                    AdbUsbState::Unauthorized => {
                        return Err(LpmError::Adb(
                            "USB 디버깅 승인이 사라졌습니다. 태블릿 화면의 USB 디버깅 허용을 확인하세요.".into()
                        ));
                    }
                    _ => {
                        on_log("[슬롯] ADB 미감지. 재부팅이나 플래싱 후 슬롯 검증 없이 기존 SPFT 설치 단계로 이행합니다.".into());
                        return Ok(SlotAStageRoute::RebootedAfterFourPreloaders);
                    }
                }
            }
            BootArrival::AndroidReady(n) => {
                on_log(format!(
                    "[ADB] PreLoader {n}/4회 후 Android로 복귀함. adb reboot bootloader를 통해 전환을 반복합니다."
                ));
                if attempt == SLOT_CHANGE_RETRIES { break; }
            }
            BootArrival::AndroidUnauthorized(n) => {
                return Err(LpmError::Adb(format!(
                    "PreLoader {n}/4회 후 ADB unauthorized 상태입니다. USB 디버깅을 재승인하세요."
                )));
            }
            BootArrival::UsbWrongMode(model) => {
                return Err(LpmError::Adb(format!(
                    "[USB 안내] 범용 직렬 버스 장치에서 {model} 감지. USB 디버깅/드라이버/케이블을 확인하세요. LPMBOX 작업을 종료합니다."
                )));
            }
            BootArrival::TimedOut(n) => {
                return Err(LpmError::Fastboot(format!(
                    "PreLoader {n}/4회만 감지한 채 90초가 경과했습니다. 오인식 상태로 플래싱하지 않습니다."
                )));
            }
        }
    }
    Err(LpmError::Fastboot(format!(
        "A 슬롯 전환 재시도 {SLOT_CHANGE_RETRIES}회 후에도 Android가 다시 연결됩니다. 더 이상의 자동 재부팅을 중단합니다."
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn four_distinct_preloader_appearances_required() {
        let mut count = PreloaderEdges::default();
        for state in [false,true,true,true,false,true,false,true,false,true] {
            count.observe(state);
        }
        assert_eq!(count.appearances, 4);
    }
    #[test]
    fn continuous_preloader_presence_is_one_event() {
        let mut count = PreloaderEdges::default();
        for _ in 0..100 { count.observe(true); }
        assert_eq!(count.appearances, 1);
    }
}
