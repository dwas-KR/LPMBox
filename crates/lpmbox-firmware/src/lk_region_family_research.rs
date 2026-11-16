//! Offline-only research of the TB361FU/TB365FC MT6835 LK certificate envelope.
//!
//! This module NEVER installs an image, modifies a firmware tree, or bypasses
//! the normal FlashPlan policy. Its output MUST NOT be selected for SPFT.
//! The known TB361FU ROW375 test candidate was rejected by the device's DA
//! with "Signature(Entity) invalid", so structural reconstruction alone is
//! NOT sufficient evidence for bootloader authenticity/flash permission.
use lpmbox_core::{LpmError, Result};
use sha2::{Digest, Sha256};
use std::{fs,io::Write,path::{Path,PathBuf}};

const MTK_MAGIC:[u8;4]=[0x88,0x16,0x88,0x58];
const OLD_BRANCH:[u8;4]=[0xe8,0x01,0x00,0xb4];
const RESEARCH_BRANCH:[u8;4]=[0x0f,0x00,0x00,0x14];
const BRANCH_CHECK:[u8;4]=[0xe0,0x03,0x14,0xaa];
const CERT1_EXPECTED:&str="034134b5af7d1c620381e669f1e897fd9a5afa670f5b730361a8f0e2c7e52411";
const OID_HEADER:[u8;12]=[0x06,0x07,0x60,0x86,0x76,0x93,0x16,0x02,0x04,0x03,0x21,0x00];
const OID_DATA:[u8;12]=[0x06,0x07,0x60,0x86,0x76,0x93,0x16,0x02,0x01,0x03,0x21,0x00];

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ResearchModel{Tb361FuRow375,Tb365FcPrc122}
impl ResearchModel{
    pub fn from_argument(argument:&str)->Result<Self>{
        match argument.trim().to_ascii_uppercase().as_str(){
            "TB361FU"|"TB361FU_ROW375"=>Ok(Self::Tb361FuRow375),
            "TB365FC"|"TB365FC_PRC122"=>Ok(Self::Tb365FcPrc122),
            _=>Err(invalid("연구 대상은 TB361FU_ROW375 또는 TB365FC_PRC122만 허용합니다."))
        }
    }
    pub fn label(self)->&'static str{
        match self{Self::Tb361FuRow375=>"TB361FU_ROW375",Self::Tb365FcPrc122=>"TB365FC_PRC122"}
    }
    pub fn known_source_hash(self)->&'static str{
        match self{
            Self::Tb361FuRow375=>"12e45b037ed1591af3a560940bce5e59834ef9c213307b1b92f30dac9ad08a39",
            Self::Tb365FcPrc122=>"02f5dbc1065cd92ddfdc5f26f8d7734e1ab88c2f33a335df6bcde4bbaa08f64c"
        }
    }
    pub fn known_research_hash(self)->&'static str{
        match self{
            Self::Tb361FuRow375=>"af5cc578e866ae0924ead6fa371a01faeb49c32a245bbb66155e2d4d0ad823d1",
            Self::Tb365FcPrc122=>"66daf0c2f7e652cbed2782b70ab8190c57bb21dc1ae95fda41a778b3863d7159"
        }
    }
    pub fn stock_bytes(self)->usize{
        match self{Self::Tb361FuRow375=>3_516_272,Self::Tb365FcPrc122=>2_897_568}
    }
    pub fn first_branch_offset(self)->usize{
        match self{Self::Tb361FuRow375=>0xb708,Self::Tb365FcPrc122=>0xb4a0}
    }
    pub fn signature_status(self)->&'static str{
        match self{
            Self::Tb361FuRow375=>"SPFT lk_a Signature(Entity) invalid (previous user log)",
            Self::Tb365FcPrc122=>"UNTESTED: no signed LK acceptance/boot result"
        }
    }
}
/// Offline certificate-envelope experiment only; neither variant is
/// proof of a vendor-authenticated or bootable LK image.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ResearchCertMode { Override, Wrap }
impl ResearchCertMode{
    pub fn from_argument(s:&str)->Result<Self>{
        match s.trim().to_ascii_lowercase().as_str(){
            "override"=>Ok(Self::Override),"wrap"=>Ok(Self::Wrap),
            _=>Err(invalid("인증 연구 방식은 override 또는 wrap만 지원합니다."))
        }
    }
    pub fn label(self)->&'static str{
        match self{Self::Override=>"OVERRIDE",Self::Wrap=>"WRAP"}
    }
    pub fn pinned_hash(self,model:ResearchModel)->&'static str{
        match self{
            Self::Override=>model.known_research_hash(),
            Self::Wrap=>match model{
                ResearchModel::Tb361FuRow375=>"7d95271e2179e6f1b8b7096f92bb790767bdacafb7b21ff744231bbb423670d0",
                ResearchModel::Tb365FcPrc122=>"2f482fafc07d999ffc1e965cbed489318ede8a2bc798a8663e29f08ad4701ccb",
            }
        }
    }
}
#[derive(Debug,Clone)]
pub struct ResearchReport{
    pub model:ResearchModel,
    pub cert_mode:ResearchCertMode,
    pub input_sha256:String,
    pub output_sha256:String,
    pub output_bytes:usize,
    pub cert1_sha256:String,
    pub first_branch_offset:usize,
    pub output_path:Option<PathBuf>,
    pub secure_boot_accepted:bool,
}
fn invalid(msg:impl Into<String>)->LpmError{LpmError::InvalidFirmwareFolder(msg.into())}
fn sha(bytes:&[u8])->String{
    Sha256::digest(bytes).iter().map(|b|format!("{b:02x}")).collect()
}
fn align16(v:usize)->Result<usize>{
    v.checked_add(15).map(|x|x&!15)
        .ok_or_else(||invalid("LK 섹션 정렬 오버플로"))
}
#[derive(Debug)]
struct Section{data:usize,len:usize,next:usize}
fn section(buf:&[u8],at:usize,name:&[u8])->Result<Section>{
    let h=at.checked_add(512).ok_or_else(||invalid("MTK LK 헤더 범위 초과"))?;
    if h>buf.len()||buf[at..at+4]!=MTK_MAGIC{return Err(invalid(format!(
        "MTK LK {} 헤더 Magic 불일치 @ 0x{:X}",
        String::from_utf8_lossy(name),at
    )));}
    let size=u32::from_le_bytes(buf[at+4..at+8].try_into().unwrap()) as usize;
    if &buf[at+8..at+8+name.len()]!=name || size==0 {
        return Err(invalid(format!("MTK LK {} 섹션 헤더 불일치",String::from_utf8_lossy(name))));
    }
    let end=h.checked_add(size).ok_or_else(||invalid("LK 길이 오버플로"))?;
    let next=align16(end)?;
    if next>buf.len(){return Err(invalid("LK 섹션 길이가 원본 파일을 초과했습니다."));}
    Ok(Section{data:h,len:size,next})
}
fn read_signed_layout(
    stock:&[u8],model:ResearchModel
)->Result<(usize,Section,Section,Section)>{
    if stock.len()!=model.stock_bytes()||sha(stock)!=model.known_source_hash(){
        return Err(invalid(format!("{} 원본 LK 길이/SHA-256 불일치: 미확인 버전에 연구 패치를 재사용하지 않습니다.",model.label())));
    }
    let image=section(stock,0,b"lk")?;
    let cert1=section(stock,image.next,b"cert1")?;
    let cert2=section(stock,cert1.next,b"cert2")?;
    if cert1.len!=1733||sha(&stock[cert1.data..cert1.data+cert1.len])!=CERT1_EXPECTED{
        return Err(invalid(format!("{} cert1 인증 서명 자료가 기존 연구 샘플과 다릅니다.",model.label())));
    }
    if cert2.len!=995||stock[cert2.data..cert2.data+2]!=[0x30,0x82]{
        return Err(invalid(format!("{} cert2 원본 DER 구조 불일치",model.label())));
    }
    let branch=model.first_branch_offset();
    if branch<image.data||branch+64>image.data+image.len
        || stock[branch..branch+4]!=OLD_BRANCH
        || stock[branch+0x3c..branch+0x40]!=BRANCH_CHECK{
        return Err(invalid(format!("{} 리전 관련 분기 바이트 구조 불일치",model.label())));
    }
    // The remaining signed images must be kept byte-for-byte intact.
    section(stock,cert2.next,b"bl2_ext")?;
    Ok((branch,image,cert1,cert2))
}

fn der_len(n:usize)->Result<Vec<u8>>{
    if n<128 {return Ok(vec![n as u8]);}
    let mut work=n;
    let mut big=Vec::new();
    while work!=0{
        big.push((work&0xff) as u8);
        work>>=8;
    }
    big.reverse();
    if big.len()>4{return Err(invalid("연구용 DER 길이 허용 범위 초과"));}
    let mut result=vec![0x80|(big.len() as u8)];
    result.extend_from_slice(&big);
    Ok(result)
}
/// On a COPY of the original cert2, substitute the two MTK custom
/// SHA-256 extension hashes, preserving all other DER bytes. The RSA
/// certificate signature is not re-signed or claimed valid.
fn forged_copy_of_cert2(
    original:&[u8], header_hash:&[u8], data_hash:&[u8]
)->Result<Vec<u8>>{
    if header_hash.len()!=32 || data_hash.len()!=32{
        return Err(invalid("인증 연구의 SHA-256 길이가 올바르지 않습니다."));
    }
    let mut copy=original.to_vec();
    for (oid,bytes) in [(OID_HEADER.as_slice(),header_hash),
                        (OID_DATA.as_slice(),data_hash)]{
        // OID + BIT STRING type/length/unused-bits; exactly one of each.
        let positions=copy.windows(oid.len())
            .enumerate()
            .filter_map(|(i,w)|if w==oid{Some(i)}else{None})
            .collect::<Vec<_>>();
        if positions.len()!=1{
            return Err(invalid("cert2 MTK 인증 해시 OID가 정확히 한 번 나타나지 않습니다."));
        }
        let pos=positions[0]+oid.len();
        let end=pos.checked_add(32).ok_or_else(||
            invalid("cert2 해시 위치 오버플로"))?;
        if end>copy.len(){return Err(invalid("cert2 해시가 DER 길이를 초과합니다."));}
        copy[pos..end].copy_from_slice(bytes);
    }
    Ok(copy)
}
/// Preserve the original signed certificate as a DER BIT STRING, followed
/// by a separate copy whose custom extension hashes match the modified
/// image. This is a research reproduction of the public "wrap" concept,
/// not an OEM signature and not confirmed accepted by any TB361/365 DA.
fn wrap_cert2(original:&[u8],header_hash:&[u8],data_hash:&[u8])
->Result<Vec<u8>>{
    let altered=forged_copy_of_cert2(original,header_hash,data_hash)?;
    let mut result=vec![0x03]; // DER BIT STRING tag
    result.extend_from_slice(&der_len(original.len()+1)?);
    result.push(0); // zero unused bits
    result.extend_from_slice(original);
    result.extend_from_slice(&altered);
    Ok(result)
}
/// Compatibility wrapper: returns the same exact Override result as before.
pub fn build_offline_candidate(stock:&[u8],model:ResearchModel)->Result<(Vec<u8>,ResearchReport)>{
    build_offline_candidate_with_mode(stock,model,ResearchCertMode::Override)
}
/// Build a SHA-pinned offline candidate using the chosen certificate-envelope
/// research encoding; never used by LPMBOX's device write or FlashPlan.
pub fn build_offline_candidate_with_mode(
    stock:&[u8],model:ResearchModel,mode:ResearchCertMode
)->Result<(Vec<u8>,ResearchReport)>{
    let (branch,image,cert1,cert2)=read_signed_layout(stock,model)?;
    let mut modified=stock.to_vec();
    modified[branch..branch+4].copy_from_slice(&RESEARCH_BRANCH);
    let code=&modified[image.data..image.data+image.len];
    let mut aligned_code=code.to_vec();
    aligned_code.resize(align16(aligned_code.len())?,0);
    let mut override_block=Vec::with_capacity(90);
    override_block.extend_from_slice(&[0xa0,0x58]);
    override_block.extend_from_slice(&OID_HEADER);
    override_block.extend_from_slice(&Sha256::digest(&modified[..512]));
    override_block.extend_from_slice(&OID_DATA);
    override_block.extend_from_slice(&Sha256::digest(&aligned_code));
    if override_block.len()!=90{return Err(invalid("연구용 인증 오버라이드 길이 오류"));}
    let signed_original=&stock[cert2.data..cert2.data+cert2.len];
    let mut payload=match mode{
        ResearchCertMode::Override=>{
            let mut new_cert=override_block;
            new_cert.extend_from_slice(signed_original);
            if new_cert.len()!=1085{
                return Err(invalid("override 인증 영역 길이 오류"));
            }
            new_cert
        }
        ResearchCertMode::Wrap=>wrap_cert2(
            signed_original,
            &Sha256::digest(&modified[..512]),
            &Sha256::digest(&aligned_code),
        )?,
    };
    let cert2_length=payload.len();
    let out_cert2_bytes=align16(cert2_length)?;
    payload.resize(out_cert2_bytes,0);
    let original_padded_cert=cert2.next-cert2.data;
    let added=out_cert2_bytes.checked_sub(original_padded_cert).ok_or_else(||
        invalid("연구 인증 영역 크기가 기존보다 작아졌습니다."))?;
    let mut out=Vec::with_capacity(stock.len()+added);
    out.extend_from_slice(&modified[..cert2.data]);
    out.extend_from_slice(&payload);
    out.extend_from_slice(&modified[cert2.next..]);
    out[cert2.data-512+4..cert2.data-512+8]
        .copy_from_slice(&(cert2_length as u32).to_le_bytes());
    if out.len()!=stock.len()+added
       || out[cert2.next+added..]!=stock[cert2.next..]{
        return Err(invalid("연구용 LK 재구성 범위가 원본과 다릅니다."));
    }
    let result_sha=sha(&out);
    if result_sha!=mode.pinned_hash(model){
        return Err(invalid(format!(
            "연구용 {} {} 결과 해시 고정값 불일치: {}",
            model.label(),mode.label(),result_sha
        )));
    }
    Ok((out,ResearchReport{
        model,
        cert_mode:mode,
        input_sha256:sha(stock),
        output_sha256:result_sha,
        output_bytes:stock.len()+added,
        cert1_sha256:sha(&stock[cert1.data..cert1.data+cert1.len]),
        first_branch_offset:branch,
        output_path:None,
        secure_boot_accepted:false
    }))
}

/// The only output path is a dedicated DO_NOT_FLASH folder. A target under a
/// firmware "image" or "ROM" tree is prohibited. No .xml/scatter is written.
pub fn stage_research_only(
    source:&Path,output_root:&Path,model:ResearchModel
)->Result<ResearchReport>{
    stage_research_with_mode(source,output_root,model,ResearchCertMode::Override)
}
pub fn stage_research_with_mode(
    source:&Path,output_root:&Path,model:ResearchModel,mode:ResearchCertMode
)->Result<ResearchReport>{
    if !source.is_file()||source.file_name().and_then(|x|x.to_str())!=Some("lk.img"){
        return Err(invalid("서명된 순정 lk.img 파일 경로만 입력하세요."));
    }
    if output_root.components().any(|c|{
        let s=c.as_os_str().to_string_lossy();
        s.eq_ignore_ascii_case("image")||s.eq_ignore_ascii_case("ROM")
    }){
        return Err(invalid("연구 출력은 ROM/image 내부가 아닌 별도 디렉터리로 지정하세요."));
    }
    let stock=fs::read(source)?;
    let (candidate,mut report)=build_offline_candidate_with_mode(&stock,model,mode)?;
    let folder=match mode{
        ResearchCertMode::Override=>output_root.join(format!("{}_DO_NOT_FLASH",model.label())),
        ResearchCertMode::Wrap=>output_root.join(format!("{}_WRAP_DO_NOT_FLASH",model.label())),
    };
    fs::create_dir_all(&folder)?;
    let output=folder.join("LPMBOX_lk.img");
    if output.exists(){
        if !output.is_file()||sha(&fs::read(&output)?)!=report.output_sha256{
            return Err(invalid("기존 연구 출력 LK 해시가 다릅니다. 파일을 덮어쓰지 않습니다."));
        }
    }else{
        let mut writer=fs::OpenOptions::new().write(true).create_new(true).open(&output)?;
        if let Err(e)=writer.write_all(&candidate).and_then(|_|writer.sync_all()){
            let _=fs::remove_file(&output);
            return Err(e.into());
        }
    }
    if sha(&fs::read(&output)?)!=report.output_sha256{
        return Err(invalid("저장된 연구 파일 해시 검증 실패"));
    }
    report.output_path=Some(output);
    Ok(report)
}

/// Validate source structure and report expected output; this never writes.
pub fn inspect_stock_only(source:&Path,model:ResearchModel)->Result<ResearchReport>{
    let stock=fs::read(source)?;
    let (_,report)=build_offline_candidate(&stock,model)?;
    Ok(report)
}

#[cfg(test)]
mod tests{
    use super::*;
    fn fixture(model:ResearchModel)->PathBuf{
        let base=Path::new(r"G:\TB375FC, TB373FU\ROM\TB361FU, TB365FC");
        match model{
            ResearchModel::Tb361FuRow375=>base.join(
                r"TB361FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.375_ST_260822\TB361FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.375_ST_260822\image\lk.img"),
            ResearchModel::Tb365FcPrc122=>base.join(
                r"TB365FC_PRC_ZUXOS_1.1.10.122\image\lk.img"),
        }
    }
    #[test]
    fn rejects_unknown_or_modified_sources_without_writing(){
        for m in [ResearchModel::Tb361FuRow375,ResearchModel::Tb365FcPrc122]{
            assert!(build_offline_candidate(&[0u8;32],m).is_err());
            if let Some(p)=std::fs::read(fixture(m)).ok(){
                let mut altered=p;
                altered[m.first_branch_offset()]=0;
                assert!(build_offline_candidate(&altered,m).is_err());
            }
        }
    }
    #[test]
    fn pinned_samples_match_independent_reference_hashes(){
        for model in [ResearchModel::Tb361FuRow375,ResearchModel::Tb365FcPrc122]{
            if !fixture(model).is_file(){continue;}
            let data=fs::read(fixture(model)).unwrap();
            let (result,report)=build_offline_candidate(&data,model).unwrap();
            assert_eq!(report.output_sha256,model.known_research_hash());
            assert_eq!(result.len(),model.stock_bytes()+80);
            assert_eq!(&result[model.first_branch_offset()..model.first_branch_offset()+4],
                &RESEARCH_BRANCH);
            assert!(!report.secure_boot_accepted);
        }
    }
    #[test]
    fn wrap_pinned_matches_independently_calculated_reference_image(){
        for model in [ResearchModel::Tb361FuRow375,ResearchModel::Tb365FcPrc122]{
            let stock_file=fixture(model);
            if !stock_file.is_file(){continue;}
            let stock=fs::read(stock_file).unwrap();
            let (candidate,report)=build_offline_candidate_with_mode(
                &stock,model,ResearchCertMode::Wrap
            ).unwrap();
            assert_eq!(report.output_sha256,
                       ResearchCertMode::Wrap.pinned_hash(model));
            assert!(!report.secure_boot_accepted);
            assert_eq!(candidate.len(),stock.len()+992);
            assert_eq!(report.output_bytes,candidate.len());
            let (_branch,_img,_cert1,cert2)=read_signed_layout(&stock,model).unwrap();
            let size=u32::from_le_bytes(
                candidate[cert2.data-512+4..cert2.data-512+8].try_into().unwrap()
            ) as usize;
            assert_eq!(size,1995);
            let wrapped=&candidate[cert2.data..cert2.data+size];
            // 0x03 BIT STRING + DER short-long encoded length = 996.
            assert_eq!(&wrapped[..5],&[0x03,0x82,0x03,0xE4,0]);
            assert_eq!(
                &wrapped[5..5+995],
                &stock[cert2.data..cert2.data+995]
            );
            assert_ne!(
                &wrapped[5+995..],
                &stock[cert2.data..cert2.data+995]
            );
            assert_eq!(
                &candidate[cert2.next+992..],
                &stock[cert2.next..]
            );
        }
    }
    #[test]
    fn tb361_generated_candidate_matches_prior_failed_candidate(){
        let orig=fixture(ResearchModel::Tb361FuRow375);
        if !orig.is_file(){return;}
        let raw=fs::read(orig).unwrap();
        let (output,_)=build_offline_candidate(&raw,ResearchModel::Tb361FuRow375).unwrap();
        let old=crate::tb361fu_row375_lk::generate_exact_row375_patch(&raw).unwrap();
        assert_eq!(output,old);
    }
    #[test]
    fn cert1_is_identical_for_row375_and_prc122(){
        let a=fixture(ResearchModel::Tb361FuRow375);
        let b=fixture(ResearchModel::Tb365FcPrc122);
        if !a.is_file()||!b.is_file(){return;}
        for (model,path) in [(ResearchModel::Tb361FuRow375,a),
            (ResearchModel::Tb365FcPrc122,b)]{
            let raw=fs::read(path).unwrap();
            let (_,_,c1,_)=read_signed_layout(&raw,model).unwrap();
            assert_eq!(sha(&raw[c1.data..c1.data+c1.len]),CERT1_EXPECTED);
        }
    }
    #[test]
    fn output_is_isolated_idempotent_and_original_untouched(){
        use std::time::{SystemTime,UNIX_EPOCH};
        let original=fixture(ResearchModel::Tb365FcPrc122);
        if !original.is_file(){return;}
        let temp=std::env::temp_dir().join(format!(
            "lpmbox_region_{}_{}",std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&temp).unwrap();
        let stock=temp.join("lk.img");
        fs::copy(&original,&stock).unwrap();
        let baseline=fs::read(&stock).unwrap();
        let root=temp.join("isolated");
        let a=stage_research_only(&stock,&root,ResearchModel::Tb365FcPrc122).unwrap();
        let b=stage_research_only(&stock,&root,ResearchModel::Tb365FcPrc122).unwrap();
        let wrap=stage_research_with_mode(
            &stock,&root,ResearchModel::Tb365FcPrc122,
            ResearchCertMode::Wrap
        ).unwrap();
        assert!(wrap.output_path.unwrap().to_string_lossy().contains("WRAP_DO_NOT_FLASH"));
        assert_eq!(a.output_sha256,b.output_sha256);
        assert_eq!(fs::read(&stock).unwrap(),baseline);
        let output=a.output_path.unwrap();
        assert!(output.to_string_lossy().contains("DO_NOT_FLASH"));
        assert_eq!(output.file_name().unwrap(),"LPMBOX_lk.img");
        fs::write(&output,b"wrong content").unwrap();
        assert!(stage_research_only(&stock,&root,ResearchModel::Tb365FcPrc122).is_err());
        assert_eq!(fs::read(&output).unwrap(),b"wrong content");
        fs::remove_dir_all(temp).unwrap();
    }
}
