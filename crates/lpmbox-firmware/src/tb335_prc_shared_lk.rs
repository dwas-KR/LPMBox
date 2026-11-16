//! TB335FC PRC > ZUXOS_1.1.10.299: use the SHA-pinned TB336FU
//! ROW321/ROW348 user-boot-tested LK in the user's confirmed cross-region
//! configuration. This only stages a separate LK; it never signs or modifies
//! the original firmware, and hardware boot is not exercised here.
use crate::experimental_lk_trial;
use crate::lk_model_policy::{self,LkRequirement};
use lpmbox_core::{InstallMode,LpmError,Result,RomRegion};
use sha2::{Digest,Sha256};
use std::{fs,io::Write,path::{Path,PathBuf}};

pub const OUTPUT_FILE:&str="LPMBOX_lk.img";
pub const SHARED_LK_SHA256:&str=
    "61e80798dd4fe8cc490543b43ccc8dd009e4dd5e5ed677eb9cf754e801c2eae8";
pub const PRC_386_SOURCE_SHA256:&str=
    "6cdcda72dd34c494aa483793da09ae95c3e9faa31611c34d5662c5b1d5d360ee";
pub const PRC_522_SOURCE_SHA256:&str=
    "b21a09ee6abf78de2b957f8cc6dbd7891b43d1004e587c0f18f5c1937d3607ce";
pub const PRC_321_SOURCE_SHA256:&str=
    "d6e02b322f282b17d8989f7eb3c7e1519468c4ea0b004140e35e1708837fc40d";
const CANDIDATE_BYTES:usize=2_909_184;

fn invalid(message:impl Into<String>)->LpmError {
    LpmError::InvalidFirmwareFolder(message.into())
}
fn hex(bytes:&[u8])->String {
    Sha256::digest(bytes).iter().map(|b|format!("{b:02x}")).collect()
}
#[derive(Debug,Clone)]
pub struct PrcSharedReport {
    pub original_sha256:String,
    pub source_version:Option<String>,
    pub known_stock_hash:bool,
    pub result_sha256:String,
    pub staged_path:PathBuf,
}
/// File-only LK candidate validation, used in GUI before backup/reboot.
/// For *every* TB335FC PRC ROM strictly above ZUXOS 1.1.10.299, select
/// precisely the same user's boot-tested LK bytes, regardless of the
/// source ROM's original LK size/hash/internal section layout.
/// The original LK fingerprint is diagnostic only, NEVER a version gate.
/// Hardware flashing/boot validation remains outside this function.
pub fn preflight_prc_shared_lk(
    image_dir:&Path,
    version:Option<&str>
)->Result<PrcSharedReport>{
    if lk_model_policy::requirement("TB335FC",RomRegion::Prc,
        version,InstallMode::ConvertWipe)?
        !=LkRequirement::Tb335PrcSharedUserBoottested {
        return Err(invalid("TB335FC PRC 1.1.10.299 이하 또는 모델/버전 확인 실패: 공통 LK를 적용하지 않습니다."));
    }
    // User policy: ALL higher PRC versions share one LK. Never reject
    // an otherwise recognized TB335FC PRC build based on the source
    // lk.img fingerprint or a version-dependent signed section layout.
    // The source remains unchanged; fingerprint is informational only.
    let source=fs::read(image_dir.join("lk.img")).ok();
    let actual_sha=source.as_deref().map(hex)
        .unwrap_or_else(||"NOT_AVAILABLE".to_string());
    let recognized=match version {
        Some("ZUXOS_1.1.10.386_ST")=>
            source.as_ref().is_some_and(|s|
                s.len()==2_912_192 && actual_sha==PRC_386_SOURCE_SHA256),
        Some("ZUXOS_1.1.10.522_ST")=>
            source.as_ref().is_some_and(|s|
                s.len()==2_912_192 && actual_sha==PRC_522_SOURCE_SHA256),
        Some("ZUXOS_1.5.10.321_ST")=>
            source.as_ref().is_some_and(|s|
                s.len()==3_414_336 && actual_sha==PRC_321_SOURCE_SHA256),
        _=>false
    };
    let bytes=experimental_lk_trial::user_boottested_tb336_lk_bytes()?;
    if bytes.len()!=CANDIDATE_BYTES || hex(bytes)!=SHARED_LK_SHA256{
        return Err(invalid("내장된 공통 TB336FU LK 후보의 길이 또는 SHA-256 불일치"));
    }
    let dest=image_dir.join(OUTPUT_FILE);
    if dest.exists() && (!dest.is_file()
        || hex(&fs::read(&dest)?)!=SHARED_LK_SHA256) {
        return Err(invalid(format!(
            "이미 존재하는 {}가 실기기 부팅 확인 LK와 다릅니다. 덮어쓰지 않습니다.",
            dest.display()
        )));
    }
    Ok(PrcSharedReport{
        original_sha256:actual_sha,
        source_version:version.map(str::to_string),
        known_stock_hash:recognized,
        result_sha256:SHARED_LK_SHA256.to_string(),
        staged_path:dest
    })
}
/// Create LPMBOX_lk.img only after the file-only preflight succeeds.
pub fn stage_prc_shared_lk(
    image_dir:&Path,version:Option<&str>
)->Result<PrcSharedReport>{
    let report=preflight_prc_shared_lk(image_dir,version)?;
    if !report.staged_path.is_file(){
        let candidate=experimental_lk_trial::user_boottested_tb336_lk_bytes()?;
        let mut output=fs::OpenOptions::new().create_new(true).write(true)
            .open(&report.staged_path)?;
        if let Err(e)=output.write_all(candidate).and_then(|_|output.sync_all()){
            drop(output);
            let _=fs::remove_file(&report.staged_path);
            return Err(e.into());
        }
    }
    if hex(&fs::read(&report.staged_path)?)!=SHARED_LK_SHA256{
        return Err(invalid("LPMBOX_lk.img 생성 후 SHA-256 검증 실패"));
    }
    Ok(report)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn candidate_is_the_exact_embedded_user_reported_booted_image(){
        let candidate=experimental_lk_trial::user_boottested_tb336_lk_bytes().unwrap();
        assert_eq!(candidate.len(),CANDIDATE_BYTES);
        assert_eq!(hex(candidate),SHARED_LK_SHA256);
    }
    #[test]
    fn all_higher_prc_versions_use_exact_shared_lk_even_when_source_varies(){
        use std::time::{SystemTime,UNIX_EPOCH};
        let dir=std::env::temp_dir().join(format!(
            "lpm_prc_shared_negative_{}_{}",std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("lk.img"),b"not-a-valid-lk").unwrap();
        for version in [
            "ZUXOS_1.1.10.300_ST","ZUXOS_1.1.10.301_ST",
            "ZUXOS_1.1.10.386_ST","ZUXOS_1.1.10.522_ST",
            "ZUXOS_1.5.10.321_ST","ZUXOS_1.6.10.999_ST",
            "ZUX_2.0.10.123_ST"
        ]{
            let outcome=preflight_prc_shared_lk(&dir,Some(version)).unwrap();
            assert_eq!(outcome.result_sha256,SHARED_LK_SHA256);
            assert_eq!(outcome.original_sha256,hex(b"not-a-valid-lk"));
            let staged=stage_prc_shared_lk(&dir,Some(version)).unwrap();
            assert_eq!(hex(&fs::read(&staged.staged_path).unwrap()),SHARED_LK_SHA256);
        }
        assert!(preflight_prc_shared_lk(&dir,Some("ZUXOS_1.1.10.299_ST")).is_err());
        assert!(preflight_prc_shared_lk(&dir,Some("ZUXOS_1.1.10.298_ST")).is_err());
        fs::write(dir.join(OUTPUT_FILE),b"incorrect-LK").unwrap();
        assert!(preflight_prc_shared_lk(&dir,Some("ZUXOS_1.6.10.999_ST")).is_err());
        assert_eq!(fs::read(dir.join(OUTPUT_FILE)).unwrap(),b"incorrect-LK");
        fs::remove_file(dir.join(OUTPUT_FILE)).unwrap();
        fs::remove_file(dir.join("lk.img")).unwrap();
        assert_eq!(preflight_prc_shared_lk(&dir,Some("ZUXOS_3.0.10.001_ST")).unwrap().original_sha256,
            "NOT_AVAILABLE");
        assert!(!dir.join(OUTPUT_FILE).exists());
        fs::remove_dir_all(dir).unwrap();
    }
}
