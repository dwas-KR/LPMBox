//! Real-firmware output integration tests: NO ADB/SPFT/device operations.
//! Only a minimal set of real firmware inputs are copied to a temporary
//! directory. Original ROM files and source scatter are never modified.
//! To use a different ROM directory set LPMBOX_REAL_ROM_ROOT.
//! Missing fixtures print SKIP for portable CI; the developer PC has them.
use lpmbox_core::InstallMode;
use lpmbox_firmware::flash_plan::prepare_flash_plan_with_log;
use roxmltree::Document;
use sha2::{Digest,Sha256};
use std::{fs,path::{Path,PathBuf},time::{SystemTime,UNIX_EPOCH},
          sync::atomic::{AtomicU64,Ordering}};
static IDS:AtomicU64=AtomicU64::new(0);
const LK:&str="LPMBOX_lk.img";

#[derive(Clone,Copy)]
struct Case{
    name:&'static str,folder:&'static str,model:&'static str,platform:&'static str,
    lk_sha:Option<&'static str>,b_enabled:bool,
}
const CASES:&[Case]=&[
 Case{name:"TB361FU ROW375",model:"TB361FU",platform:"MT6835",b_enabled:false,
 folder:r"TB361FU, TB365FC\TB361FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.375_ST_260822\TB361FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.375_ST_260822\image",
 lk_sha:Some("af5cc578e866ae0924ead6fa371a01faeb49c32a245bbb66155e2d4d0ad823d1")},
 Case{name:"TB336FU ROW321",model:"TB336FU",platform:"MT6835",b_enabled:false,
 folder:r"TB335FC, TB336FU\ROW\TB336FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.321_ST_260804\TB336FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.321_ST_260804\image",
 lk_sha:Some("61e80798dd4fe8cc490543b43ccc8dd009e4dd5e5ed677eb9cf754e801c2eae8")},
 Case{name:"TB373FU ROW084",model:"TB373FU",platform:"MT6897",b_enabled:true,
 folder:r"TB375FC, TB373FU\ROW\TB373FU_ROW_OPEN_USER_M21.814_A16_ZUI_17.5.10.084_ST_260515\TB373FU_ROW_OPEN_USER_M21.814_A16_ZUI_17.5.10.084_ST_260515\image",
 lk_sha:Some("f1e5b7f3a2cc9ead1c4fc2f57821e27a9ba5a49410cd7d91e1e9922087bbfa66")},
 Case{name:"TB375FC PRC103",model:"TB375FC",platform:"MT6897",b_enabled:true,
 folder:r"TB375FC, TB373FU\PRC\TB375FC_ZUX_1.5.10.103_Tool\TB375FC_ZUX_1.5.10.103_Tool\image",
 lk_sha:Some("f1e5b7f3a2cc9ead1c4fc2f57821e27a9ba5a49410cd7d91e1e9922087bbfa66")},
 Case{name:"TB365FC PRC122 baseline",model:"TB365FC",platform:"MT6835",b_enabled:false,
 folder:r"TB361FU, TB365FC\TB365FC_PRC_ZUXOS_1.1.10.122\image",lk_sha:None},
 Case{name:"TB373FU ROW095",model:"TB373FU",platform:"MT6897",b_enabled:true,
 folder:r"TB375FC, TB373FU\ROW\TB373FU_ROW_OPEN_USER_M21.814_A16_ZUI_17.5.10.095_ST_260709\TB373FU_ROW_OPEN_USER_M21.814_A16_ZUI_17.5.10.095_ST_260709\image",
 lk_sha:Some("f1e5b7f3a2cc9ead1c4fc2f57821e27a9ba5a49410cd7d91e1e9922087bbfa66")},
 Case{name:"TB373FU ROW107",model:"TB373FU",platform:"MT6897",b_enabled:true,
 folder:r"TB375FC, TB373FU\ROW\TB373FU_ROW_OPEN_USER_M21.814_A16_ZUI_17.5.10.107_ST_260908\TB373FU_ROW_OPEN_USER_M21.814_A16_ZUI_17.5.10.107_ST_260908\image",
 lk_sha:Some("f1e5b7f3a2cc9ead1c4fc2f57821e27a9ba5a49410cd7d91e1e9922087bbfa66")},
 Case{name:"TB336FU ROW348",model:"TB336FU",platform:"MT6835",b_enabled:false,
 folder:r"TB335FC, TB336FU\ROW\TB336FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.348_ST_260828\image",
 lk_sha:Some("61e80798dd4fe8cc490543b43ccc8dd009e4dd5e5ed677eb9cf754e801c2eae8")},
 Case{name:"TB335FC PRC321 common LK",model:"TB335FC",platform:"MT6835",b_enabled:false,
 folder:r"TB335FC, TB336FU\PRC\TB335FC_ZUXOS_1.5.10.321_Tool\TB335FC_ZUXOS_1.5.10.321_Tool\image",
 lk_sha:Some("61e80798dd4fe8cc490543b43ccc8dd009e4dd5e5ed677eb9cf754e801c2eae8")},
 Case{name:"TB335FC PRC299 baseline",model:"TB335FC",platform:"MT6835",b_enabled:false,
 folder:r"TB335FC, TB336FU\PRC\TB335FC_PRC_ZUXOS_1.1.10.299\image",
 lk_sha:None},
 Case{name:"TB335FC PRC386 common LK",model:"TB335FC",platform:"MT6835",b_enabled:false,
 folder:r"TB335FC, TB336FU\PRC\TB335FC_CN_OPEN_USER_M1317.3_V_ZUXOS_1.1.10.386_ST_250714\TB_335FC_ZUX_1.1.10.386\image",
 lk_sha:Some("61e80798dd4fe8cc490543b43ccc8dd009e4dd5e5ed677eb9cf754e801c2eae8")},
 Case{name:"TB335FC PRC522 common LK",model:"TB335FC",platform:"MT6835",b_enabled:false,
 folder:r"TB335FC, TB336FU\PRC\TB335FC_CN_OPEN_USER_M1317.3_V_ZUXOS_1.1.10.522_ST_251110\image",
 lk_sha:Some("61e80798dd4fe8cc490543b43ccc8dd009e4dd5e5ed677eb9cf754e801c2eae8")},
 Case{name:"TB336FU ROW271 family LK",model:"TB336FU",platform:"MT6835",b_enabled:false,
 folder:r"TB335FC, TB336FU\ROW\TB336FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.271_ST_260616\TB336FU_ROW_OPEN_USER_M1317.3_W_ZUI_17.5.10.271_ST_260616\image",
 lk_sha:Some("61e80798dd4fe8cc490543b43ccc8dd009e4dd5e5ed677eb9cf754e801c2eae8")},
 Case{name:"TB336FU ROW457 family LK",model:"TB336FU",platform:"MT6835",b_enabled:false,
 folder:r"TB335FC, TB336FU\ROW\TB336FU_ROW_OPEN_USER_M1317.3_V_ZUI_17.0.10.457_ST_250919\TB336FU_ROW_OPEN_USER_M1317.3_V_ZUI_17.0.10.457_ST_250919\image",
 lk_sha:Some("61e80798dd4fe8cc490543b43ccc8dd009e4dd5e5ed677eb9cf754e801c2eae8")},
];
fn rom()->PathBuf{
 std::env::var_os("LPMBOX_REAL_ROM_ROOT").map(PathBuf::from)
 .unwrap_or_else(||PathBuf::from(r"G:\TB375FC, TB373FU\ROM"))
}
fn sha(path:&Path)->String{
 let bytes=fs::read(path).unwrap();
 Sha256::digest(bytes).iter().map(|b|format!("{b:02x}")).collect()
}
struct Fixture{root:PathBuf,image:PathBuf}
impl Drop for Fixture{fn drop(&mut self){
 if let Err(e)=fs::remove_dir_all(&self.root){eprintln!("TEMP_CLEANUP_FAILED {e}");}
}}
fn make(case:Case)->Option<(Fixture,String,String,String)>{
 let input=rom().join(case.folder);
 if !input.is_dir(){
     eprintln!("REAL_FIRMWARE_INTEGRATION=SKIP {} source missing {}",case.name,input.display());
     return None
 }
 let now=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
 let root=std::env::temp_dir().join(format!("lpmbox_real_{}_{}_{}",
      case.model,now,IDS.fetch_add(1,Ordering::Relaxed)));
 let image=root.join("image");
 fs::create_dir_all(image.join("download_agent")).unwrap();
 let mut needed=vec!["lk.img".to_string(),"dtbo.img".into(),
   "vendor_boot-debug.img".into(),"download_agent/flash.xml".into(),
   "download_agent/DA_BR.bin".into()];
 for ext in ["xml","x"]{
    let name=format!("{}_Android_scatter.{ext}",case.platform);
    if input.join(&name).is_file(){needed.push(name);}
 }
 assert!(needed.iter().any(|n|n.contains("Android_scatter")),
   "{} no scatter fixture",case.name);
 for name in &needed{
    let source=input.join(name);
    assert!(source.is_file(),"{} missing {}",case.name,source.display());
    fs::copy(&source,image.join(name)).unwrap();
 }
 if case.platform=="MT6835"{
    // Most legacy test fixtures already have local A-only stock images.
    // PRC321 must retain its actual shipped 16MiB lk_a; it contains a
    // different (PRC386) LK and the production preflight must report it.
    if case.name=="TB335FC PRC321 common LK"{
        fs::copy(input.join("lk_a"),image.join("lk_a")).unwrap();
    }else{
        fs::copy(image.join("lk.img"),image.join("lk_a")).unwrap();
    }
    fs::copy(image.join("dtbo.img"),image.join("dtbo_a")).unwrap();
 }
 if case.model=="TB373FU"&&case.name!="TB373FU ROW084"{
    // Use the real ROW084 *stock* LK as the trusted patch reference.
    // The production code regenerates the pinned ROW084 patch for
    // ROW095/107, while the selected version's lk.img stays untouched.
    let stock_084=rom().join(
      r"TB375FC, TB373FU\ROW\TB373FU_ROW_OPEN_USER_M21.814_A16_ZUI_17.5.10.084_ST_260515\TB373FU_ROW_OPEN_USER_M21.814_A16_ZUI_17.5.10.084_ST_260515\image\lk.img");
    assert!(stock_084.is_file(),"ROW084 trusted LK source missing");
    fs::copy(stock_084,image.join("ROW_084_lk.img")).unwrap();
 }
 let original_lk=sha(&image.join("lk.img"));
 let original_flash=sha(&image.join("download_agent/flash.xml"));
 // ROW preparation intentionally removes a stale plaintext XML when encrypted .x exists.
 // Preserve/compare the selected .x source, not the deleted temp XML.
 let scatter=needed.iter().find(|n|n.ends_with("Android_scatter.x"))
    .or_else(||needed.iter().find(|n|n.contains("Android_scatter"))).unwrap();
 let original_scatter=sha(&image.join(scatter));
 Some((Fixture{root,image},original_lk,original_flash,original_scatter))
}
fn value<'a,'input>(node:roxmltree::Node<'a,'input>,tag:&str)->Option<&'a str>{
 node.children().find(|n|n.is_element()&&n.tag_name().name()==tag)
    .and_then(|n|n.text()).map(str::trim)
}
fn run(case:Case){
 let Some((fix,lk_before,flash_before,scatter_before))=make(case) else{return};
 let mut logs=Vec::<String>::new();
 let result=prepare_flash_plan_with_log(&fix.image,InstallMode::ConvertWipe,
    |s|logs.push(s));
 if case.model=="TB361FU" {
    let err=result.expect_err("known-bad TB361FU ROW375 LK must be rejected");
    let msg=err.to_string();
    assert!(msg.contains("Signature(Entity) invalid"),"error={msg}");
    assert!(!fix.image.join(LK).is_file(),"rejected LK must not be staged");
    assert!(!fix.image.join("download_agent/LPMBox_flash.xml").is_file(),
        "rejected LK must not generate executable download plan");
    assert_eq!(sha(&fix.image.join("lk.img")),lk_before);
    assert_eq!(sha(&fix.image.join("download_agent/flash.xml")),flash_before);
    println!("REAL_FIRMWARE_INTEGRATION=PASS {} EXPECTED_REJECT_BEFORE_WRITE: {msg}",case.name);
    return;
 }
 let output=result.unwrap_or_else(|e|panic!(
       "{} prepare_flash_plan failed: {e}\n{}",case.name,logs.join("\n")));
 assert_eq!(output.plan.model,case.model);
 assert_eq!(output.plan.platform,case.platform);
 let patched=fix.image.join(LK);
 if let Some(expected)=case.lk_sha{
    assert!(patched.is_file(),"{} patched image missing",case.name);
    assert_eq!(sha(&patched),expected,"{} patch output SHA",case.name);
 }else{
    assert!(!patched.exists(),"{} baseline should not patch LK",case.name);
 }
 let scatter=output.plan.work_scatter_xml.clone();
 let flash=output.plan.work_flash_xml.clone();
 assert!(scatter.is_file(),"{} missing generated scatter",case.name);
 assert!(flash.is_file(),"{} missing generated flash.xml",case.name);
 let scatter_text=fs::read_to_string(&scatter).unwrap();
 let doc=Document::parse(&scatter_text).unwrap();
 let mut lk_blocks=0;
 for block in doc.descendants().filter(|n|n.is_element()
     &&n.tag_name().name()=="partition_index"){
   let Some(name)=value(block,"partition_name") else{continue};
   if name!="lk_a"&&name!="lk_b"{continue}
   lk_blocks+=1;
   let filename=value(block,"file_name").unwrap_or("MISSING");
   let download=value(block,"is_download").unwrap_or("MISSING");
   let upgradable=value(block,"is_upgradable").unwrap_or("MISSING");
   if case.lk_sha.is_some(){
      assert_eq!(filename,LK,"{} {name} filename",case.name);
   }else{
      assert!(filename=="lk_a"||filename=="lk_b"||filename=="lk.img",
       "{} baseline unexpected {name} filename {filename}",case.name);
   }
   let enabled=name=="lk_a"||case.b_enabled;
   assert_eq!(download,if enabled{"true"}else{"false"},
     "{} {name} download",case.name);
   if name=="lk_b"&&!case.b_enabled{
      assert_eq!(upgradable,"false","{} lk_b upgradable",case.name);
   }
 }
 assert!(lk_blocks==2||lk_blocks==4,
    "{} expected 2/4 LK blocks, got {lk_blocks}",case.name);
 if case.name.contains("ROW"){
    let protected=["preloader","preloader_backup","efuse","misc","para",
                   "boot_para","metadata"];
    let mut inspected=0usize;
    for block in doc.descendants().filter(|n|n.is_element()
        &&n.tag_name().name()=="partition_index"){
       let Some(name)=value(block,"partition_name") else{continue};
       if !(name.starts_with("preloader")||protected.contains(&name)){continue}
       inspected+=1;
       assert_eq!(value(block,"is_download"),Some("false"),
           "{} protected {} download",case.name,name);
       assert_eq!(value(block,"is_upgradable"),Some("false"),
           "{} protected {} upgrade",case.name,name);
    }
    assert!(inspected>0,"{} no sensitive partitions examined",case.name);
 }
 let flash_text=fs::read_to_string(&flash).unwrap();
 let flash_doc=Document::parse(&flash_text).unwrap();
 let target=flash_doc.descendants()
    .find(|n|n.is_element()&&n.tag_name().name()=="scatter")
    .and_then(|n|n.text()).unwrap().trim();
 assert_eq!(fs::canonicalize(flash.parent().unwrap().join(target)).unwrap(),
    fs::canonicalize(&scatter).unwrap(),
    "{} flash.xml must reference generated scatter",case.name);
 assert_eq!(sha(&fix.image.join("lk.img")),lk_before,
   "{} source lk modified",case.name);
 assert_eq!(sha(&fix.image.join("download_agent/flash.xml")),flash_before,
   "{} source flash.xml modified",case.name);
 let scatter_src=if fix.image.join(format!("{}_Android_scatter.xml",case.platform)).is_file(){
   fix.image.join(format!("{}_Android_scatter.xml",case.platform))
 }else{fix.image.join(format!("{}_Android_scatter.x",case.platform))};
 assert_eq!(sha(&scatter_src),scatter_before,"{} source scatter modified",case.name);
 assert!(output.generated_scatter_info.partition_count>=60,
   "{} too few partitions",case.name);
 println!("REAL_FIRMWARE_INTEGRATION=PASS {} blocks={lk_blocks} scatter={} flash={}",
   case.name,scatter.display(),flash.display());
}
#[test]fn tb361fu_row375_actual_files(){run(CASES[0]);}
#[test]fn tb336fu_row321_actual_files(){run(CASES[1]);}
#[test]fn tb373fu_row084_actual_files(){run(CASES[2]);}
#[test]fn tb375fc_prc103_actual_files(){run(CASES[3]);}
#[test]fn tb365fc_prc122_baseline_actual_files(){run(CASES[4]);}
#[test]fn tb373fu_row095_cross_version_lk_actual_files(){run(CASES[5]);}
#[test]fn tb373fu_row107_cross_version_lk_actual_files(){run(CASES[6]);}
#[test]fn tb336fu_row348_pinned_cross_version_lk_actual_files(){run(CASES[7]);}
#[test]fn tb335fc_prc321_uses_common_lk_and_ignores_stale_prc386_lk_a(){run(CASES[8]);}
#[test]fn tb335fc_prc299_boundary_baseline_actual_files(){run(CASES[9]);}
#[test]fn tb335fc_prc386_above_threshold_uses_shared_lk(){run(CASES[10]);}
#[test]fn tb335fc_prc522_above_threshold_uses_shared_lk(){run(CASES[11]);}
#[test]fn tb336fu_row271_uses_sha_pinned_family_lk(){run(CASES[12]);}
#[test]fn tb336fu_row457_uses_sha_pinned_family_lk(){run(CASES[13]);}
#[test]fn tb335fc_prc321_readonly_preflight_accepts_shared_lk_and_warns_stale_lk_a(){
    let Some((fix,lk_before,flash_before,_))=make(CASES[8]) else{return};
    let mut logs=Vec::<String>::new();
    let result=lpmbox_firmware::flash_plan::preflight_lk_before_device_changes(
        &fix.image,"TB335FC",lpmbox_core::RomRegion::Prc,
        Some("ZUXOS_1.5.10.321_ST"),InstallMode::ConvertWipe,&mut |x|logs.push(x)
    );
    assert!(result.is_ok(),"pinned common LK should be accepted: {result:?}");
    assert!(logs.iter().any(|s|s.contains("1.1.10.386")),
        "existing wrong-generation LK_A must be detected");
    assert!(logs.iter().any(|s|s.contains("PRC321 순정 lk.img")),
        "PRC321 vendor LK fingerprint must be reported");
    assert!(!fix.image.join(LK).exists());
    assert_eq!(sha(&fix.image.join("lk.img")),lk_before);
    assert_eq!(sha(&fix.image.join("download_agent/flash.xml")),flash_before);
}

#[test]
fn bad_signature_is_blocked_by_readonly_preflight_before_backup(){
    let Some((fix,lk_before,flash_before,_))=make(CASES[0]) else{return};
    let result=lpmbox_firmware::flash_plan::preflight_lk_before_device_changes(
        &fix.image,"TB361FU",lpmbox_core::RomRegion::Row,
        Some("ZUI_17.5.10.375_ST"),InstallMode::ConvertWipe,&mut |_|{}
    );
    assert!(result.unwrap_err().to_string().contains("Signature(Entity) invalid"));
    assert!(!fix.image.join(LK).exists());
    assert_eq!(sha(&fix.image.join("lk.img")),lk_before);
    assert_eq!(sha(&fix.image.join("download_agent/flash.xml")),flash_before);
}
#[test]
fn row348_readonly_preflight_checks_source_without_patching(){
    let Some((fix,lk_before,flash_before,_))=make(CASES[7]) else{return};
    lpmbox_firmware::flash_plan::preflight_lk_before_device_changes(
        &fix.image,"TB336FU",lpmbox_core::RomRegion::Row,
        Some("ZUI_17.5.10.348_ST"),InstallMode::ConvertWipe,&mut |_|{}
    ).unwrap();
    assert!(!fix.image.join(LK).exists());
    assert_eq!(sha(&fix.image.join("lk.img")),lk_before);
    assert_eq!(sha(&fix.image.join("download_agent/flash.xml")),flash_before);
}

#[test]
fn tb335fc_prc321_rejects_unexpected_preexisting_candidate(){
    let Some((fix,lk_before,flash_before,_))=make(CASES[8]) else{return};
    fs::write(fix.image.join(LK),b"not-the-user-tested-patched-lk").unwrap();
    let mut output=Vec::new();
    let check=lpmbox_firmware::flash_plan::preflight_lk_before_device_changes(
        &fix.image,"TB335FC",lpmbox_core::RomRegion::Prc,
        Some("ZUXOS_1.5.10.321_ST"),InstallMode::ConvertWipe,
        &mut |m|output.push(m)
    );
    assert!(check.is_err());
    assert_eq!(fs::read(fix.image.join(LK)).unwrap(),b"not-the-user-tested-patched-lk");
    assert_eq!(sha(&fix.image.join("lk.img")),lk_before);
    assert_eq!(sha(&fix.image.join("download_agent/flash.xml")),flash_before);
}
#[test]
fn tb335fc_prc321_shared_lk_is_idempotent_in_temp_copy(){
    let Some((fix,lk_before,flash_before,_))=make(CASES[8]) else{return};
    let original_stale=sha(&fix.image.join("lk_a"));
    let report=lpmbox_firmware::tb335_prc_shared_lk::stage_prc_shared_lk(
        &fix.image,Some("ZUXOS_1.5.10.321_ST")).unwrap();
    assert_eq!(report.result_sha256,
        "61e80798dd4fe8cc490543b43ccc8dd009e4dd5e5ed677eb9cf754e801c2eae8");
    let second=lpmbox_firmware::tb335_prc_shared_lk::stage_prc_shared_lk(
        &fix.image,Some("ZUXOS_1.5.10.321_ST")).unwrap();
    assert_eq!(second.result_sha256,report.result_sha256);
    assert_eq!(sha(&fix.image.join("lk_a")),original_stale);
    assert_eq!(sha(&fix.image.join("lk.img")),lk_before);
    assert_eq!(sha(&fix.image.join("download_agent/flash.xml")),flash_before);
}
#[test]
fn tb335fc_unknown_high_version_accepts_only_structural_stock_in_readonly_mode(){
    let Some((fix,lk_before,flash_before,_))=make(CASES[8]) else{return};
    let candidate=lpmbox_firmware::tb335_prc_shared_lk::preflight_prc_shared_lk(
        &fix.image,Some("ZUXOS_1.5.10.999_ST")).unwrap();
    assert_eq!(candidate.original_sha256,lk_before);
    assert!(!candidate.known_stock_hash,
        "unknown future version must carry an explicit untested warning");
    assert!(!fix.image.join(LK).exists());
    let version_below=lpmbox_firmware::tb335_prc_shared_lk::preflight_prc_shared_lk(
        &fix.image,Some("ZUXOS_1.1.10.299_ST"));
    assert!(version_below.is_err());
    assert_eq!(sha(&fix.image.join("download_agent/flash.xml")),flash_before);
}

#[test]
fn tb335fc_any_above_threshold_shared_lk_ignores_stock_size_hash_and_layout(){
    // This is the user-requested policy test: a new or unfamiliar PRC
    // source LK cannot gate use of the exact bundled shared LK candidate.
    let Some((fix,_,source_flash,_))=make(CASES[8]) else{return};
    let original=vec![0x5au8;47]; // deliberately not an MTK section tree
    fs::write(fix.image.join("lk.img"),&original).unwrap();
    for version in [
        "ZUXOS_1.1.10.300_ST",
        "ZUXOS_1.1.10.777_ST",
        "ZUXOS_1.5.10.321_ST",
        "ZUXOS_1.5.10.999_ST",
        "ZUX_2.0.10.999_ST",
    ]{
        let report=lpmbox_firmware::tb335_prc_shared_lk::stage_prc_shared_lk(
            &fix.image,Some(version)
        ).unwrap();
        assert_eq!(report.result_sha256,
            "61e80798dd4fe8cc490543b43ccc8dd009e4dd5e5ed677eb9cf754e801c2eae8");
        assert_eq!(sha(&fix.image.join(LK)),report.result_sha256);
        lpmbox_firmware::flash_plan::preflight_lk_before_device_changes(
            &fix.image,"TB335FC",lpmbox_core::RomRegion::Prc,
            Some(version),InstallMode::ConvertWipe,&mut |_|{}
        ).unwrap();
        assert_eq!(fs::read(fix.image.join("lk.img")).unwrap(),original);
    }
    for version in ["ZUXOS_1.1.10.299_ST","ZUXOS_1.0.10.100_ST"]{
        assert!(lpmbox_firmware::tb335_prc_shared_lk::preflight_prc_shared_lk(
            &fix.image,Some(version)).is_err());
    }
    assert_eq!(sha(&fix.image.join("download_agent/flash.xml")),source_flash);
}

// ReinstallWipe (Device Recovery) must run the exact LK creation/Scatter
// policies used by normal PRC<->ROW ConvertWipe installations.
// These tests execute prepare_flash_plan_with_log on TEMP COPIES only.
// No device is connected and the real ROM files are never written.
fn verify_device_recovery_lk(case:Case){
    let Some((fix,stock_lk,stock_flash,_))=make(case) else {return};
    let mut messages=Vec::new();
    let result=prepare_flash_plan_with_log(
        &fix.image,InstallMode::ReinstallWipe,|s|messages.push(s)
    );
    if case.model=="TB361FU" {
        let err=result.expect_err("known-bad TB361FU LK must stay blocked in recovery");
        assert!(err.to_string().contains("Signature(Entity) invalid"));
        assert!(!fix.image.join(LK).exists());
        assert_eq!(sha(&fix.image.join("lk.img")),stock_lk);
        assert_eq!(sha(&fix.image.join("download_agent/flash.xml")),stock_flash);
        return;
    }
    let output=result.unwrap_or_else(|e|panic!(
        "{} recovery plan failure: {e}\n{}",
        case.name,messages.join("\n")
    ));
    let plan=output.plan;
    assert_eq!(plan.mode,InstallMode::ReinstallWipe);
    assert_eq!(plan.model,case.model);
    assert_eq!(plan.platform,case.platform);
    assert!(!plan.requires_device_stage,"recovery must preserve its no-slot-stage flow");
    assert!(!plan.requires_current_slot_stage,"recovery must not add slot operations");
    if let Some(expected)=case.lk_sha {
        assert_eq!(sha(&fix.image.join(LK)),expected,
            "{} recovery LK candidate differs from ROM install",case.name);
    } else {
        assert!(!fix.image.join(LK).exists(),
            "{} baseline PRC recovery must not patch",case.name);
    }
    assert!(plan.work_scatter_xml.exists());
    assert!(plan.work_flash_xml.exists());
    let xml=fs::read_to_string(&plan.work_scatter_xml).unwrap();
    let doc=Document::parse(&xml).unwrap();
    let mut a_count=0;
    let mut b_count=0;
    let mut proinfo_count=0;
    for node in doc.descendants().filter(|n|
        n.is_element()&&n.tag_name().name()=="partition_index"
    ){
        match value(node,"partition_name") {
            Some("lk_a")=>{
                a_count+=1;
                assert_eq!(value(node,"is_download"),Some("true"),
                    "{} recovery lk_a disabled",case.name);
                if case.lk_sha.is_some(){
                    assert_eq!(value(node,"file_name"),Some(LK),
                        "{} recovery LK_A must use patched file",case.name);
                }
            }
            Some("lk_b")=>{
                b_count+=1;
                assert_eq!(
                    value(node,"is_download"),
                    Some(if case.b_enabled {"true"} else {"false"}),
                    "{} recovery LK_B policy",case.name
                );
                if !case.b_enabled{
                    assert_eq!(value(node,"is_upgradable"),Some("false"),
                        "{} recovery LK_B must never be upgradable",case.name);
                }
                if case.lk_sha.is_some(){
                    assert_eq!(value(node,"file_name"),Some(LK),
                        "{} recovery LK_B filename",case.name);
                }
            }
            Some("proinfo")=>{
                proinfo_count+=1;
                assert_eq!(value(node,"is_download"),Some("false"));
                assert_eq!(value(node,"is_upgradable"),Some("false"));
            }
            _=>{}
        }
    }
    assert!(a_count>=1 && b_count>=1,"{} missing LK_A/B blocks",case.name);
    assert!(proinfo_count>=1,"{} missing recovery proinfo block",case.name);
    let flash=fs::read_to_string(&plan.work_flash_xml).unwrap();
    let flash_doc=Document::parse(&flash).unwrap();
    let scatter=flash_doc.descendants().find(|n|
        n.is_element()&&n.tag_name().name()=="scatter"
    ).and_then(|n|n.text()).unwrap().trim();
    assert_eq!(
        fs::canonicalize(plan.work_flash_xml.parent().unwrap().join(scatter)).unwrap(),
        fs::canonicalize(&plan.work_scatter_xml).unwrap(),
        "{} recovery download-agent xml must point to generated scatter",case.name
    );
    assert_eq!(sha(&fix.image.join("lk.img")),stock_lk,
        "{} recovery source LK altered",case.name);
    assert_eq!(sha(&fix.image.join("download_agent/flash.xml")),stock_flash,
        "{} recovery original flash.xml altered",case.name);
    println!("REAL_RECOVERY_INTEGRATION=PASS {} LK={} lk_b={}",
        case.name,case.lk_sha.unwrap_or("baseline"),
        case.b_enabled);
}
#[test]
fn device_recovery_matches_lk_patch_for_actual_row_firmware(){
    for i in [0,1,2,5,6,7]{
        verify_device_recovery_lk(CASES[i]);
    }
}
#[test]
fn device_recovery_matches_lk_patch_for_actual_prc_firmware(){
    for i in [3,4,8,9,10,11]{
        verify_device_recovery_lk(CASES[i]);
    }
}

#[test]
fn tb361fu_signature_rejection_still_blocks_every_rom_mode_after_research_logging(){
    let Some((fix,stock_lk,stock_flash,_))=make(CASES[0]) else{return};
    for mode in [InstallMode::ConvertWipe,InstallMode::RowUpdateKeepData,
                 InstallMode::ReinstallWipe]{
        let mut messages=Vec::<String>::new();
        let err=lpmbox_firmware::flash_plan::preflight_lk_before_device_changes(
            &fix.image,"TB361FU",lpmbox_core::RomRegion::Row,
            Some("ZUI_17.5.10.375_ST"),mode,&mut |s|messages.push(s)
        ).unwrap_err();
        assert!(err.to_string().contains("Signature(Entity) invalid"));
        assert!(messages.iter().any(|s|s.contains("af5cc578e866ae0924ead6fa371a01faeb49c32a245bbb66155e2d4d0ad823d1")),
            "expected exact research candidate hash in read-only logs");
        assert!(!fix.image.join(LK).exists(),"no LK may be staged for device write");
    }
    assert_eq!(sha(&fix.image.join("lk.img")),stock_lk);
    assert_eq!(sha(&fix.image.join("download_agent/flash.xml")),stock_flash);
}
#[test]
fn tb365fc_above_threshold_remains_guarded_without_verified_signed_image(){
    let Some((fix,stock_lk,stock_flash,_))=make(CASES[4]) else{return};
    let mut logs=Vec::<String>::new();
    let err=lpmbox_firmware::flash_plan::preflight_lk_before_device_changes(
        &fix.image,"TB365FC",lpmbox_core::RomRegion::Prc,
        Some("ZUXOS_1.1.10.123_ST"),InstallMode::ConvertWipe,
        &mut |line|logs.push(line)
    ).unwrap_err();
    assert!(err.to_string().contains("LK"));
    assert!(logs.iter().any(|s|s.contains("PRC122")));
    assert!(!fix.image.join(LK).exists());
    assert_eq!(sha(&fix.image.join("lk.img")),stock_lk);
    assert_eq!(sha(&fix.image.join("download_agent/flash.xml")),stock_flash);
}

#[test]
fn tb361fu_wrap_research_is_logged_without_flash_or_original_modification(){
    let Some((fix,original_hash,original_flash,_))=make(CASES[0]) else{return};
    let mut msgs=Vec::new();
    let err=lpmbox_firmware::flash_plan::preflight_lk_before_device_changes(
        &fix.image,"TB361FU",lpmbox_core::RomRegion::Row,
        Some("ZUI_17.5.10.375_ST"),InstallMode::ReinstallWipe,
        &mut |s|msgs.push(s)).unwrap_err();
    assert!(err.to_string().contains("Signature(Entity) invalid"));
    assert!(msgs.iter().any(|s|s.contains("7d95271e2179e6f1b8b7096f92bb790767bdacafb7b21ff744231bbb423670d0")));
    assert!(msgs.iter().any(|s|s.contains("자동 플래싱하지 않습니다")));
    assert!(!fix.image.join(LK).exists());
    assert!(!fix.image.join("download_agent/LPMBox_flash.xml").exists());
    assert_eq!(sha(&fix.image.join("lk.img")),original_hash);
    assert_eq!(sha(&fix.image.join("download_agent/flash.xml")),original_flash);
}
