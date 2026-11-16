//! Model-specific LK generation policy for ROM installs.
//!
//! Do not reuse the MT6897 TB375FC/TB373FU LK patch on MT6835 hardware.
//! A reported firmware cutoff is not evidence of a valid binary patch.
//! Missing validated model-specific patch data MUST fail before SPFT or
//! modifying an original ROM image.
use lpmbox_core::{InstallMode, LpmError, Result, RomRegion};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LkRequirement {
    // Existing MT6897 ROW 084 shim is prepared in lk_patch.rs.
    Tb373ExistingVerifiedPatch,
    // MT6835 TB335FC/TB336FU and TB365FC/TB361FU:
    // no LK *binary modification* stage. Legacy, separate LK/DTBO
    // file staging in flash_plan remains unchanged.
    Mt6835NoBinaryPatch,
    // Exact ROW 321 pinned candidate: user reports successful real-device boot.
    Tb336Row321UserBoottested,
    // Exact user-confirmed cross-region LK: TB335FC PRC above 1.1.10.299.
    Tb335PrcSharedUserBoottested,
    // TB361FU ROW375 exact source hash, structural candidate only (not boot-verified).
    Tb361Row375Experimental,
    // No original + signed LK patch is available for this combination.
    Mt6835VerifiedPatchMissing,
    Unchanged,
}

fn version_parts(version: Option<&str>, prefix: &str) -> Option<[u32; 4]> {
    let raw = version?.trim().strip_prefix(prefix)?;
    let raw = raw.strip_suffix("_ST")?;
    let parts: Vec<u32> = raw.split('.').map(str::parse::<u32>)
        .collect::<std::result::Result<Vec<_>, _>>().ok()?;
    <[u32; 4]>::try_from(parts).ok()
}

pub fn requirement(
    model: &str,
    region: RomRegion,
    version: Option<&str>,
    mode: InstallMode,
) -> Result<LkRequirement> {
    if matches!(mode, InstallMode::CountryReset) {
        return Ok(LkRequirement::Unchanged);
    }
    let model = model.trim().to_ascii_uppercase();
    match (model.as_str(), region) {
        ("TB373FU", RomRegion::Row) => Ok(LkRequirement::Tb373ExistingVerifiedPatch),
        ("TB336FU", RomRegion::Row) => {
            let parsed=version_parts(version,"ZUI_").ok_or_else(||{
                LpmError::InvalidFirmwareFolder(format!(
                    "TB336FU ROW 펌웨어 버전 형식 확인 실패: {}",
                    version.unwrap_or("UNKNOWN")))
            })?;
            // One SHA-pinned family candidate is bundled for all recognized
            // ZUI 17.x ROW images. Unknown source LK bytes still require a
            // valid signed-image container at the read-only preflight.
            if parsed[0] == 17 {
                Ok(LkRequirement::Tb336Row321UserBoottested)
            }else{
                Ok(LkRequirement::Mt6835VerifiedPatchMissing)
            }
        }
        ("TB335FC", RomRegion::Prc) => {
            let parsed=version_parts(version,"ZUXOS_")
                .or_else(||version_parts(version,"ZUX_")).ok_or_else(||{
                    LpmError::InvalidFirmwareFolder(format!(
                        "TB335FC PRC 버전 확인 실패: {}",
                        version.unwrap_or("UNKNOWN")
                    ))
                })?;
            if parsed>[1,1,10,299]{
                Ok(LkRequirement::Tb335PrcSharedUserBoottested)
            }else{
                Ok(LkRequirement::Unchanged)
            }
        }
        ("TB361FU", RomRegion::Row) => {
            let parsed=version_parts(version,"ZUI_").ok_or_else(|| {
                LpmError::InvalidFirmwareFolder("TB361FU ROW 버전 확인 실패".into())
            })?;
            if parsed==[17,5,10,375]{
                Ok(LkRequirement::Tb361Row375Experimental)
            }else{
                Ok(LkRequirement::Mt6835VerifiedPatchMissing)
            }
        }
        ("TB365FC", RomRegion::Prc) => {
            let parsed=version_parts(version,"ZUXOS_")
                .or_else(||version_parts(version,"ZUX_")).ok_or_else(||{
                    LpmError::InvalidFirmwareFolder(
                        "TB365FC PRC 버전 확인 실패".into()
                    )
                })?;
            if parsed>[1,1,10,122]{
                Ok(LkRequirement::Mt6835VerifiedPatchMissing)
            }else{
                Ok(LkRequirement::Unchanged)
            }
        }
        _ => Ok(LkRequirement::Unchanged),
    }
}

/// Preflight only: never writes flash state, LK, DTBO, or original scatter.
pub fn enforce_before_flash_plan<F>(
    model: &str,
    region: RomRegion,
    version: Option<&str>,
    mode: InstallMode,
    on_log: &mut F,
) -> Result<()>
where F: FnMut(String) {
    match requirement(model, region, version, mode)? {
        LkRequirement::Tb373ExistingVerifiedPatch => {
            crate::lk_patch::validate_tb373_row_release(version)?;
            on_log(format!(
                "[LK 정책][LPMBOX v{}] TB373FU ROW: LPMBOX-LK 1.0.0 내장 LK SHA-256={} 검증 완료. ROW084 전용 파일은 필요하지 않습니다. 빌드 식별자={}",
                env!("CARGO_PKG_VERSION"),
                crate::lk_patch::TB375_RELEASE_SHA256,
                concat!("LPMBOX_LK_RELEASE_BUILD_V", env!("CARGO_PKG_VERSION"))
            ));
            Ok(())
        }
        LkRequirement::Mt6835NoBinaryPatch => {
            on_log(format!(
                "[LK 정책] {} {}: MT6835 계열은 별도 LK 바이너리 패치 단계를 생략합니다.                  기존 펌웨어 준비, LK/DTBO 파일 배치 및 서명 검사는 그대로 진행합니다.                  이 설정이 실기기 부팅 호환성을 증명하는 것은 아닙니다.",
                model,version.unwrap_or("UNKNOWN")
            ));
            Ok(())
        }
        LkRequirement::Tb336Row321UserBoottested => {
            on_log(format!("[LK 패치] {} {}: ROW321 사용자 부팅 성공 후보를 검증된 ROW 입력에 적용합니다. 선택한 원본 LK SHA-256과 공통 후보 SHA-256을 확인하고 LK_A만 반영합니다. 다른 버전의 독립적인 부팅 성공은 보증하지 않습니다.",
                model,version.unwrap_or("UNKNOWN")));
            Ok(())
        }
        LkRequirement::Tb335PrcSharedUserBoottested => {
            on_log(format!(
                "[LK 패치] {} {}: TB335FC PRC 1.1.10.299 초과 모든 버전에 공통 LPMBOX_lk.img를 적용합니다. 원본 LK 버전별 크기·구조·SHA-256으로 적용을 차단하지 않으며, 공통 파일 SHA-256은 확인합니다.",
                model,version.unwrap_or("UNKNOWN")
            ));
            Ok(())
        }
        LkRequirement::Tb361Row375Experimental => {
            on_log(format!(
                "[LK 차단] {} {}: SP Flash Tool 실기기 로그에서 TB361FU ROW375 생성 LK의 Signature(Entity) invalid 확인. 해당 후보는 오프라인 연구 전용으로 유지하며 기기에 다시 기록하지 않습니다.",
                model,version.unwrap_or("UNKNOWN")
            ));
            Err(LpmError::ExperimentalLkUnverified(
                "TB361FU ROW 17.5.10.375: 이전 실기기 플래싱에서 lk_a Signature(Entity) invalid가 확인되었습니다. 기존 실험 LK를 자동 기록하면 안 됩니다. 제조사 서명 또는 해당 기기에서 인증된 LK가 확보되기 전까지 사전검사에서 차단합니다.".into()
            ))
        }
        LkRequirement::Mt6835VerifiedPatchMissing => {
            on_log(format!("[LK 정책] {} {}: LK 패치가 필요하지만 대상 버전의 서명 검증된 패치가 없습니다.",
                model,version.unwrap_or("UNKNOWN")));
            if model.eq_ignore_ascii_case("TB365FC"){
                on_log("[LK 연구] TB365FC PRC122 원본과 TB361FU ROW375의 첫 cert1 해시는 일치합니다.                     PRC122 연구 후보 생성은 lpmbox_lk_research.exe에서 지원하지만,                     PRC 1.1.10.122 초과 실펌웨어 및 SPFT 인증 성공 근거가 없어                     이를 자동 LK_A 플래싱에 사용하지 않습니다.".into());
            }
            Err(LpmError::InvalidFirmwareFolder(format!(
                "{} {}: 사용할 수 있는 서명/부팅 검증된 LK 패치 파일이 없어 플래싱을 중단했습니다.",
                model,version.unwrap_or("UNKNOWN"))))
        }
        LkRequirement::Unchanged => Ok(()),
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn recognized_tb336_zui_17_versions_use_pinned_family_image(){
        for ver in ["ZUI_17.5.10.321_ST","ZUI_17.5.10.348_ST",
            "ZUI_17.5.10.271_ST","ZUI_17.0.10.260_ST"]{
            assert_eq!(requirement("TB336FU",RomRegion::Row,
                Some(ver),InstallMode::ConvertWipe).unwrap(),
                LkRequirement::Tb336Row321UserBoottested);
        }
        for version in ["ZUI_16.5.10.271_ST","ZUI_18.5.10.193_ST"]{
            assert_eq!(requirement("TB336FU",RomRegion::Row,
                Some(version),InstallMode::ConvertWipe).unwrap(),
                LkRequirement::Mt6835VerifiedPatchMissing);
        }
    }
    #[test]
    fn tb335fc_prc_upper_versions_use_boot_reported_cross_region_patch(){
        for v in ["ZUXOS_1.1.10.300_ST","ZUXOS_1.1.10.386_ST",
            "ZUXOS_1.1.10.522_ST","ZUXOS_1.5.10.321_ST",
            "ZUXOS_1.5.10.322_ST","ZUXOS_1.6.10.100_ST"]{
            assert_eq!(requirement("TB335FC",RomRegion::Prc,
                Some(v),InstallMode::ConvertWipe).unwrap(),
                LkRequirement::Tb335PrcSharedUserBoottested);
            enforce_before_flash_plan("TB335FC",RomRegion::Prc,
                Some(v),InstallMode::ConvertWipe,&mut |_|{}).unwrap();
        }
        for v in ["ZUXOS_1.1.10.299_ST","ZUXOS_1.0.10.999_ST"]{
            assert_eq!(requirement("TB335FC",RomRegion::Prc,
                Some(v),InstallMode::ConvertWipe).unwrap(),LkRequirement::Unchanged);
        }
    }
    #[test]
    fn prc_thresholds_are_strict_and_missing_patches_fail_closed(){
        for (model,base,next) in [
            ("TB365FC","ZUXOS_1.1.10.122_ST","ZUXOS_1.1.10.123_ST"),
        ]{
            assert_eq!(requirement(model,RomRegion::Prc,
                Some(base),InstallMode::ConvertWipe).unwrap(),LkRequirement::Unchanged);
            assert_eq!(requirement(model,RomRegion::Prc,
                Some(next),InstallMode::ConvertWipe).unwrap(),LkRequirement::Mt6835VerifiedPatchMissing);
        }
        assert_eq!(requirement("TB361FU",RomRegion::Row,
            Some("ZUI_17.0.10.231_ST"),InstallMode::ConvertWipe).unwrap(),
            LkRequirement::Mt6835VerifiedPatchMissing);
        assert_eq!(requirement("TB361FU",RomRegion::Row,
            Some("ZUI_17.5.10.375_ST"),InstallMode::ConvertWipe).unwrap(),
            LkRequirement::Tb361Row375Experimental);
        let fail=enforce_before_flash_plan(
            "TB361FU",RomRegion::Row,Some("ZUI_17.5.10.375_ST"),
            InstallMode::ConvertWipe,&mut |_|{}
        ).unwrap_err().to_string();
        assert!(fail.contains("Signature(Entity) invalid"));
        assert_eq!(requirement("TB361FU",RomRegion::Row,
            Some("ZUI_17.5.10.376_ST"),InstallMode::ConvertWipe).unwrap(),
            LkRequirement::Mt6835VerifiedPatchMissing);
        assert!(enforce_before_flash_plan("TB361FU",RomRegion::Row,
            Some("ZUI_17.0.10.231_ST"),InstallMode::ConvertWipe,&mut |_|{}).is_err());
    }
    #[test]
    fn tb373_verified_patch_unchanged(){
        assert_eq!(requirement("TB373FU",RomRegion::Row,
            Some("ZUI_17.5.10.107_ST"),InstallMode::ConvertWipe).unwrap(),
            LkRequirement::Tb373ExistingVerifiedPatch);
    }
    #[test]
    fn reset_does_not_touch_lk(){
        assert_eq!(requirement("TB336FU",RomRegion::Row,None,
            InstallMode::CountryReset).unwrap(),LkRequirement::Unchanged);
    }
}
