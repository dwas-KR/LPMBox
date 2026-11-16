//! TB336FU ROW 17.5.10.321 exact-hash user boot-tested LK image.
//!
//! The user reports successful boot on real hardware with this specific
//! candidate. An OEM signing chain verification was not independently
//! performed. Only the exact pinned version and hashes are allowed.
//! Image preparation does not write a device; the ROM plan explicitly
//! binds the LK_A scatter entry to a distinct staged image.
use lpmbox_core::{LpmError,Result};
use sha2::{Digest,Sha256};
use std::{fs,path::{Path,PathBuf}};

pub const MODEL:&str="TB336FU";
pub const VERSION:&str="ZUI_17.5.10.321_ST";
pub const CANDIDATE_NAME:&str=
    "TB336FU_ROW_17_5_10_321_LK_REGION_PATCH_EXPERIMENTAL_UNVERIFIED.img";
pub const EXPECTED_STOCK_SHA:&str=
    "d6d4a269d722c1994b4efedad34e1e953273c2a979368d288050bda7b8a3c068";
pub const ROW348_STOCK_SHA:&str=
    "763057baccfc7b18cb3ee4f8e108bca7a9b6a28ee8ccc0c700493e8243e98048";
const ROW348_STOCK_BYTES:usize=2_959_680;
pub const EXPECTED_TRIAL_SHA:&str=
    "61e80798dd4fe8cc490543b43ccc8dd009e4dd5e5ed677eb9cf754e801c2eae8";
const EXPECTED_STOCK_BYTES:usize=2_909_104;
const EXPECTED_TRIAL_BYTES:usize=2_909_184;
const PATCH_OFFSET:usize=0xaf7c;
const BEFORE:[u8;4]=[0xe8,0x01,0x00,0xb4];
const AFTER:[u8;4]=[0x0f,0x00,0x00,0x14];

#[derive(Debug,Clone)]
pub struct TrialReport {
    pub original_sha256:String,
    pub candidate_sha256:String,
    pub candidate_path:PathBuf,
    pub size_delta:usize,
    pub patch_offset:usize,
    pub staged_path:PathBuf,
    pub secure_boot_verified:bool,
    pub official_signed:bool,
}
fn invalid(msg:impl Into<String>)->LpmError{
    LpmError::InvalidFirmwareFolder(msg.into())
}
fn shahex(bytes:&[u8])->String{
    Sha256::digest(bytes).iter().map(|b|format!("{b:02x}")).collect()
}
fn find_trial_file(image_dir:&Path)->PathBuf{
    if let Some(path)=std::env::var_os("LPMBOX_TB336FU_ROW321_TRIAL_LK") {
        return PathBuf::from(path);
    }
    let local=image_dir.join(CANDIDATE_NAME);
    if local.is_file(){return local;}
    Path::new(r"G:\TB375FC, TB373FU\ROM\TB335FC, TB336FU")
        .join("TB336FU_ROW_271_321_EXPERIMENTAL_LK_PATCH_UNVERIFIED")
        .join(CANDIDATE_NAME)
}
/// Verify exact pinned inputs; stage a named research-only artifact in an
/// isolated output folder. No image is used by SPFT or linked into scatter.
pub fn inspect_and_stage_offline(
    image_dir:&Path,staging_dir:&Path
)->Result<TrialReport>{
    let original_path=image_dir.join("lk.img");
    let candidate_path=find_trial_file(image_dir);
    let original=fs::read(&original_path).map_err(|err|
        invalid(format!("ROW 321 원본 LK를 읽지 못했습니다: {} / {err}",
            original_path.display())))?;
    let candidate=fs::read(&candidate_path).map_err(|err|
        invalid(format!("실험용 LK 후보를 읽지 못했습니다: {} / {err}",
            candidate_path.display())))?;
    verify_trial_bytes(&original,&candidate)?;
    // Keep the output isolated and explicitly marked DO_NOT_FLASH so no
    // existing firmware scatter or old lk_a can silently pick it up.
    fs::create_dir_all(staging_dir)?;
    let dest=staging_dir.join("TB336FU_ROW321_LK_UNVERIFIED_DO_NOT_FLASH.img");
    if dest.is_file(){
        if shahex(&fs::read(&dest)?)!=EXPECTED_TRIAL_SHA{
            return Err(invalid(format!(
                "검증되지 않은 기존 스테이징 파일이 존재합니다. 덮어쓰지 않습니다: {}",
                dest.display()
            )));
        }
    }else{
        let mut output=fs::OpenOptions::new().write(true)
            .create_new(true).open(&dest)?;
        use std::io::Write;
        output.write_all(&candidate)?;
        output.sync_all()?;
    }
    if shahex(&fs::read(&dest)?)!=EXPECTED_TRIAL_SHA{
        return Err(invalid("스테이징된 실험용 LK의 SHA-256 검증 실패"));
    }
    Ok(TrialReport{
        original_sha256:EXPECTED_STOCK_SHA.into(),
        candidate_sha256:EXPECTED_TRIAL_SHA.into(),
        candidate_path,
        size_delta:EXPECTED_TRIAL_BYTES-EXPECTED_STOCK_BYTES,
        patch_offset:PATCH_OFFSET,
        staged_path:dest,
        secure_boot_verified:false,
        official_signed:false
    })
}
/// Verify hashes, exact instruction replacement and structural size.
pub fn verify_trial_bytes(original:&[u8],candidate:&[u8])->Result<()>{
    if original.len()!=EXPECTED_STOCK_BYTES ||
        shahex(original)!=EXPECTED_STOCK_SHA {
        return Err(invalid("ROW 321 원본 lk.img 크기/해시 불일치. 이 LK 후보를 다른 버전/모델에 적용할 수 없습니다."));
    }
    if candidate.len()!=EXPECTED_TRIAL_BYTES ||
        shahex(candidate)!=EXPECTED_TRIAL_SHA {
        return Err(invalid("실험용 ROW 321 LK 후보 크기/해시 불일치. 검증된 연구 후보와 다릅니다."));
    }
    if !original.starts_with(&[0x88,0x16,0x88,0x58]) ||
        !candidate.starts_with(&[0x88,0x16,0x88,0x58]) ||
        original[PATCH_OFFSET..PATCH_OFFSET+4]!=BEFORE ||
        candidate[PATCH_OFFSET..PATCH_OFFSET+4]!=AFTER {
        return Err(invalid("실험용 LK 헤더/분기 바이트가 연구 기록과 일치하지 않습니다."));
    }
    if candidate.len()-original.len()!=80 {
        return Err(invalid("연구용 LK 인증 영역 길이 변화가 80바이트가 아닙니다."));
    }
    // This proves identical pinned bytes, NOT that vendor secure boot
    // will accept the custom cert2 envelope or executable branch.
    Ok(())
}

// Embedded to avoid relying on research paths or a network connection.
const EMBEDDED_ROW321_LK:&[u8]=include_bytes!(
    "../assets/TB336FU_ROW_17_5_10_321_LK_REGION_PATCH_EXPERIMENTAL_UNVERIFIED.img"
);
pub(crate) fn user_boottested_tb336_lk_bytes()->Result<&'static [u8]>{
    if EMBEDDED_ROW321_LK.len()!=EXPECTED_TRIAL_BYTES
        || shahex(EMBEDDED_ROW321_LK)!=EXPECTED_TRIAL_SHA {
        return Err(invalid("내장 TB336FU ROW LK 후보 해시 검증 실패"));
    }
    Ok(EMBEDDED_ROW321_LK)
}
/// Isolated LK_A candidate used by the existing ROM pipeline's scatter
/// output for precisely TB336FU ROW 17.5.10.321. Original lk.img and lk_a
/// files remain intact and are never overwritten by this function.
pub const USER_VERIFIED_LK_FILENAME:&str=
    "LPMBOX_lk.img";
/// Read-only, exact-version LK source preflight. ROW321's user-tested
/// candidate is reused for ROW348 only under a pinned stock-image SHA.
/// This checks the selected ROM identity, NOT vendor Secure Boot acceptance.
pub fn validate_selected_tb336_row_lk(
    image_dir:&Path,version:Option<&str>
)->Result<String>{
    let original_path=image_dir.join("lk.img");
    let original=fs::read(&original_path).map_err(|e|invalid(format!(
        "TB336FU ROW 원본 lk.img 접근 실패: {} / {e}",
        original_path.display())))?;
    let actual=shahex(&original);
    match version {
        Some("ZUI_17.5.10.321_ST") =>{
            verify_trial_bytes(&original,EMBEDDED_ROW321_LK)?;
        }
        Some("ZUI_17.5.10.348_ST") =>{
            if original.len()!=ROW348_STOCK_BYTES || actual!=ROW348_STOCK_SHA
                || !original.starts_with(&[0x88,0x16,0x88,0x58])
                || EMBEDDED_ROW321_LK.len()!=EXPECTED_TRIAL_BYTES
                || shahex(EMBEDDED_ROW321_LK)!=EXPECTED_TRIAL_SHA {
                return Err(invalid(
                    "TB336FU ROW348 원본 LK 길이/해시 또는 공통 패치 SHA-256 불일치: 설치 전 중단합니다."
                ));
            }
        }
        _=>{
            // The LK candidate is the same user-supplied boot-reported
            // family image. Do not infer that every target version boots.
            // Validate the stock container before staging either way.
            if !original.starts_with(&[0x88,0x16,0x88,0x58])
                || original.len()<1024*1024 || original.len()>8*1024*1024
                || EMBEDDED_ROW321_LK.len()!=EXPECTED_TRIAL_BYTES
                || shahex(EMBEDDED_ROW321_LK)!=EXPECTED_TRIAL_SHA {
                return Err(invalid(
                    "TB336FU ROW 원본 LK 형식 또는 내장 공통 LK SHA-256 검증 실패"
                ));
            }
        }
    }
    Ok(actual)
}
pub fn stage_user_boottested_row321_lk(
    image_dir:&Path,version:Option<&str>
)->Result<TrialReport>{
    let original_sha=validate_selected_tb336_row_lk(image_dir,version)?;
    let dest=image_dir.join(USER_VERIFIED_LK_FILENAME);
    if dest.exists() {
        if !dest.is_file() || shahex(&fs::read(&dest)?)!=EXPECTED_TRIAL_SHA {
            return Err(invalid(format!(
                "패치 파일이 존재하지만 해시가 다릅니다. 원본 보호를 위해 덮어쓰지 않습니다: {}",
                dest.display()
            )));
        }
    }else{
        use std::io::Write;
        let mut out=fs::OpenOptions::new().write(true)
            .create_new(true).open(&dest)?;
        if let Err(e)=out.write_all(EMBEDDED_ROW321_LK)
            .and_then(|_|out.sync_all()) {
            let _=fs::remove_file(&dest);
            return Err(e.into());
        }
    }
    if shahex(&fs::read(&dest)?)!=EXPECTED_TRIAL_SHA {
        return Err(invalid("생성된 LK_A 패치 이미지의 최종 해시 검증 실패"));
    }
    Ok(TrialReport{
        original_sha256:original_sha,
        candidate_sha256:EXPECTED_TRIAL_SHA.into(),
        candidate_path:PathBuf::from("LPMBox bundled LK (pinned SHA-256)"),
        size_delta:EXPECTED_TRIAL_BYTES-EXPECTED_STOCK_BYTES,
        patch_offset:PATCH_OFFSET,
        staged_path:dest,
        secure_boot_verified:false,
        official_signed:false
    })
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn exact_pinned_binary_is_included_in_application(){
        assert_eq!(shahex(EMBEDDED_ROW321_LK),EXPECTED_TRIAL_SHA);
        assert_eq!(EMBEDDED_ROW321_LK.len(),EXPECTED_TRIAL_BYTES);
    }
    #[test]
    fn user_boottested_stage_preserves_stock_and_existing_lk_a_if_available(){
        use std::time::{SystemTime,UNIX_EPOCH};
        let original=PathBuf::from(r"G:\TB375FC, TB373FU\ROM\TB335FC, TB336FU\ROW")
            .join("TB336FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.321_ST_260804")
            .join("TB336FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.321_ST_260804")
            .join("image/lk.img");
        if !original.is_file(){return;}
        let unique=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let dir=std::env::temp_dir().join(format!("lpmbox_row321_patch_{}_{}",std::process::id(),unique));
        fs::create_dir_all(&dir).unwrap();
        fs::copy(&original,dir.join("lk.img")).unwrap();
        fs::write(dir.join("lk_a"),b"STOCK_PACKAGE_LK_A_UNCHANGED").unwrap();
        let report=stage_user_boottested_row321_lk(&dir,Some("ZUI_17.5.10.321_ST")).unwrap();
        assert_eq!(report.candidate_sha256,EXPECTED_TRIAL_SHA);
        assert_eq!(shahex(&fs::read(&report.staged_path).unwrap()),EXPECTED_TRIAL_SHA);
        assert_eq!(fs::read(dir.join("lk_a")).unwrap(),b"STOCK_PACKAGE_LK_A_UNCHANGED");
        assert_eq!(shahex(&fs::read(dir.join("lk.img")).unwrap()),EXPECTED_STOCK_SHA);
        assert_eq!(report.staged_path.file_name().unwrap(),USER_VERIFIED_LK_FILENAME);
        // Repeat preflight is idempotent and never rewrites the original.
        let again=stage_user_boottested_row321_lk(&dir,Some("ZUI_17.5.10.321_ST")).unwrap();
        assert_eq!(again.staged_path,report.staged_path);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn row348_reuses_pinned_candidate_but_rejects_lookalikes(){
        let dir=PathBuf::from(r"G:\TB375FC, TB373FU\ROM\TB335FC, TB336FU\ROW")
            .join("TB336FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.348_ST_260828")
            .join("image");
        if dir.join("lk.img").is_file(){
            assert_eq!(validate_selected_tb336_row_lk(
                &dir,Some("ZUI_17.5.10.348_ST")).unwrap(),ROW348_STOCK_SHA);
            assert!(validate_selected_tb336_row_lk(
                &dir,Some("ZUI_17.5.10.321_ST")).is_err());
            // Unknown ZUI17 version now accepts the pinned family LK if
            // the selected source still has a recognized LK container.
            assert_eq!(validate_selected_tb336_row_lk(
                &dir,Some("ZUI_17.5.10.349_ST")).unwrap(),ROW348_STOCK_SHA);
        }
    }
    #[test]
    fn rejects_malformed_sources(){
        let source=vec![0u8;EXPECTED_STOCK_BYTES];
        let trial=vec![0u8;EXPECTED_TRIAL_BYTES];
        assert!(verify_trial_bytes(&source,&trial).is_err());
        assert!(verify_trial_bytes(&source,&[]).is_err());
    }
    #[test]
    fn offline_candidate_is_explicitly_unverified(){
        assert!(CANDIDATE_NAME.contains("UNVERIFIED"));
        assert_eq!(EXPECTED_TRIAL_BYTES-EXPECTED_STOCK_BYTES,80);
        assert_eq!(PATCH_OFFSET,0xaf7c);
    }
    #[test]
    fn staging_creates_isolated_copy_without_modifying_original(){
        use std::time::{SystemTime,UNIX_EPOCH};
        let base=PathBuf::from(r"G:\TB375FC, TB373FU\ROM\TB335FC, TB336FU\ROW")
            .join("TB336FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.321_ST_260804")
            .join("TB336FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.321_ST_260804")
            .join("image");
        if !base.join("lk.img").is_file()||!find_trial_file(&base).is_file(){
            return;
        }
        let before=shahex(&fs::read(base.join("lk.img")).unwrap());
        let nonce=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let stage=std::env::temp_dir().join(format!("lpmbox_tb336_trial_{}_{}",
            std::process::id(),nonce));
        let result=inspect_and_stage_offline(&base,&stage).unwrap();
        assert_eq!(result.original_sha256,EXPECTED_STOCK_SHA);
        assert_eq!(result.candidate_sha256,EXPECTED_TRIAL_SHA);
        assert!(!result.secure_boot_verified);
        assert!(!result.official_signed);
        assert!(result.staged_path.starts_with(&stage));
        assert_eq!(shahex(&fs::read(&result.staged_path).unwrap()),EXPECTED_TRIAL_SHA);
        assert_eq!(shahex(&fs::read(base.join("lk.img")).unwrap()),before);
        fs::remove_dir_all(&stage).unwrap();
    }
    #[test]
    fn user_supplied_fixture_if_available(){
        let base=PathBuf::from(r"G:\TB375FC, TB373FU\ROM\TB335FC, TB336FU\ROW")
            .join("TB336FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.321_ST_260804")
            .join("TB336FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.321_ST_260804")
            .join("image");
        let candidate=find_trial_file(&base);
        if base.join("lk.img").is_file()&&candidate.is_file(){
            verify_trial_bytes(&fs::read(base.join("lk.img")).unwrap(),
                &fs::read(candidate).unwrap()).unwrap();
        }
    }
}
