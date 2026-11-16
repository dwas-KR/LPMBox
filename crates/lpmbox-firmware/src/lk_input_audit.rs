//! Read-only, exact-source LK provenance diagnostics for TB335FC PRC321.
//!
//! Important: Pinned hashes and recognizable signed-section layout ARE NOT
//! proof of an MTK Secure Boot accepted cross-region LK patch. Never make
//! this routine change LK bytes, Flash Plans or SPFT partitions.
use lpmbox_core::{LpmError,Result};
use sha2::{Digest,Sha256};
use std::{fs,path::Path};

pub const MODEL:&str="TB335FC";
pub const VERSION:&str="ZUXOS_1.5.10.321_ST";
pub const PRC321_STOCK_LENGTH:usize=3_414_336;
pub const PRC321_STOCK_SHA256:&str=
    "d6e02b322f282b17d8989f7eb3c7e1519468c4ea0b004140e35e1708837fc40d";
pub const PRC386_STOCK_LENGTH:usize=2_912_192;
pub const PRC386_STOCK_SHA256:&str=
    "6cdcda72dd34c494aa483793da09ae95c3e9faa31611c34d5662c5b1d5d360ee";
const PADDED_LK_A_LENGTH:usize=16*1024*1024;
const MTK_MAGIC:[u8;4]=[0x88,0x16,0x88,0x58];
fn invalid(message:impl Into<String>)->LpmError{
    LpmError::InvalidFirmwareFolder(message.into())
}
fn sha_hex(data:&[u8])->String{
    Sha256::digest(data).iter().map(|b|format!("{b:02x}")).collect()
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum LkAProvenance{
    Missing,
    SelectedPrcStockPadded,
    KnownPrc386StockPadded,
    Unexpected{bytes:usize,sha256:String},
}

#[derive(Debug,Clone)]
pub struct Prc321Audit{
    pub selected_stock_sha256:String,
    pub staged_lk_a:LkAProvenance,
    pub signed_patch_verified:bool,
}
fn is_mtk_header(b:&[u8],offset:usize,size:usize,name:&[u8])->bool{
    if b.len()<offset+512 || name.len()>32{return false}
    b[offset..offset+4]==MTK_MAGIC
        && u32::from_le_bytes(b[offset+4..offset+8].try_into().unwrap()) as usize==size
        && &b[offset+8..offset+8+name.len()]==name
}
pub fn audit_prc321_sources(image_dir:&Path)->Result<Prc321Audit>{
    let stock_path=image_dir.join("lk.img");
    let stock=fs::read(&stock_path).map_err(|e|invalid(format!(
        "TB335FC PRC321 원본 lk.img 읽기 실패 (읽기 전용 검사): {} / {e}",
        stock_path.display()
    )))?;
    let actual=sha_hex(&stock);
    // Avoid treating a repacked, overwritten or wrong-family firmware as
    // the verified input. Code and cert-envelope size are intentionally pinned.
    if stock.len()!=PRC321_STOCK_LENGTH || actual!=PRC321_STOCK_SHA256
        || !is_mtk_header(&stock,0,949_296,b"lk")
        || !is_mtk_header(&stock,0xe7e30,1_733,b"cert1")
        || !is_mtk_header(&stock,0xe8700,995,b"cert2")
        || !is_mtk_header(&stock,0xe8cf0,1_038_096,b"bl2_ext") {
        return Err(invalid(format!(
            "TB335FC PRC 1.5.10.321 순정 LK가 알려진 원본과 다릅니다: bytes={}, SHA-256={actual}. \
             예상 bytes={PRC321_STOCK_LENGTH}, SHA-256={PRC321_STOCK_SHA256}. \
             다른 ROM 파일이나 기존 패치 파일을 원본으로 사용하지 마세요.",
            stock.len()
        )));
    }
    // Allow only the exact SHA-pinned ROW348 LK the user reported booting
    // on TB335FC PRC321; a stale or edited LK must never be silently reused.
    let common_lk=image_dir.join("LPMBOX_lk.img");
    if common_lk.exists(){
        if !common_lk.is_file()
            || sha_hex(&fs::read(&common_lk)?)
                !=crate::tb335_prc_shared_lk::SHARED_LK_SHA256 {
            return Err(invalid(
                "TB335FC PRC321 LPMBOX_lk.img의 SHA-256이 실기기 부팅 확인 파일과 다릅니다. 기존 파일을 자동 승인하거나 덮어쓰지 않습니다."
            ));
        }
    }
    let staged_path=image_dir.join("lk_a");
    let staged_lk_a=if !staged_path.exists(){
        LkAProvenance::Missing
    }else if !staged_path.is_file(){
        return Err(invalid("TB335FC PRC321 lk_a 경로가 일반 파일이 아닙니다."));
    }else{
        let staged=fs::read(&staged_path).map_err(|e|invalid(format!(
            "TB335FC PRC321 lk_a 읽기 실패: {e}"
        )))?;
        if staged.len()==PADDED_LK_A_LENGTH
            && staged[..stock.len()]==stock
            && staged[stock.len()..].iter().all(|b|*b==0){
            LkAProvenance::SelectedPrcStockPadded
        }else if staged.len()==PADDED_LK_A_LENGTH
            && sha_hex(&staged[..PRC386_STOCK_LENGTH])==PRC386_STOCK_SHA256
            && staged[PRC386_STOCK_LENGTH..].iter().all(|b|*b==0){
            LkAProvenance::KnownPrc386StockPadded
        }else{
            LkAProvenance::Unexpected{
                bytes:staged.len(),sha256:sha_hex(&staged)
            }
        }
    };
    Ok(Prc321Audit{
        selected_stock_sha256:actual,
        staged_lk_a,
        signed_patch_verified:false
    })
}
pub fn log_prc321_audit<F>(
    image_dir:&Path,on_log:&mut F
)->Result<()>
where F:FnMut(String){
    let report=audit_prc321_sources(image_dir)?;
    on_log(format!(
        "[LK 출처 검사] TB335FC PRC321 순정 lk.img 인증서 구조·고정 SHA-256 일치: {} (읽기 전용)",
        report.selected_stock_sha256
    ));
    match report.staged_lk_a{
        LkAProvenance::Missing=>on_log(
            "[LK 출처 안내] 원래 lk_a가 없습니다. 별도로 생성하는 TB336FU 공통 LPMBOX_lk.img를 LK_A에 연결합니다.".into()
        ),
        LkAProvenance::SelectedPrcStockPadded=>on_log(
            "[LK 출처 안내] lk_a는 선택한 PRC321 순정 LK를 16MiB 0 패딩한 파일입니다. 지역 변경용 패치는 아니며 자동 플래싱 승인 대상도 아닙니다.".into()
        ),
        LkAProvenance::KnownPrc386StockPadded=>on_log(
            "[LK 출처 경고] lk_a의 유효 데이터는 PRC 1.1.10.386 순정 LK와 일치합니다. \
             선택한 PRC 1.5.10.321의 lk.img와 다른 이미지이므로 사용하지 않습니다. 공통 LPMBOX_lk.img만 LK_A 플래싱 대상으로 설정합니다.".into()
        ),
        LkAProvenance::Unexpected{bytes,sha256}=>on_log(format!(
            "[LK 출처 경고] 현재 lk_a는 확인되지 않은 이미지입니다 ({}바이트, SHA-256={sha256}). PRC321 패치로 자동 사용하지 않습니다.",
            bytes
        ))
    }
    on_log(
        "[LK 정책] TB335FC PRC321은 1.1.10.299 기준 초과입니다. \
         사용자 실기기에서 정상 부팅을 확인한 TB336FU ROW321/348 공통 LPMBOX_lk.img를 LK_A에 적용합니다. \
         이미 존재하는 순정 lk.img 및 다른 버전의 lk_a는 변경하지 않으며 LK_B는 비활성화합니다.".into()
    );
    Ok(())
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn rejects_wrong_input_without_writing(){
        let dir=std::env::temp_dir().join(format!(
            "lpmbox_prc321_provenance_invalid_{}",std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("lk.img"),b"wrong-LK").unwrap();
        assert!(audit_prc321_sources(&dir).is_err());
        assert!(!dir.join("LPMBOX_lk.img").exists());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn actual_prc321_source_when_present_has_prc386_padded_lk_a(){
        let dir=Path::new(r"G:\TB375FC, TB373FU\ROM\TB335FC, TB336FU\PRC")
            .join("TB335FC_ZUXOS_1.5.10.321_Tool")
            .join("TB335FC_ZUXOS_1.5.10.321_Tool")
            .join("image");
        if !dir.is_dir(){return}
        let report=audit_prc321_sources(&dir).unwrap();
        assert_eq!(report.selected_stock_sha256,PRC321_STOCK_SHA256);
        assert_eq!(report.staged_lk_a,LkAProvenance::KnownPrc386StockPadded);
        assert!(!report.signed_patch_verified);
    }
}
