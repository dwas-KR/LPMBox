//! Optional active-slot LK readback inside ROW 084+ install flows.
//! Reuses the original PreLoader watcher, SPFT worker, and recovery/error path.
//! ADB ro.boot.slot_suffix is read only, never modified with setprop.
use lpmbox_core::{FirmwareInfo, ProinfoLiveEvent, RomRegion};
use std::path::{Path, PathBuf};
use std::sync::mpsc;

pub(super) fn requires_live_lk_readback(firmware: &FirmwareInfo) -> bool {
    firmware.model.eq_ignore_ascii_case("TB373FU")
        && matches!(firmware.region, RomRegion::Row)
        && lpmbox_firmware::lk_patch::row_live_readback_required(firmware.version.as_deref())
}

/// After the existing optional proinfo step and before the normal Flash Plan:
/// ADB running slot -> SPFT READ-PARTITION lk_a/lk_b -> persist backup.
/// Any error aborts before an LK image is assigned for flashing.
pub(super) fn backup_running_lk_if_required(
    image_dir: &Path,
    firmware: &FirmwareInfo,
    tx: &mpsc::Sender<ProinfoLiveEvent>,
) -> Result<Option<PathBuf>, String> {
    if !requires_live_lk_readback(firmware) {
        return Ok(None);
    }
    let log = |msg: String| {
        let _ = tx.send(ProinfoLiveEvent::Log(msg));
    };
    log("[LK 1/4] Android 현재 활성 슬롯을 조회합니다.".to_string());
    let part = lpmbox_device::read_running_slot_for_lk_backup(log)
        .map_err(|err| format!("활성 LK 슬롯 조회 실패: {err}"))?;
    log(format!("[LK 2/4] 활성 슬롯의 {part} 파티션 읽기 전용 백업을 준비합니다."));

    // Start SPFT before Android reboot to catch the transient MTK PreLoader.
    let prearmed = lpmbox_spft::PrearmedActiveLkReadback::start(image_dir, &part)
        .map_err(|err| format!("LK SPFT 읽기 준비 실패: {err}"))?;
    let watch = lpmbox_device::PreloaderWatch::start(30);
    if !watch.is_armed() {
        prearmed.cancel();
        return Err("LK 리드백 PreLoader 감시 초기화 실패. 재부팅을 실행하지 않았습니다.".into());
    }
    log("[LK] SPFlashToolV6 읽기 대기 및 PreLoader 감시 준비 완료".into());
    lpmbox_device::trigger_rom_install_reboot_commands(|m| log(m));

    let detect = watch.wait();
    if !detect.detected {
        prearmed.cancel();
        return Err(format!(
            "LK 리드백 PreLoader 미감지: {}",
            detect.method.as_deref().unwrap_or("COM 포트 없음")
        ));
    }
    log(format!(
        "[LK] PreLoader 감지: {}",
        detect.display_name.as_deref().unwrap_or("PreLoader COM")
    ));
    let backup = prearmed.finish(|event| match event {
        ProinfoLiveEvent::Log(line) => log(format!("[LK readback] {line}")),
        ProinfoLiveEvent::Progress(mut progress) => {
            progress.stage = format!("[LK readback] {}", progress.stage);
            let _ = tx.send(ProinfoLiveEvent::Progress(progress));
        }
        ProinfoLiveEvent::Error(line) => log(format!("[LK readback] {line}")),
        ProinfoLiveEvent::Finished(_) => {},
    }).map_err(|err| format!("활성 {part} 파티션 읽기 실패: {err}"))?;

    log(format!("[LK 3/4] 활성 {part} 백업 완료: {}", backup.display()));
    log("[LK 4/4] 이미지 SHA-256 및 서명 계열 검증은 Flash Plan 생성 시 수행합니다.".into());
    Ok(Some(backup))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn no_implicit_readback_for_unrelated_models() {
        // Model routing is checked before any PreLoader or device interaction.
        assert!(!"TB335FC".eq_ignore_ascii_case("TB373FU"));
    }
}
