//! Offline hash/format simulation for MediaTek LK certificate sections.
//!
//! IMPORTANT: this is NOT the manufacturer's Download Agent, ROM verifier,
//! a crypto signature verifier, or proof that a modified LK can boot.
//! It never writes devices, images, scatter XMLs, or any other files.
use lpmbox_core::{LpmError,Result};
use sha2::{Digest,Sha256};
use std::{fs,path::Path};
const HEADER_BYTES:usize=512;
const MAGIC:&[u8;4]=&[0x88,0x16,0x88,0x58];
const OID_HEADER:&[u8;12]=&[0x06,0x07,0x60,0x86,0x76,0x93,0x16,0x02,0x04,0x03,0x21,0x00];
const OID_DATA:&[u8;12]=&[0x06,0x07,0x60,0x86,0x76,0x93,0x16,0x02,0x01,0x03,0x21,0x00];

fn invalid(message:impl Into<String>)->LpmError{
    LpmError::InvalidFirmwareFolder(message.into())
}
fn aligned(v:usize)->Result<usize>{
    v.checked_add(15).map(|x|x&!15).ok_or_else(||invalid("LK 정렬 오버플로"))
}
fn sha(bytes:&[u8])->[u8;32]{
    Sha256::digest(bytes).into()
}
fn shahex(bytes:&[u8])->String{
    Sha256::digest(bytes).iter().map(|b|format!("{b:02x}")).collect()
}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum CandidateFormat{StockDer,Override,Wrap,Unknown}
impl CandidateFormat{
    pub fn label(self)->&'static str{
        match self{
            Self::StockDer=>"STOCK_DER",Self::Override=>"OVERRIDE_RESEARCH",
            Self::Wrap=>"WRAP_RESEARCH",Self::Unknown=>"UNRECOGNIZED",
        }
    }
}
#[derive(Debug,Clone)]
struct Section{
    name:String,offset:usize,data:usize,len:usize,next:usize
}
#[derive(Debug,Clone)]
pub struct PartitionAuthSimulation{
    pub name:String,
    pub original_section_offset:usize,
    pub cert2_size:usize,
    pub format:CandidateFormat,
    pub data_hash:String,
    pub embedded_header_sha_match:bool,
    pub embedded_data_sha_match:bool,
    pub original_cert_header_sha_match:bool,
    pub original_cert_data_sha_match:bool,
    pub candidate_cert_header_sha_match:Option<bool>,
    pub candidate_cert_data_sha_match:Option<bool>,
    pub original_cert_der_present:bool,
}
#[derive(Debug,Clone)]
pub struct LkAuthSimulation{
    pub image_sha256:String,
    pub image_bytes:usize,
    pub parsed_section_count:usize,
    pub partitions:Vec<PartitionAuthSimulation>,
    pub all_embedded_hashes_match:bool,
    pub cryptographic_signature_verified:bool,
    pub device_signature_authorized:bool,
}
fn parse_sections(raw:&[u8])->Result<Vec<Section>>{
    let mut pos=0usize;
    let mut out=Vec::new();
    while pos<raw.len(){
        let header_end=pos.checked_add(HEADER_BYTES)
            .ok_or_else(||invalid("MTK 헤더 경계 오버플로"))?;
        if header_end>raw.len()||&raw[pos..pos+4]!=MAGIC{
            // MTK layout often ends with 0 padding; any nonzero is suspect.
            if raw[pos..].iter().all(|&b|b==0){break;}
            return Err(invalid(format!(
                "MTK LK 섹션 파싱 실패: offset=0x{pos:X}에서 헤더 누락"
            )));
        }
        let size=u32::from_le_bytes(raw[pos+4..pos+8].try_into().unwrap()) as usize;
        if size==0{return Err(invalid("LK 섹션 크기 0"));}
        let name_end=raw[pos+8..pos+40].iter().position(|b|*b==0).unwrap_or(32);
        let name=std::str::from_utf8(&raw[pos+8..pos+8+name_end])
            .map_err(|_|invalid("MTK LK 비표준 섹션 이름"))?.to_string();
        let end=header_end.checked_add(size).ok_or_else(||invalid("MTK 데이터 길이 오버플로"))?;
        let next=aligned(end)?;
        if next>raw.len(){return Err(invalid("MTK LK 섹션이 파일을 초과합니다."));}
        out.push(Section{name,offset:pos,data:header_end,len:size,next});
        pos=next;
        if out.len()>100{return Err(invalid("MTK 섹션 개수 초과"));}
    }
    if out.len()<3{return Err(invalid("MTK LK 인증 섹션 3개 미만"));}
    Ok(out)
}
fn embedded_sha(cert:&[u8],needle:&[u8;12])->Option<[u8;32]>{
    let mut locations=cert.windows(needle.len()).enumerate()
        .filter_map(|(idx,b)|if b==needle{Some(idx)}else{None});
    let first=locations.next()?;
    if locations.next().is_some(){return None;}
    let start=first+needle.len();
    let end=start.checked_add(32)?;
    cert.get(start..end)?.try_into().ok()
}
fn cert_hash_flags(cert:&[u8],h:&[u8;32],d:&[u8;32])->(bool,bool){
    (
        embedded_sha(cert,OID_HEADER).is_some_and(|x|x==*h),
        embedded_sha(cert,OID_DATA).is_some_and(|x|x==*d),
    )
}
fn unwrap_wrap(cert2:&[u8])->Option<(&[u8],&[u8])>{
    if cert2.first().copied()!=Some(0x03){return None;}
    let head=*cert2.get(1)?;
    let (bit_len,head_len)=if head&0x80==0{
        (head as usize,2usize)
    }else{
        let n=(head&0x7f) as usize;
        if n==0||n>4{return None;}
        let end=2+n;
        let mut len=0usize;
        for x in cert2.get(2..end)?{
            len=len.checked_mul(256)?.checked_add(*x as usize)?;
        }
        (len,end)
    };
    if bit_len<2{return None;}
    let body=cert2.get(head_len..head_len.checked_add(bit_len)?)?;
    if body[0]!=0{return None;} // DER BIT STRING unused bits must be zero
    let old=&body[1..];
    if old.first().copied()!=Some(0x30){return None;}
    let forged=cert2.get(head_len+bit_len..)?;
    if forged.first().copied()!=Some(0x30){return None;}
    Some((old,forged))
}
fn classify(cert:&[u8])->CandidateFormat{
    if cert.first()==Some(&0x30){CandidateFormat::StockDer}
    else if cert.starts_with(&[0xa0,0x58]){CandidateFormat::Override}
    else if unwrap_wrap(cert).is_some(){CandidateFormat::Wrap}
    else{CandidateFormat::Unknown}
}
fn analyze_one(raw:&[u8],section:&Section,cert_section:&Section)->Result<PartitionAuthSimulation>{
    let code=&raw[section.data..section.data+section.len];
    let mut code_with_padding=code.to_vec();
    code_with_padding.resize(aligned(code.len())?,0);
    let data_sha=sha(&code_with_padding);
    let header_sha=sha(&raw[section.offset..section.offset+HEADER_BYTES]);
    let cert=&raw[cert_section.data..cert_section.data+cert_section.len];
    let format=classify(cert);
    let (orig,selected)=match format{
        CandidateFormat::StockDer=>(cert,cert),
        CandidateFormat::Override=>{
            // The [0] prefix contains the newly calculated hashes. The
            // untouched original DER begins directly after 90 bytes.
            let signed=cert.get(90..).unwrap_or(&[]);
            (signed,cert.get(..90).unwrap_or(&[]))
        }
        CandidateFormat::Wrap=>{
            unwrap_wrap(cert).unwrap_or((&[],&[]))
        }
        CandidateFormat::Unknown=>(&[][..],&[][..]),
    };
    let (orig_h,orig_d)=cert_hash_flags(orig,&header_sha,&data_sha);
    let (h,d)=cert_hash_flags(selected,&header_sha,&data_sha);
    let candidate_cert_header_sha_match=if format==CandidateFormat::StockDer{None}else{Some(h)};
    let candidate_cert_data_sha_match=if format==CandidateFormat::StockDer{None}else{Some(d)};
    Ok(PartitionAuthSimulation{
        name:section.name.clone(),
        original_section_offset:section.offset,
        cert2_size:cert.len(),
        format,
        data_hash:shahex(&code_with_padding),
        embedded_header_sha_match:h,
        embedded_data_sha_match:d,
        original_cert_header_sha_match:orig_h,
        original_cert_data_sha_match:orig_d,
        candidate_cert_header_sha_match,
        candidate_cert_data_sha_match,
        original_cert_der_present:orig.first()==Some(&0x30),
    })
}
/// Only compares SHA-256 digests and the layout of the first cert2 in each
/// three-section group. It does *not* validate RSA, the OEM root chain,
/// board lifecycle, Download Agent policy, anti-rollback or boot behavior.
pub fn simulate_signed_lk_bytes(raw:&[u8])->Result<LkAuthSimulation>{
    let entries=parse_sections(raw)?;
    let mut statuses=Vec::new();
    for (i,part) in entries.iter().enumerate(){
        if !matches!(part.name.as_str(),"lk"|"bl2_ext"|"aee"|"lk_main_dtb"|"lk_dtbo"){
            continue;
        }
        let cert1=entries.get(i+1).ok_or_else(||invalid("LK cert1 누락"))?;
        let cert2=entries.get(i+2).ok_or_else(||invalid("LK cert2 누락"))?;
        if cert1.name!="cert1"||cert2.name!="cert2"{
            return Err(invalid(format!("LK {} 인증서 순서가 올바르지 않습니다.",part.name)));
        }
        statuses.push(analyze_one(raw,part,cert2)?);
    }
    if statuses.len()!=5{return Err(invalid(format!(
        "LK 코드 파티션 5개 예상, 확인: {}",statuses.len()
    )));}
    let all_hashes=statuses.iter().all(|s|
        s.embedded_header_sha_match && s.embedded_data_sha_match
    );
    Ok(LkAuthSimulation{
        image_sha256:shahex(raw),image_bytes:raw.len(),
        parsed_section_count:entries.len(),partitions:statuses,
        all_embedded_hashes_match:all_hashes,
        cryptographic_signature_verified:false,
        device_signature_authorized:false,
    })
}
pub fn simulate_signed_lk_file(path:&Path)->Result<LkAuthSimulation>{
    simulate_signed_lk_bytes(&fs::read(path)?)
}
/// Simulate only the AArch64 branch instruction at a pinned candidate
/// offset. CBZ X8 with +0x3C would jump only if X8==0, whereas B +0x3C
/// jumps regardless. This is *not* proof that X8 is a region-code check.
#[derive(Debug,Clone)]
pub struct BranchPathSimulation{
    pub offset:usize,
    pub jump_target:usize,
    pub original_zero_next_pc:usize,
    pub original_nonzero_next_pc:usize,
    pub patched_zero_next_pc:usize,
    pub patched_nonzero_next_pc:usize,
}
pub fn simulate_exact_arm64_branch_pair(
    stock:&[u8],candidate:&[u8],offset:usize
)->Result<BranchPathSimulation>{
    let end=offset.checked_add(4).ok_or_else(||invalid("분기 오프셋 오버플로"))?;
    let before=stock.get(offset..end)
        .ok_or_else(||invalid("순정 LK 분기 오프셋 초과"))?;
    let after=candidate.get(offset..end)
        .ok_or_else(||invalid("후보 LK 분기 오프셋 초과"))?;
    if before!=[0xe8,0x01,0x00,0xb4] || after!=[0x0f,0x00,0x00,0x14]{
        return Err(invalid(
            "ARM64 CBZ X8,+0x3C → B +0x3C 코드 패턴이 예상과 다릅니다."
        ));
    }
    // AArch64 PC-relative branch offsets are multiples of 4 bytes.
    // These exact opcodes both use displacement +0x3C.
    let target=offset.checked_add(0x3c).ok_or_else(||invalid("PC 계산 오버플로"))?;
    let next=offset.checked_add(4).ok_or_else(||invalid("PC 계산 오버플로"))?;
    Ok(BranchPathSimulation{
        offset,jump_target:target,
        original_zero_next_pc:target,
        original_nonzero_next_pc:next,
        patched_zero_next_pc:target,
        patched_nonzero_next_pc:target,
    })
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::lk_region_family_research::{
        build_offline_candidate_with_mode,ResearchCertMode,ResearchModel
    };
    fn path(model:ResearchModel)->std::path::PathBuf{
        let base=Path::new(r"G:\TB375FC, TB373FU\ROM\TB361FU, TB365FC");
        match model{
            ResearchModel::Tb361FuRow375=>base.join(
                r"TB361FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.375_ST_260822\TB361FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.375_ST_260822\image\lk.img"),
            ResearchModel::Tb365FcPrc122=>base.join(
                r"TB365FC_PRC_ZUXOS_1.1.10.122\image\lk.img"),
        }
    }
    #[test]
    fn original_signed_parts_all_five_hashes_match(){
        for m in [ResearchModel::Tb361FuRow375,ResearchModel::Tb365FcPrc122]{
            if !path(m).exists(){continue;}
            let sim=simulate_signed_lk_file(&path(m)).unwrap();
            assert_eq!(sim.parsed_section_count,15);
            assert!(sim.all_embedded_hashes_match);
            assert_eq!(sim.partitions.len(),5);
            assert!(sim.partitions.iter().all(|x|x.format==CandidateFormat::StockDer));
            assert!(sim.partitions.iter().all(|x|x.original_cert_data_sha_match));
            assert!(!sim.cryptographic_signature_verified);
            assert!(!sim.device_signature_authorized);
        }
    }
    #[test]
    fn candidate_formats_and_original_hash_delta_are_explicit(){
        for m in [ResearchModel::Tb361FuRow375,ResearchModel::Tb365FcPrc122]{
            if !path(m).exists(){continue;}
            let stock=fs::read(path(m)).unwrap();
            for (mode,format) in [
                (ResearchCertMode::Override,CandidateFormat::Override),
                (ResearchCertMode::Wrap,CandidateFormat::Wrap)
            ]{
                let (patched,_)=build_offline_candidate_with_mode(&stock,m,mode).unwrap();
                let sim=simulate_signed_lk_bytes(&patched).unwrap();
                assert_eq!(sim.partitions[0].format,format);
                assert_eq!(sim.partitions[0].cert2_size,
                    if mode==ResearchCertMode::Wrap{1995}else{1085});
                assert!(sim.partitions[0].original_cert_der_present);
                assert!(sim.partitions[0].candidate_cert_data_sha_match.unwrap());
                assert!(!sim.partitions[0].original_cert_data_sha_match);
                assert!(sim.all_embedded_hashes_match);
                assert!(sim.partitions[1..].iter().all(|x|
                    x.original_cert_data_sha_match
                    && x.format==CandidateFormat::StockDer
                ));
                assert!(!sim.device_signature_authorized);
            }
        }
    }
    #[test]
    fn tampered_code_breaks_embedded_hash_match(){
        for m in [ResearchModel::Tb361FuRow375,ResearchModel::Tb365FcPrc122]{
            if !path(m).exists(){continue;}
            let mut stock=fs::read(path(m)).unwrap();
            stock[4096]^=0x55;
            let sim=simulate_signed_lk_bytes(&stock).unwrap();
            assert!(!sim.all_embedded_hashes_match);
            assert!(!sim.partitions[0].embedded_data_sha_match);
        }
    }
    #[test]
    fn arm64_branch_path_simulation_is_consistent_for_both_models(){
        for model in [ResearchModel::Tb361FuRow375,ResearchModel::Tb365FcPrc122]{
            if !path(model).exists(){continue;}
            let original=fs::read(path(model)).unwrap();
            for mode in [ResearchCertMode::Override,ResearchCertMode::Wrap]{
                let (candidate,_)=build_offline_candidate_with_mode(
                    &original,model,mode
                ).unwrap();
                let offset=model.first_branch_offset();
                let paths=simulate_exact_arm64_branch_pair(
                    &original,&candidate,offset
                ).unwrap();
                assert_eq!(paths.jump_target,offset+0x3c);
                assert_eq!(paths.original_zero_next_pc,offset+0x3c);
                assert_eq!(paths.original_nonzero_next_pc,offset+4);
                assert_eq!(paths.patched_zero_next_pc,offset+0x3c);
                assert_eq!(paths.patched_nonzero_next_pc,offset+0x3c);
                let mut changed=candidate.clone();
                changed[offset]=0;
                assert!(simulate_exact_arm64_branch_pair(
                    &original,&changed,offset
                ).is_err());
            }
        }
    }
    #[test]
    fn malformed_sections_fail_closed(){
        assert!(simulate_signed_lk_bytes(&[0x01;1024]).is_err());
        let mut garbage=vec![0u8;1024];
        garbage[0..4].copy_from_slice(MAGIC);
        garbage[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(simulate_signed_lk_bytes(&garbage).is_err());
    }
}
