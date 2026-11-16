//! Exact-source-hash TB361FU ROW 17.5.10.375 LK candidate.
//! Implementation reproduces the TB336FU research certificate-envelope
//! transformation on a separately pinned TB361FU source. This is an
//! *experimental* candidate, NOT a verified OEM-signed or boot-tested image.
//! No commands are sent to the device in this module.
use lpmbox_core::{LpmError, Result};
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::{Path, PathBuf}};

pub const MODEL: &str = "TB361FU";
pub const VERSION: &str = "ZUI_17.5.10.375_ST";
pub const OUTPUT_FILE: &str = crate::lk_patch::LPMBOX_PATCHED_LK_FILENAME;
pub const ORIGINAL_SHA256: &str =
    "12e45b037ed1591af3a560940bce5e59834ef9c213307b1b92f30dac9ad08a39";
pub const GENERATED_SHA256: &str =
    "af5cc578e866ae0924ead6fa371a01faeb49c32a245bbb66155e2d4d0ad823d1";
const ORIGINAL_BYTES:usize = 3_516_272;
const GENERATED_BYTES:usize = ORIGINAL_BYTES + 80;
const FIRST_CODE_BYTES:usize = 943_976;
const BRANCH_OFFSET:usize = 0xb708;
const BRANCH_BEFORE:[u8;4] = [0xe8,0x01,0x00,0xb4];
const BRANCH_AFTER:[u8;4] = [0x0f,0x00,0x00,0x14];
const SECOND_BRANCH_TARGET:[u8;4] = [0xe0,0x03,0x14,0xaa];
const MTK_MAGIC:[u8;4] = [0x88,0x16,0x88,0x58];
const HDR_OID:[u8;12] = [0x06,0x07,0x60,0x86,0x76,0x93,0x16,0x02,0x04,0x03,0x21,0x00];
const DATA_OID:[u8;12] = [0x06,0x07,0x60,0x86,0x76,0x93,0x16,0x02,0x01,0x03,0x21,0x00];

#[derive(Debug,Clone)]
pub struct Tb361Row375PatchReport {
    pub source_sha256:String,
    pub output_sha256:String,
    pub output_path:PathBuf,
    pub patch_offset:usize,
    pub output_size:usize,
}
fn invalid(s:impl Into<String>)->LpmError {
    LpmError::InvalidFirmwareFolder(s.into())
}
fn hash(data:&[u8])->[u8;32]{
    let mut out=[0u8;32];
    out.copy_from_slice(&Sha256::digest(data));
    out
}
fn hex(data:&[u8])->String{
    data.iter().map(|b|format!("{b:02x}")).collect()
}
fn align16(x:usize)->Result<usize>{
    x.checked_add(15).map(|y|y&!15).ok_or_else(||invalid("LK 정렬 오버플로"))
}
fn read_u32(data:&[u8],offset:usize)->Result<usize>{
    if offset.checked_add(4).map_or(true,|e|e>data.len()){
        return Err(invalid("LK 패키지 헤더 범위를 초과했습니다."));
    }
    Ok(u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize)
}
fn header(data:&[u8],offset:usize,name:&[u8],length:usize)->Result<usize>{
    if offset.checked_add(512).map_or(true,|e|e>data.len())
        || data[offset..offset+4]!=MTK_MAGIC
        || !data[offset+8..offset+8+name.len()].eq(name)
        || read_u32(data,offset+4)?!=length {
        return Err(invalid(format!(
            "TB361FU ROW375 LK 섹션 검증 실패: {} @ 0x{offset:X}",
            String::from_utf8_lossy(name)
        )));
    }
    Ok(offset+512)
}
/// Reconstructs one exact candidate without changing source LK or scatter.
/// Structural integrity is checked, *not* secure-boot authenticity.
pub fn generate_exact_row375_patch(original:&[u8])->Result<Vec<u8>>{
    if original.len()!=ORIGINAL_BYTES || hex(&hash(original))!=ORIGINAL_SHA256{
        return Err(invalid(
            "TB361FU ROW 17.5.10.375 순정 lk.img 길이/해시 불일치: 버전별 후보를 재사용하지 않습니다."
        ));
    }
    let start=header(original,0,b"lk",FIRST_CODE_BYTES)?;
    if original[BRANCH_OFFSET..BRANCH_OFFSET+4]!=BRANCH_BEFORE
        || original[BRANCH_OFFSET+0x3c..BRANCH_OFFSET+0x40]!=SECOND_BRANCH_TARGET {
        return Err(invalid("ROW375 분기 바이트와 대상 명령어 구조가 예상과 다릅니다."));
    }
    let c1=align16(start+FIRST_CODE_BYTES)?;
    header(original,c1,b"cert1",1733)?;
    let c2=align16(c1+512+1733)?;
    let c2_payload=header(original,c2,b"cert2",995)?;
    if original[c2_payload..c2_payload+2]!=[0x30,0x82]{
        return Err(invalid("ROW375 첫 cert2의 ASN.1 DER 시작값 불일치"));
    }
    let old_next=align16(c2_payload+995)?;
    if old_next+4>original.len() || original[old_next..old_next+4]!=MTK_MAGIC{
        return Err(invalid("ROW375 다음 MTK image 섹션 경계가 올바르지 않습니다."));
    }
    let mut patched=original.to_vec();
    patched[BRANCH_OFFSET..BRANCH_OFFSET+4].copy_from_slice(&BRANCH_AFTER);
    let mut padded=patched[start..start+FIRST_CODE_BYTES].to_vec();
    padded.resize(align16(FIRST_CODE_BYTES)?,0);
    let header_hash=hash(&patched[..512]);
    let data_hash=hash(&padded);
    let mut prefix=Vec::with_capacity(90);
    prefix.extend_from_slice(&[0xa0,0x58]);
    prefix.extend_from_slice(&HDR_OID);
    prefix.extend_from_slice(&header_hash);
    prefix.extend_from_slice(&DATA_OID);
    prefix.extend_from_slice(&data_hash);
    if prefix.len()!=90{
        return Err(invalid("TB361FU 패치 프리픽스 길이가 90바이트가 아닙니다."));
    }
    let mut output=Vec::with_capacity(GENERATED_BYTES);
    output.extend_from_slice(&patched[..c2_payload]);
    output.extend_from_slice(&prefix);
    output.extend_from_slice(&original[c2_payload..c2_payload+995]);
    output.resize(output.len()+align16(1085)?-1085,0);
    output.extend_from_slice(&patched[old_next..]);
    if output.len()!=GENERATED_BYTES {
        return Err(invalid("TB361FU ROW375 LK 생성 후 길이 불일치"));
    }
    output[c2+4..c2+8].copy_from_slice(&1085u32.to_le_bytes());
    header(&output,c2,b"cert2",1085)?;
    if output[old_next+80..]!=original[old_next..]{
        return Err(invalid("LK의 다른 이미지 섹션이 예상과 다르게 변경되었습니다."));
    }
    if hex(&hash(&output))!=GENERATED_SHA256{
        return Err(invalid("TB361FU ROW375 LK 출력 SHA-256 불일치"));
    }
    Ok(output)
}
/// Prepares only LPMBOX_lk.img. Source signed lk.img and vendor scatter
/// remain untouched. Existing unexpected output is never overwritten.
pub fn stage_exact_row375_patch(image_dir:&Path)->Result<Tb361Row375PatchReport>{
    let source=image_dir.join("lk.img");
    let bytes=fs::read(&source).map_err(|e|invalid(format!(
        "TB361FU ROW375 원본 LK 접근 실패: {} / {e}",source.display()
    )))?;
    let generated=generate_exact_row375_patch(&bytes)?;
    let output_path=image_dir.join(OUTPUT_FILE);
    if output_path.exists(){
        if !output_path.is_file()
            || hex(&hash(&fs::read(&output_path)?))!=GENERATED_SHA256{
            return Err(invalid(format!(
                "기존 LPMBOX_lk.img가 예상과 달라 덮어쓰지 않습니다: {}",
                output_path.display()
            )));
        }
    } else {
        let mut file=fs::OpenOptions::new().write(true)
            .create_new(true).open(&output_path)?;
        if let Err(e)=file.write_all(&generated).and_then(|_|file.sync_all()){
            let _=fs::remove_file(&output_path);
            return Err(e.into());
        }
    }
    if hex(&hash(&fs::read(&output_path)?))!=GENERATED_SHA256{
        return Err(invalid("생성된 TB361FU ROW375 LPMBOX_lk.img 최종 해시 오류"));
    }
    Ok(Tb361Row375PatchReport{
        source_sha256:ORIGINAL_SHA256.into(),
        output_sha256:GENERATED_SHA256.into(),
        output_path,
        patch_offset:BRANCH_OFFSET,
        output_size:GENERATED_BYTES
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unrecognized_inputs(){
        assert!(generate_exact_row375_patch(&[0u8;256]).is_err());
        assert!(generate_exact_row375_patch(&vec![0u8;ORIGINAL_BYTES]).is_err());
    }
    #[test]
    fn staging_is_idempotent_and_does_not_change_stock_lk(){
        use std::time::{SystemTime,UNIX_EPOCH};
        let rom=Path::new(r"G:\TB375FC, TB373FU\ROM\TB361FU, TB365FC")
            .join("TB361FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.375_ST_260822")
            .join("TB361FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.375_ST_260822")
            .join("image").join("lk.img");
        if !rom.is_file(){return;}
        let epoch=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let dir=std::env::temp_dir()
            .join(format!("lpmbox_tb361_375_{}_{}",std::process::id(),epoch));
        fs::create_dir(&dir).unwrap();
        fs::copy(&rom,dir.join("lk.img")).unwrap();
        let before=fs::read(dir.join("lk.img")).unwrap();
        let report=stage_exact_row375_patch(&dir).unwrap();
        assert_eq!(report.source_sha256,ORIGINAL_SHA256);
        assert_eq!(report.output_sha256,GENERATED_SHA256);
        assert_eq!(report.output_path,dir.join(OUTPUT_FILE));
        assert_eq!(fs::read(dir.join("lk.img")).unwrap(),before);
        assert_eq!(stage_exact_row375_patch(&dir).unwrap().output_sha256,GENERATED_SHA256);
        fs::write(dir.join(OUTPUT_FILE),b"invalid").unwrap();
        assert!(stage_exact_row375_patch(&dir).is_err());
        assert_eq!(fs::read(dir.join(OUTPUT_FILE)).unwrap(),b"invalid");
        assert_eq!(fs::read(dir.join("lk.img")).unwrap(),before);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn pinned_row375_fixture_when_present(){
        let dir=Path::new(r"G:\TB375FC, TB373FU\ROM\TB361FU, TB365FC")
            .join("TB361FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.375_ST_260822")
            .join("TB361FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.375_ST_260822")
            .join("image");
        let source=dir.join("lk.img");
        if source.is_file(){
            let input=fs::read(&source).unwrap();
            let output=generate_exact_row375_patch(&input).unwrap();
            assert_eq!(hex(&hash(&input)),ORIGINAL_SHA256);
            assert_eq!(hex(&hash(&output)),GENERATED_SHA256);
            assert_eq!(output[BRANCH_OFFSET..BRANCH_OFFSET+4],BRANCH_AFTER);
            assert_eq!(input[BRANCH_OFFSET..BRANCH_OFFSET+4],BRANCH_BEFORE);
            assert_eq!(output.len(),GENERATED_BYTES);
        }
    }
}
