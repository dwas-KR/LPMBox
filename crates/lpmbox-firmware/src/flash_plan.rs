use crate::guard::inspect_firmware;
use crate::lk_patch;
use crate::lk_model_policy;
use crate::scatter::parse_scatter_xml;
use crate::xml_crypto::decrypt_scatter_x;
use lpmbox_core::{
    FlashPlan, FlashPreparedOutput, InstallMode, LpmError, PatchPlanMode, Result,
    ScatterPartition,
};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const EXTRA_PARTITION_FILES_FULL: &[&str] = &["lk_a", "lk_b", "dtbo_a", "dtbo_b"];
const EXTRA_PARTITION_FILES_A_ONLY: &[&str] = &["lk_a", "dtbo_a"];

const EXTRA_PARTITION_PACKAGES: &[(&str, &str, &str)] = &[
    (
        "TB365FC",
        "TB365FC",
        "https://raw.githubusercontent.com/dwas-KR/LPMBox/799bfde0a73a1c310e111f047ebece6d58131aac/TB365FC.zip",
    ),
    (
        "TB361FU",
        "TB365FC",
        "https://raw.githubusercontent.com/dwas-KR/LPMBox/799bfde0a73a1c310e111f047ebece6d58131aac/TB365FC.zip",
    ),
    (
        "TB335FC",
        "TB335FC",
        "https://raw.githubusercontent.com/dwas-KR/LPMBox/799bfde0a73a1c310e111f047ebece6d58131aac/TB335FC.zip",
    ),
    (
        "TB336FU",
        "TB335FC",
        "https://raw.githubusercontent.com/dwas-KR/LPMBox/799bfde0a73a1c310e111f047ebece6d58131aac/TB335FC.zip",
    ),
];

/// Validate model / version / selected LK bytes before any device reboot,
/// proinfo backup, country-code change, or SP Flash Tool write. This is a
/// read-only preflight. The normal Flash Plan will repeat the same check.
pub fn preflight_lk_before_device_changes<F>(
    image_dir:&Path,
    model:&str,
    region:lpmbox_core::RomRegion,
    version:Option<&str>,
    mode:InstallMode,
    on_log:&mut F,
)->Result<()>
where F: FnMut(String){
    // Diagnose the original PRC321 LK and pre-staged lk_a before selecting
    // the user-confirmed cross-region candidate. This is read-only.
    if model.eq_ignore_ascii_case("TB335FC")
        && matches!(region,lpmbox_core::RomRegion::Prc)
        && version==Some(crate::lk_input_audit::VERSION)
        && !matches!(mode,InstallMode::CountryReset){
        if let Err(err)=crate::lk_input_audit::log_prc321_audit(image_dir,on_log){
            on_log(format!(
                "[LK 출처 참고] TB335FC PRC321 원본 LK 진단: {err}.                  출처 진단은 공통 LK 적용을 제한하지 않습니다."
            ));
        }
    }
    // Research-only inspection of the exact TB361FU ROW375 source is
    // read-only. It MUST NOT change the LK authentication failure policy.
    // This reports the candidate's source hash before the existing guard
    // refuses any SP Flash Tool device write.
    if model.eq_ignore_ascii_case("TB361FU")
        && matches!(region,lpmbox_core::RomRegion::Row)
        && version==Some("ZUI_17.5.10.375_ST")
        && !matches!(mode,InstallMode::CountryReset){
        match crate::lk_region_family_research::inspect_stock_only(
            &image_dir.join("lk.img"),
            crate::lk_region_family_research::ResearchModel::Tb361FuRow375
        ){
            Ok(report)=>{
                on_log(format!(
                    "[LK 오프라인 연구] TB361FU ROW375: 원본 SHA-256={} / override 후보 SHA-256={} / 인증 자료(cert1)={} / 분기 위치=0x{:X}. 기존 override는 SPFT Signature(Entity) invalid로 거부되어 자동 플래싱하지 않습니다.",
                    report.input_sha256,report.output_sha256,
                    report.cert1_sha256,report.first_branch_offset
                ));
                match fs::read(image_dir.join("lk.img")).map_err(LpmError::Io)
                    .and_then(|stock|crate::lk_region_family_research::build_offline_candidate_with_mode(
                        &stock,
                        crate::lk_region_family_research::ResearchModel::Tb361FuRow375,
                        crate::lk_region_family_research::ResearchCertMode::Wrap
                    )){
                    Ok((_,wrap))=>on_log(format!(
                        "[LK 오프라인 연구] TB361FU ROW375 wrap 후보 SHA-256={} / {}바이트. 새로운 연구 방식이며 SPFT 인증 및 실제 부팅은 미검증. 자동 플래싱하지 않습니다.",
                        wrap.output_sha256,wrap.output_bytes
                    )),
                    Err(err)=>on_log(format!(
                        "[LK 오프라인 연구] wrap 분석 불가: {err}. 자동 플래싱은 차단합니다."
                    ))
                }
            },
            Err(err)=>on_log(format!(
                "[LK 오프라인 연구] TB361FU ROW375 원본 구조 확인 실패: {err}. 자동 플래싱은 차단합니다."
            )),
        }
    }
    lk_model_policy::enforce_before_flash_plan(model,region,version,mode,on_log)?;
    let required=lk_model_policy::requirement(model,region,version,mode)?;
    if matches!(required,lk_model_policy::LkRequirement::Tb335PrcSharedUserBoottested){
        let report=crate::tb335_prc_shared_lk::preflight_prc_shared_lk(
            image_dir,version
        )?;
        on_log(format!(
            "[LK 사전검사] TB335FC PRC {} (기준 초과 공통 적용): 원본 LK SHA-256={} / 공통 LPMBOX_lk.img SHA-256={} / 기존 목록 지문 일치={}",
            version.unwrap_or("UNKNOWN"),report.original_sha256,
            report.result_sha256,report.known_stock_hash
        ));
        if !report.known_stock_hash{
            on_log(
                "[LK 사전검사] 원본 LK 지문이 등록되지 않은 상위 PRC 버전입니다. 별도 버전 제한 없이 동일한 공통 LPMBOX_lk.img를 적용합니다.".into()
            );
        }
    }
    if matches!(required,lk_model_policy::LkRequirement::Tb336Row321UserBoottested){
        let stock_sha=crate::experimental_lk_trial::validate_selected_tb336_row_lk(
            image_dir,version
        )?;
        on_log(format!(
            "[LK 사전검사] {} {}: 원본 lk.img SHA-256={} / ROW321 공통 후보 SHA-256={} (파일 읽기만 수행)",
            model,version.unwrap_or("UNKNOWN"),stock_sha,
            crate::experimental_lk_trial::EXPECTED_TRIAL_SHA
        ));
    }
    Ok(())
}
pub fn prepare_flash_plan(selected_path: &Path, mode: InstallMode) -> Result<FlashPreparedOutput> {
    prepare_flash_plan_with_log(selected_path, mode, |_| {})
}

pub fn prepare_flash_plan_with_country_code_log<F>(
    selected_path: &Path,
    mode: InstallMode,
    selected_country_code: Option<&str>,
    mut on_log: F,
) -> Result<FlashPreparedOutput>
where
    F: FnMut(String),
{
    prepare_flash_plan_internal(selected_path, mode, selected_country_code, None, &mut on_log)
}

pub fn prepare_flash_plan_with_log<F>(
    selected_path: &Path,
    mode: InstallMode,
    mut on_log: F,
) -> Result<FlashPreparedOutput>
where
    F: FnMut(String),
{
    prepare_flash_plan_internal(selected_path, mode, None, None, &mut on_log)
}

pub fn prepare_flash_plan_with_active_lk_log<F>(
    selected_path: &Path,
    mode: InstallMode,
    selected_country_code: Option<&str>,
    active_lk_readback: &Path,
    mut on_log: F,
) -> Result<FlashPreparedOutput>
where F: FnMut(String) {
    prepare_flash_plan_internal(
        selected_path, mode, selected_country_code, Some(active_lk_readback), &mut on_log
    )
}

fn prepare_flash_plan_internal<F>(
    selected_path: &Path,
    mode: InstallMode,
    selected_country_code: Option<&str>,
    active_lk_readback: Option<&Path>,
    on_log: &mut F,
) -> Result<FlashPreparedOutput>
where
    F: FnMut(String),
{
    let (_firmware_root_dir, image_dir) = resolve_root_and_image_dir(selected_path)?;
    let root_dir = lpmbox_core::app_paths::runtime_root();

    let firmware = inspect_firmware(&image_dir)?;

    if firmware.blocked_firmware_check.blocked {
        return Err(LpmError::BlockedFirmware(
            firmware.blocked_firmware_check.message.clone(),
        ));
    }

    let platform = firmware.platform.clone().ok_or_else(|| {
        LpmError::InvalidFirmwareFolder("플랫폼을 감지하지 못했습니다.".to_string())
    })?;

    let scatter_info = firmware.scatter_xml_info.clone().ok_or_else(|| {
        LpmError::InvalidFirmwareFolder("scatter XML 정보를 읽지 못했습니다.".to_string())
    })?;

    let patch_mode = install_mode_to_patch_mode(mode);

    let selected_patch_plan = scatter_info
        .patch_plans
        .iter()
        .find(|plan| plan.mode == patch_mode)
        .cloned()
        .ok_or_else(|| {
            LpmError::InvalidFirmwareFolder(format!("{patch_mode:?} plan을 찾지 못했습니다."))
        })?;

    if !selected_patch_plan.available {
        return Err(LpmError::InvalidFirmwareFolder(format!(
            "{} plan은 현재 펌웨어에서 사용할 수 없습니다. 경고: {}",
            selected_patch_plan.title,
            if selected_patch_plan.warnings.is_empty() {
                "-".to_string()
            } else {
                selected_patch_plan.warnings.join(" / ")
            }
        )));
    }

    let selected_snapshot = scatter_info
        .patched_snapshots
        .iter()
        .find(|snapshot| snapshot.mode == patch_mode)
        .cloned()
        .ok_or_else(|| {
            LpmError::InvalidFirmwareFolder(format!(
                "{patch_mode:?} patch snapshot을 찾지 못했습니다."
            ))
        })?;

    let selected_patch_validation = scatter_info
        .patch_validations
        .iter()
        .find(|validation| validation.mode == patch_mode)
        .cloned();

    if let Some(validation) = &selected_patch_validation {
        if !validation.passed {
            return Err(LpmError::InvalidFirmwareFolder(format!(
                "{} patch 검증 실패: {}",
                validation.title,
                validation.errors.join(" / ")
            )));
        }
    }

    // Validate exact model/ROM LK requirements before any device write.
    // TB336FU ROW321/348 and TB335FC PRC>299 use the user-confirmed
    // same-family, SHA-256 pinned candidate. This is not a cross-family
    // TB361FU/TB365FC approval. Other model paths remain unchanged.
    preflight_lk_before_device_changes(
        &image_dir,&firmware.model,firmware.region,
        firmware.version.as_deref(),mode,on_log
    )?;

    // TB336FU ROW 321 exact-version user-boot-tested candidate:
    // stage a separate, pinned LK_A file BEFORE standard ROM staging.
    // This does not write to the device; SPFT uses the selected scatter.
    let use_row321_user_lk = matches!(
        lk_model_policy::requirement(
            &firmware.model, firmware.region, firmware.version.as_deref(),mode
        )?,
        lk_model_policy::LkRequirement::Tb336Row321UserBoottested
    );
    let use_tb335_prc_shared_lk=matches!(
        lk_model_policy::requirement(
            &firmware.model,firmware.region,firmware.version.as_deref(),mode
        )?,
        lk_model_policy::LkRequirement::Tb335PrcSharedUserBoottested
    );
    let tb335_prc_shared_report=if use_tb335_prc_shared_lk{
        let report=crate::tb335_prc_shared_lk::stage_prc_shared_lk(
            &image_dir,firmware.version.as_deref()
        )?;
        on_log(format!(
            "[LK 패치] TB335FC PRC {}: 공통 LK 파일={} / 원본 SHA-256={} / 패치 SHA-256={}",
            firmware.version.as_deref().unwrap_or("UNKNOWN"),
            report.staged_path.display(),report.original_sha256,
            report.result_sha256
        ));
        Some(report)
    }else{None};
    let use_row375_patched_lk=matches!(
        lk_model_policy::requirement(
            &firmware.model,firmware.region,firmware.version.as_deref(),mode
        )?,
        lk_model_policy::LkRequirement::Tb361Row375Experimental
    );
    let row375_lk_report=if use_row375_patched_lk{
        let report=crate::tb361fu_row375_lk::stage_exact_row375_patch(&image_dir)?;
        on_log(format!(
            "[LK 패치][미검증] TB361FU ROW375: 순정 해시={} / 결과 해시={} / 명령어=0x{:X} / 크기={} / 출력={}",
            report.source_sha256,report.output_sha256,report.patch_offset,
            report.output_size,report.output_path.display()
        ));
        Some(report)
    }else{None};
    let row321_lk_report=if use_row321_user_lk {
        let report=crate::experimental_lk_trial::stage_user_boottested_row321_lk(
            &image_dir,firmware.version.as_deref()
        )?;
        on_log(format!(
            "[LK 패치] TB336FU ROW 입력 원본 SHA-256={} / 실기기 부팅 확인 후보 SHA-256={}",
            report.original_sha256,report.candidate_sha256
        ));
        on_log(format!(
            "[LK 패치] 변경 위치=0x{:X}, 크기 변화 +{}B, LK_A 전용 파일={}",
            report.patch_offset,report.size_delta,report.staged_path.display()
        ));
        Some(report)
    }else{None};

    let source_flash_xml = resolve_flash_xml(&image_dir)?;
    let da_path = resolve_da_path(&image_dir)?;

    remove_stale_row_platform_scatter_xml_if_needed(firmware.region, &image_dir, &platform)?;

    let source_scatter = resolve_scatter_referenced_by_flash_xml(
        &image_dir,
        &source_flash_xml,
        &platform,
        &firmware.model,
        firmware.region,
    )?;

    validate_source_flash_xml(&source_flash_xml, &source_scatter)?;

    let title = install_mode_title(mode).to_string();

    // Keep PRC stock LK/DTBO and source scatter references intact for the
    // <=060 generation. Higher PRC versions need a SHA-256 verified LK patch.
    // ROW keeps its previously validated 084+ patch and DTBO staging route.
    let mut prc_patched_lk = None;
    if lk_patch::is_tb375_family(&firmware.model) {
        if !matches!(mode, InstallMode::CountryReset) {
            match firmware.region {
                lpmbox_core::RomRegion::Prc => {
                    let changed = lk_patch::prepare_prc_lk_for_version(
                        &image_dir,firmware.version.as_deref(),on_log
                    )?;
                    prc_patched_lk = Some(changed);
                }
                lpmbox_core::RomRegion::Row => {
                    lk_patch::prepare_local_lk_dtbo_from_readback(
                        firmware.region,firmware.version.as_deref(),
                        &image_dir,active_lk_readback,on_log
                    )?;
                }
                lpmbox_core::RomRegion::Unknown => {
                    return Err(LpmError::InvalidFirmwareFolder(
                        "PRC/ROW 펌웨어 지역 확인 실패".to_string()
                    ));
                }
            }
        }
    } else {
        if use_row321_user_lk || use_row375_patched_lk {
            // Keep the exact-hash source lk.img for subsequent re-validation.
            // Stock scatter will not reference this file after LK_A mapping.
            on_log("[LK 패치] 해시 검증된 ROW 원본 lk.img/dtbo.img를 삭제하지 않고 보존합니다.".into());
        }else{
            remove_row_lk_dtbo_img_files(firmware.region, &image_dir, on_log)?;
        }
        prepare_extra_partition_files_for_model(&firmware.model, &image_dir, on_log)?;
    }

    let work_dir = image_dir.clone();

    let work_download_agent_dir = source_flash_xml
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| image_dir.join("download_agent"));

    let work_scatter_xml = build_lpmbox_scatter_path(&source_scatter, &platform);
    let mut target_partitions = selected_snapshot.partitions.clone();
    if let Some(report)=&row321_lk_report{
        if !report.staged_path.is_file(){
            return Err(LpmError::InvalidFirmwareFolder(
                "TB336FU ROW321 LPMBOX_lk.img 결과 파일이 없습니다.".into()
            ));
        }
        on_log("[LK 패치] TB336FU 공통 LK: LPMBOX_lk.img 생성과 해시 검증 완료.".into());
    }
    if let Some(patched)=prc_patched_lk{
        lk_patch::keep_prc_stock_image_references(&mut target_partitions,patched);
        on_log(format!("[LK] PRC LK/DTBO scatter 경로 설정: LK 패치={patched}"));
    }
    let patched_supported=
        row375_lk_report.is_some()
        || row321_lk_report.is_some()
        || tb335_prc_shared_report.is_some()
        || (prc_patched_lk==Some(true))
        || (lk_patch::is_tb375_family(&firmware.model)
            && firmware.region==lpmbox_core::RomRegion::Row
            && !matches!(mode,InstallMode::CountryReset));
    if patched_supported{
        let output=image_dir.join(lk_patch::LPMBOX_PATCHED_LK_FILENAME);
        if !output.is_file(){
            return Err(LpmError::InvalidFirmwareFolder(format!(
                "패치된 LK 파일이 없습니다: {}",
                output.display()
            )));
        }
        apply_unified_patched_lk_to_scatter(
            &mut target_partitions,
            &firmware.model,
            lk_patch::LPMBOX_PATCHED_LK_FILENAME
        )?;
        on_log(format!(
            "[LK] {} {}: LK_A/LK_B 파일명=LPMBOX_lk.img, LK_B 다운로드={}",
            firmware.model,
            firmware.version.as_deref().unwrap_or("UNKNOWN"),
            if matches!(firmware.model.as_str(),"TB335FC"|"TB336FU"|"TB365FC"|"TB361FU"){
                "false"
            }else{"true"}
        ));
    }
    // This must run for all MT6835 ROM plans, including baseline PRC builds
    // that intentionally do not create a patched LK image.
    let mt6835_lk_b_guard_count=
        protect_mt6835_lk_b(&mut target_partitions,&firmware.model);
    if mt6835_lk_b_guard_count>0{
        on_log(format!(
            "[LK 보호] {}: lk_b 다운로드/업데이트 플래그 비활성화",
            firmware.model
        ));
    }
    // Apply the same sensitive-partition protection to every supported
    // ROW LK route, including the boot-tested TB336FU ROW321 and the
    // exact-input TB361FU ROW375 research patch.
    let row_lk_guard = matches!(firmware.region, lpmbox_core::RomRegion::Row)
        && (lk_patch::is_tb375_family(&firmware.model)
            || (firmware.model.eq_ignore_ascii_case("TB361FU") && use_row375_patched_lk)
            || (firmware.model.eq_ignore_ascii_case("TB336FU") && use_row321_user_lk));
    // TB335FC PRC high-version LK stays LK_A-only via
    // protect_mt6835_lk_b, without altering the existing PRC partition
    // selection or device-stage sequence.
    let protected_count = mt6835_lk_b_guard_count + if row_lk_guard
    {
        let count = lk_patch::protect_tb373_row_downloads(&mut target_partitions);
        on_log(format!(
            "[LK 보호] ROW 전용 LK 설치: PreLoader, efuse, misc/para/metadata 등 보호 파티션 {}개 항목 비활성화",
            count
        ));
        count
    } else {
        0
    };
    write_patched_scatter_xml(
        &source_scatter,
        &target_partitions,
        &work_scatter_xml,
        selected_country_code,
        mode,
        &image_dir,
        &source_flash_xml,
    )?;

    let work_flash_xml = work_download_agent_dir.join("LPMBox_flash.xml");
    write_lpmbox_flash_xml(&source_flash_xml, &work_flash_xml, &work_scatter_xml)?;

    let keep_user_data = matches!(mode, InstallMode::RowUpdateKeepData);

    let requires_current_slot_stage = matches!(
        mode,
        InstallMode::ConvertWipe | InstallMode::RowUpdateKeepData
    );

    let requires_device_stage = !matches!(mode, InstallMode::ReinstallWipe);

    let requires_proinfo_patch = matches!(mode, InstallMode::CountryReset)
        || selected_country_code
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);

    let spft_command_preview = format!(
        "SPFlashToolV6.exe -a \"{}\" -f \"{}\" -c download",
        da_path.display(),
        work_flash_xml.display()
    );

    let generated_scatter_info = parse_scatter_xml(&work_scatter_xml)?;

    let plan = FlashPlan {
        mode,
        patch_mode,
        title,
        root_dir,
        image_dir,
        work_dir,
        work_download_agent_dir,
        source_flash_xml: source_flash_xml.clone(),
        source_scatter: source_scatter.clone(),
        work_flash_xml,
        work_scatter_xml,
        da_path,
        platform,
        model: firmware.model.clone(),
        image_version: firmware.version.clone(),
        image_region: firmware.region,
        keep_user_data,
        requires_current_slot_stage,
        requires_device_stage,
        requires_proinfo_patch,
        spft_command_preview,
    };

    Ok(FlashPreparedOutput {
        firmware,
        plan,
        selected_patch_plan,
        selected_patch_validation,
        generated_scatter_info,
        changed_count: selected_snapshot.changed_count + protected_count,
    })

}

fn resolve_root_and_image_dir(selected_path: &Path) -> Result<(PathBuf, PathBuf)> {
    if !selected_path.exists() || !selected_path.is_dir() {
        return Err(LpmError::InvalidFirmwareFolder(
            selected_path.display().to_string(),
        ));
    }

    if file_name_eq(selected_path, "image") {
        let Some(root_dir) = selected_path.parent() else {
            return Err(LpmError::InvalidFirmwareFolder(format!(
                "image 폴더의 상위 root 경로를 찾지 못했습니다: {}",
                selected_path.display()
            )));
        };

        return Ok((root_dir.to_path_buf(), selected_path.to_path_buf()));
    }

    let image_dir = selected_path.join("image");

    if image_dir.is_dir() {
        return Ok((selected_path.to_path_buf(), image_dir));
    }

    Err(LpmError::InvalidFirmwareFolder(format!(
        "image 폴더 또는 image 폴더를 포함한 root 폴더를 선택해야 합니다: {}",
        selected_path.display()
    )))
}


/// Map only a precisely prepared LK image in the generated scatter.
/// MT6835 families: LK_B must never be downloaded. TB373/TB375 keep
/// distinct A/B entries but refer to the same verified patched file.
fn apply_unified_patched_lk_to_scatter(
    partitions:&mut [ScatterPartition],
    model:&str,
    filename:&str
)->Result<()>{
    if filename!=lk_patch::LPMBOX_PATCHED_LK_FILENAME {
        return Err(LpmError::InvalidFirmwareFolder(format!(
            "LK 생성 파일명은 {}여야 합니다.",lk_patch::LPMBOX_PATCHED_LK_FILENAME
        )));
    }
    for slot in ["lk_a","lk_b"]{
        if partitions.iter().filter(|p|p.name.eq_ignore_ascii_case(slot)).count()!=1{
            return Err(LpmError::InvalidFirmwareFolder(format!(
                "{model}: Scatter에 {slot} 항목이 정확히 한 개 있어야 합니다."
            )));
        }
    }
    let is_mt6835=matches!(
        model.to_ascii_uppercase().as_str(),
        "TB335FC" | "TB336FU" | "TB365FC" | "TB361FU"
    );
    for p in partitions.iter_mut(){
        if p.name.eq_ignore_ascii_case("lk_a"){
            p.file_name=Some(filename.to_owned());
            p.is_download=Some("true".into());
        }else if p.name.eq_ignore_ascii_case("lk_b"){
            p.file_name=Some(filename.to_owned());
            p.is_download=Some((if is_mt6835{"false"}else{"true"}).into());
            if is_mt6835{
                p.is_upgradable=Some("false".into());
            }
        }
    }
    Ok(())
}


/// Enforce the documented MT6835 B-slot exclusion even when no LK patch
/// is required for an older PRC firmware.
fn protect_mt6835_lk_b(
    partitions:&mut [ScatterPartition],model:&str
)->usize{
    let is_mt6835=matches!(model.to_ascii_uppercase().as_str(),
        "TB335FC" | "TB336FU" | "TB365FC" | "TB361FU");
    if !is_mt6835{return 0;}
    let mut changed=0usize;
    for p in partitions.iter_mut().filter(|p|p.name.eq_ignore_ascii_case("lk_b")){
        if p.is_download.as_deref()!=Some("false"){
            p.is_download=Some("false".into());
            changed+=1;
        }
        if p.is_upgradable.as_deref()!=Some("false"){
            p.is_upgradable=Some("false".into());
            changed+=1;
        }
    }
    changed
}

fn remove_row_lk_dtbo_img_files<F>(
    image_region: lpmbox_core::RomRegion,
    image_dir: &Path,
    on_log: &mut F,
) -> Result<()>
where
    F: FnMut(String),
{
    if !matches!(image_region, lpmbox_core::RomRegion::Row) {
        return Ok(());
    }

    for file_name in ["lk.img", "dtbo.img"] {
        let path = image_dir.join(file_name);

        if path.exists() {
            fs::remove_file(&path).map_err(|err| {
                LpmError::InvalidFirmwareFolder(format!(
                    "ROW lk/dtbo 파일을 제거하지 못했습니다: {} / {err}",
                    path.display()
                ))
            })?;
        }
    }

    on_log("[Image] ROW(글로벌롬) lk/dtbo 파일을 제거합니다.".to_string());

    Ok(())
}

fn prepare_extra_partition_files_for_model<F>(
    model: &str,
    image_dir: &Path,
    on_log: &mut F,
) -> Result<()>
where
    F: FnMut(String),
{
    let model = model.trim().to_ascii_uppercase();

    let Some(package) = EXTRA_PARTITION_PACKAGES
        .iter()
        .find(|package| package.0.eq_ignore_ascii_case(&model))
    else {
        return Ok(());
    };

    let package_name = package.1;
    let url = package.2;
    let required_files = extra_partition_files_for_model(&model);

    if required_files.is_empty() {
        return Ok(());
    }

    if extra_partition_files_exist(image_dir, required_files) {
        on_log(extra_partition_copy_log_message(&model).to_string());
        return Ok(());
    }

    let tool_model_dir = lpmbox_core::app_paths::tool_dir().join(package_name);
    let zip_path = lpmbox_core::app_paths::tool_download_dir().join(format!("{package_name}.zip"));

    if let Some(parent) = zip_path.parent() {
        fs::create_dir_all(parent)?;
    }

    if tool_model_dir.exists() {
        fs::remove_dir_all(&tool_model_dir)?;
    }

    fs::create_dir_all(&tool_model_dir)?;

    if zip_path.exists() {
        fs::remove_file(&zip_path)?;
    }

    on_log(format!(
        "[Image] {package_name} lk, dtbo 파일을 다운로드 합니다."
    ));

    download_binary_file(url, &zip_path)?;
    extract_zip_file_to_dir(&zip_path, &tool_model_dir)?;

    if zip_path.exists() {
        fs::remove_file(&zip_path)?;
    }

    on_log(extra_partition_copy_log_message(&model).to_string());

    copy_extra_partition_files_to_image(&tool_model_dir, image_dir, package_name, required_files)?;

    Ok(())
}

fn extra_partition_files_for_model(model: &str) -> &'static [&'static str] {
    match model.trim().to_ascii_uppercase().as_str() {
        "TB375FC" | "TB373FU" => EXTRA_PARTITION_FILES_FULL,
        "TB365FC" | "TB361FU" | "TB335FC" | "TB336FU" => EXTRA_PARTITION_FILES_A_ONLY,
        _ => &[],
    }
}

fn extra_partition_copy_log_message(model: &str) -> &'static str {
    match model.trim().to_ascii_uppercase().as_str() {
        "TB375FC" | "TB373FU" => {
            "[Image] image 폴더에 lk_a, lk_b, dtbo_a, dtbo_b 파일을 복사합니다."
        }
        "TB365FC" | "TB361FU" | "TB335FC" | "TB336FU" => {
            "[Image] image 폴더에 lk_a, dtbo_a 파일을 복사합니다."
        }
        _ => "[Image] image 폴더에 lk, dtbo 파일을 복사합니다.",
    }
}

fn extra_partition_files_exist(image_dir: &Path, required_files: &[&str]) -> bool {
    required_files
        .iter()
        .all(|file_name| image_dir.join(file_name).is_file())
}

fn copy_extra_partition_files_to_image(
    tool_model_dir: &Path,
    image_dir: &Path,
    model: &str,
    required_files: &[&str],
) -> Result<()> {
    for file_name in required_files {
        let source = find_file_recursively(tool_model_dir, file_name).ok_or_else(|| {
            LpmError::InvalidFirmwareFolder(format!(
                "{model}.zip 안에서 {file_name} 파일을 찾지 못했습니다."
            ))
        })?;

        let target = image_dir.join(file_name);

        fs::copy(&source, &target).map_err(|err| {
            LpmError::InvalidFirmwareFolder(format!(
                "{file_name} 파일을 image 폴더로 복사하지 못했습니다. source={} / target={} / {err}",
                source.display(),
                target.display()
            ))
        })?;
    }

    Ok(())
}

fn resolve_flash_xml(image_dir: &Path) -> Result<PathBuf> {
    let candidates = [
        image_dir.join("download_agent").join("flash.xml"),
        image_dir.join("flash.xml"),
    ];

    for path in candidates {
        if path.is_file() {
            return Ok(path);
        }
    }

    Err(LpmError::FileNotFound(format!(
        "{} 또는 {}",
        image_dir.join("download_agent").join("flash.xml").display(),
        image_dir.join("flash.xml").display()
    )))
}

fn resolve_da_path(image_dir: &Path) -> Result<PathBuf> {
    let candidates = [
        image_dir.join("download_agent").join("DA_BR.bin"),
        image_dir.join("download_agent").join("da.auth"),
        image_dir.join("DA_BR.bin"),
        image_dir.join("da.auth"),
    ];

    for path in candidates {
        if path.is_file() {
            return Ok(path);
        }
    }

    Err(LpmError::FileNotFound(format!(
        "{} 또는 {}",
        image_dir.join("download_agent").join("DA_BR.bin").display(),
        image_dir.join("DA_BR.bin").display()
    )))
}

fn remove_stale_row_platform_scatter_xml_if_needed(
    image_region: lpmbox_core::RomRegion,
    image_dir: &Path,
    platform: &str,
) -> Result<()> {
    if !matches!(image_region, lpmbox_core::RomRegion::Row) {
        return Ok(());
    }

    let platform = platform.trim();
    let scatter_x_exists = image_dir
        .join(format!("{platform}_Android_scatter.x"))
        .is_file()
        || image_dir
            .join("download_agent")
            .join(format!("{platform}_Android_scatter.x"))
            .is_file();

    if !scatter_x_exists {
        return Ok(());
    }

    for path in [
        image_dir.join(format!("{platform}_Android_scatter.xml")),
        image_dir.join(format!("LPMBox_{platform}_Android_scatter.xml")),
        image_dir
            .join("download_agent")
            .join(format!("{platform}_Android_scatter.xml")),
        image_dir
            .join("download_agent")
            .join(format!("LPMBox_{platform}_Android_scatter.xml")),
    ] {
        if path.is_file() {
            fs::remove_file(&path).map_err(|err| {
                LpmError::InvalidFirmwareFolder(format!(
                    "ROW scatter XML 정리 실패: {} / {err}",
                    path.display()
                ))
            })?;
        }
    }

    Ok(())
}

fn resolve_scatter_referenced_by_flash_xml(
    image_dir: &Path,
    flash_xml: &Path,
    platform: &str,
    model: &str,
    image_region: lpmbox_core::RomRegion,
) -> Result<PathBuf> {
    let expected_platform = expected_scatter_platform_for_model(model)
        .unwrap_or_else(|| platform.trim());

    if !expected_platform.eq_ignore_ascii_case(platform.trim()) {
        return Err(LpmError::InvalidFirmwareFolder(format!(
            "모델과 플랫폼 정보가 일치하지 않습니다: model={model}, image platform={platform}, expected scatter={expected_platform}"
        )));
    }

    let flash_text = read_text_lossy(flash_xml)?;

    if let Some(scatter_text) = extract_tag_text_case_insensitive(&flash_text, "scatter") {
        let scatter_text = scatter_text.trim();

        if !scatter_text.is_empty() {
            let referenced = if Path::new(scatter_text).is_absolute() {
                PathBuf::from(scatter_text)
            } else {
                flash_xml.parent().unwrap_or(image_dir).join(scatter_text)
            };

            let referenced = prefer_row_scatter_x_if_available(&referenced, image_region);

            if referenced.is_file() && scatter_path_matches_platform(&referenced, expected_platform) {
                return Ok(referenced);
            }
        }
    }

    for path in platform_scatter_candidates(image_dir, expected_platform, image_region) {
        if path.is_file() {
            return Ok(path);
        }
    }

    Err(LpmError::FileNotFound(format!(
        "{} 또는 {}",
        image_dir
            .join(format!("{expected_platform}_Android_scatter.xml"))
            .display(),
        image_dir
            .join(format!("{expected_platform}_Android_scatter.x"))
            .display()
    )))
}

fn expected_scatter_platform_for_model(model: &str) -> Option<&'static str> {
    match model.trim().to_ascii_uppercase().as_str() {
        "TB335FC" | "TB336FU" | "TB365FC" | "TB361FU" => Some("MT6835"),
        "TB375FC" | "TB373FU" => Some("MT6897"),
        _ => None,
    }
}

fn platform_scatter_candidates(
    image_dir: &Path,
    platform: &str,
    image_region: lpmbox_core::RomRegion,
) -> Vec<PathBuf> {
    let xml_root = image_dir.join(format!("{platform}_Android_scatter.xml"));
    let x_root = image_dir.join(format!("{platform}_Android_scatter.x"));
    let xml_da = image_dir
        .join("download_agent")
        .join(format!("{platform}_Android_scatter.xml"));
    let x_da = image_dir
        .join("download_agent")
        .join(format!("{platform}_Android_scatter.x"));

    if matches!(image_region, lpmbox_core::RomRegion::Row) {
        vec![x_root, x_da, xml_root, xml_da]
    } else {
        vec![xml_root, xml_da, x_root, x_da]
    }
}

fn scatter_path_matches_platform(path: &Path, platform: &str) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    let file_name = file_name.to_ascii_uppercase();
    let platform = platform.trim().to_ascii_uppercase();

    file_name.contains(&format!("{platform}_ANDROID_SCATTER"))
}

fn prefer_row_scatter_x_if_available(
    path: &Path,
    image_region: lpmbox_core::RomRegion,
) -> PathBuf {
    if !matches!(image_region, lpmbox_core::RomRegion::Row) {
        return prefer_plain_xml_for_scatter(path);
    }

    let is_xml = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("xml"))
        .unwrap_or(false);

    if is_xml {
        let x_path = path.with_extension("x");

        if x_path.is_file() {
            return x_path;
        }
    }

    path.to_path_buf()
}

fn prefer_plain_xml_for_scatter(path: &Path) -> PathBuf {
    let is_scatter_x = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("x"))
        .unwrap_or(false);

    if is_scatter_x {
        let xml_path = path.with_extension("xml");

        if xml_path.is_file() {
            return xml_path;
        }
    }

    path.to_path_buf()
}

fn read_text_lossy(path: &Path) -> Result<String> {
    let data = fs::read(path)?;
    Ok(String::from_utf8_lossy(&data).replace('\u{feff}', ""))
}

fn read_scatter_text_lossy_or_decrypt(path: &Path) -> Result<String> {
    let is_scatter_x = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("x"))
        .unwrap_or(false);

    if is_scatter_x {
        let decrypted = decrypt_scatter_x(path)?;
        return Ok(String::from_utf8_lossy(&decrypted).replace('\u{feff}', ""));
    }

    read_text_lossy(path)
}

fn validate_source_flash_xml(flash_xml: &Path, scatter_path: &Path) -> Result<()> {
    let text = read_text_lossy(flash_xml)?;

    if text.contains("xmlml") {
        return Err(LpmError::InvalidFirmwareFolder(format!(
            "flash.xml scatter 경로가 손상되었습니다: {}",
            flash_xml.display()
        )));
    }

    let scatter_text = extract_tag_text_case_insensitive(&text, "scatter").ok_or_else(|| {
        LpmError::InvalidFirmwareFolder(format!(
            "flash.xml 안에서 <scatter> 항목을 찾지 못했습니다: {}",
            flash_xml.display()
        ))
    })?;

    if scatter_text.trim().is_empty() {
        return Err(LpmError::InvalidFirmwareFolder(format!(
            "flash.xml 안의 <scatter> 값이 비어 있습니다: {}",
            flash_xml.display()
        )));
    }

    if !scatter_path.is_file() {
        return Err(LpmError::FileNotFound(scatter_path.display().to_string()));
    }

    Ok(())
}

fn extract_tag_text_case_insensitive(text: &str, tag: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();

    let start_tag = format!("<{}>", tag.to_ascii_lowercase());
    let end_tag = format!("</{}>", tag.to_ascii_lowercase());

    let start = lower.find(&start_tag)?;
    let after_start = start + start_tag.len();
    let relative_end = lower[after_start..].find(&end_tag)?;
    let end = after_start + relative_end;

    Some(text[after_start..end].to_string())
}

fn install_mode_to_patch_mode(mode: InstallMode) -> PatchPlanMode {
    match mode {
        InstallMode::ConvertWipe => PatchPlanMode::ConvertWipe,
        InstallMode::RowUpdateKeepData => PatchPlanMode::RowUpdateKeepData,
        InstallMode::ReinstallWipe => PatchPlanMode::ReinstallWipe,
        InstallMode::CountryReset => PatchPlanMode::CountryReset,
    }
}

fn install_mode_title(mode: InstallMode) -> &'static str {
    match mode {
        InstallMode::ConvertWipe => "설치 [데이터 초기화]",
        InstallMode::RowUpdateKeepData => "ROW 업데이트 [데이터 유지]",
        InstallMode::ReinstallWipe => "기기 복구 [데이터 초기화]",
        InstallMode::CountryReset => "국가 코드 재설정",
    }
}

fn file_name_eq(path: &Path, expected: &str) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.eq_ignore_ascii_case(expected))
        .unwrap_or(false)
}

fn build_lpmbox_scatter_path(source_scatter: &Path, platform: &str) -> PathBuf {
    let parent = source_scatter
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| lpmbox_core::app_paths::runtime_root());

    parent.join(format!("LPMBox_{platform}_Android_scatter.xml"))
}

fn write_patched_scatter_xml(
    source_scatter: &Path,
    patched_partitions: &[ScatterPartition],
    out_path: &Path,
    selected_country_code: Option<&str>,
    mode: InstallMode,
    image_dir: &Path,
    source_flash_xml: &Path,
) -> Result<()> {
    let source_text = read_scatter_text_lossy_or_decrypt(source_scatter)?;

    let mut map = HashMap::<String, ScatterPartition>::new();

    for partition in patched_partitions {
        map.insert(partition.name.to_ascii_lowercase(), partition.clone());
    }

    let mut patched_text = patch_scatter_xml_text(&source_text, &map);

    if matches!(mode, InstallMode::ReinstallWipe) {
        patched_text = force_proinfo_disabled_values(&patched_text);
    } else if selected_country_code
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
    {
        patched_text = ensure_proinfo_download_block(&patched_text);
    }

    patched_text = fix_missing_preloader_file_names(&patched_text, image_dir, source_flash_xml)?;

    fs::write(out_path, patched_text.as_bytes())?;

    Ok(())
}

fn fix_missing_preloader_file_names(
    source_text: &str,
    image_dir: &Path,
    source_flash_xml: &Path,
) -> Result<String> {
    let flash_text = read_text_lossy(source_flash_xml).unwrap_or_default();
    let flash_project = extract_tag_text_case_insensitive(&flash_text, "project")
        .unwrap_or_default()
        .trim()
        .to_string();

    let fallback_preloader = if flash_project.is_empty() {
        None
    } else {
        let file_name = format!("preloader_{flash_project}.bin");

        if image_dir.join(&file_name).is_file() {
            Some(file_name)
        } else {
            None
        }
    }
    .or_else(|| find_single_preloader_bin(image_dir));

    let Some(fallback_preloader) = fallback_preloader else {
        return Ok(source_text.to_string());
    };

    Ok(patch_missing_preloader_blocks(
        source_text,
        image_dir,
        &fallback_preloader,
    ))
}

fn find_single_preloader_bin(image_dir: &Path) -> Option<String> {
    let entries = fs::read_dir(image_dir).ok()?;
    let mut preloaders = Vec::new();

    for entry in entries.filter_map(|entry| entry.ok()) {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        let lower = name.to_ascii_lowercase();

        if lower.starts_with("preloader_") && lower.ends_with(".bin") {
            preloaders.push(name.to_string());
        }
    }

    if preloaders.len() == 1 {
        preloaders.pop()
    } else {
        None
    }
}

fn patch_missing_preloader_blocks(
    source_text: &str,
    image_dir: &Path,
    fallback_preloader: &str,
) -> String {
    let lower = source_text.to_ascii_lowercase();
    let start_token = "<partition_index";
    let end_token = "</partition_index>";

    let mut result = String::new();
    let mut cursor = 0usize;

    while let Some(relative_start) = lower[cursor..].find(start_token) {
        let start = cursor + relative_start;

        result.push_str(&source_text[cursor..start]);

        let Some(relative_end) = lower[start..].find(end_token) else {
            result.push_str(&source_text[start..]);
            return result;
        };

        let end = start + relative_end + end_token.len();
        let block = &source_text[start..end];

        let patched_block = if let Some(partition_name) =
            extract_tag_text_case_insensitive(block, "partition_name")
        {
            let partition_name = partition_name.trim().to_ascii_lowercase();

            if partition_name.starts_with("preloader") {
                let file_name = extract_tag_text_case_insensitive(block, "file_name")
                    .unwrap_or_default()
                    .trim()
                    .to_string();

                if file_name.eq_ignore_ascii_case("NONE")
                    || file_name.is_empty()
                    || !image_dir.join(&file_name).is_file()
                {
                    replace_or_insert_tag_text(block, "partition_index", "file_name", fallback_preloader)
                } else {
                    block.to_string()
                }
            } else {
                block.to_string()
            }
        } else {
            block.to_string()
        };

        result.push_str(&patched_block);
        cursor = end;
    }

    result.push_str(&source_text[cursor..]);
    result
}

fn patch_scatter_xml_text(
    source_text: &str,
    patched_partitions: &HashMap<String, ScatterPartition>,
) -> String {
    patch_scatter_blocks_by_tag(source_text, patched_partitions, "partition_index")
}

fn ensure_proinfo_download_block(source_text: &str) -> String {
    if source_text
        .to_ascii_lowercase()
        .contains("<partition_name>proinfo</partition_name>")
    {
        return force_proinfo_download_values(source_text);
    }

    let proinfo_block = if source_text.to_ascii_uppercase().contains("HW_STORAGE_UFS") {
        proinfo_partition_block_ufs()
    } else {
        proinfo_partition_block_emmc()
    };

    if let Some(index) = source_text.rfind("</partition_index>") {
        let insert_at = index + "</partition_index>".len();

        return format!(
            "{}\n{}\n{}",
            &source_text[..insert_at],
            proinfo_block,
            &source_text[insert_at..]
        );
    }

    format!("{source_text}\n{proinfo_block}\n")
}

fn force_proinfo_download_values(source_text: &str) -> String {
    let lower = source_text.to_ascii_lowercase();
    let start_token = "<partition_index";
    let end_token = "</partition_index>";

    let mut result = String::new();
    let mut cursor = 0usize;

    while let Some(relative_start) = lower[cursor..].find(start_token) {
        let start = cursor + relative_start;

        result.push_str(&source_text[cursor..start]);

        let Some(relative_end) = lower[start..].find(end_token) else {
            result.push_str(&source_text[start..]);
            return result;
        };

        let end = start + relative_end + end_token.len();
        let block = &source_text[start..end];

        if block
            .to_ascii_lowercase()
            .contains("<partition_name>proinfo</partition_name>")
        {
            let mut patched = block.to_string();

            patched = replace_or_insert_tag_text(&patched, "partition_index", "file_name", "proinfo");
            patched = replace_or_insert_tag_text(&patched, "partition_index", "is_download", "true");
            patched = replace_or_insert_tag_text(&patched, "partition_index", "is_upgradable", "false");

            result.push_str(&patched);
        } else {
            result.push_str(block);
        }

        cursor = end;
    }

    result.push_str(&source_text[cursor..]);
    result
}

fn force_proinfo_disabled_values(source_text: &str) -> String {
    let lower = source_text.to_ascii_lowercase();
    let start_token = "<partition_index";
    let end_token = "</partition_index>";

    let mut result = String::new();
    let mut cursor = 0usize;

    while let Some(relative_start) = lower[cursor..].find(start_token) {
        let start = cursor + relative_start;

        result.push_str(&source_text[cursor..start]);

        let Some(relative_end) = lower[start..].find(end_token) else {
            result.push_str(&source_text[start..]);
            return result;
        };

        let end = start + relative_end + end_token.len();
        let block = &source_text[start..end];

        if block
            .to_ascii_lowercase()
            .contains("<partition_name>proinfo</partition_name>")
        {
            let mut patched = block.to_string();

            patched = replace_or_insert_tag_text(&patched, "partition_index", "file_name", "NONE");
            patched = replace_or_insert_tag_text(&patched, "partition_index", "is_download", "false");
            patched = replace_or_insert_tag_text(&patched, "partition_index", "is_upgradable", "false");

            result.push_str(&patched);
        } else {
            result.push_str(block);
        }

        cursor = end;
    }

    result.push_str(&source_text[cursor..]);
    result
}

fn proinfo_partition_block_emmc() -> &'static str {
    r#"    <partition_index name="SYS49">
      <partition_name>proinfo</partition_name>
      <file_name>proinfo</file_name>
      <is_download>true</is_download>
      <type>NORMAL_ROM</type>
      <linear_start_addr>0x3d200000</linear_start_addr>
      <physical_start_addr>0x3d200000</physical_start_addr>
      <partition_size>0x300000</partition_size>
      <region>EMMC_USER</region>
      <storage>HW_STORAGE_EMMC</storage>
      <boundary_check>true</boundary_check>
      <is_reserved>false</is_reserved>
      <operation_type>PROTECTED</operation_type>
      <is_upgradable>false</is_upgradable>
      <empty_boot_needed>false</empty_boot_needed>
      <combo_partsize_check>false</combo_partsize_check>
      <reserve>0x00</reserve>
    </partition_index>"#
}

fn proinfo_partition_block_ufs() -> &'static str {
    r#"    <partition_index name="SYS49">
      <partition_name>proinfo</partition_name>
      <file_name>proinfo</file_name>
      <is_download>true</is_download>
      <type>NORMAL_ROM</type>
      <linear_start_addr>0x3d200000</linear_start_addr>
      <physical_start_addr>0x3d200000</physical_start_addr>
      <partition_size>0x300000</partition_size>
      <region>UFS_LU2</region>
      <storage>HW_STORAGE_UFS</storage>
      <boundary_check>true</boundary_check>
      <is_reserved>false</is_reserved>
      <operation_type>PROTECTED</operation_type>
      <is_upgradable>false</is_upgradable>
      <empty_boot_needed>false</empty_boot_needed>
      <combo_partsize_check>false</combo_partsize_check>
      <reserve>0x00</reserve>
    </partition_index>"#
}


fn patch_scatter_blocks_by_tag(
    source_text: &str,
    patched_partitions: &HashMap<String, ScatterPartition>,
    block_tag: &str,
) -> String {
    let lower = source_text.to_ascii_lowercase();
    let start_token = format!("<{block_tag}");
    let end_token = format!("</{block_tag}>");

    let mut result = String::new();
    let mut cursor = 0usize;

    while let Some(relative_start) = lower[cursor..].find(&start_token) {
        let start = cursor + relative_start;

        result.push_str(&source_text[cursor..start]);

        let Some(relative_end) = lower[start..].find(&end_token) else {
            result.push_str(&source_text[start..]);
            return result;
        };

        let end = start + relative_end + end_token.len();
        let block = &source_text[start..end];

        let patched_block = if let Some(name) =
            extract_tag_text_case_insensitive(block, "partition_name")
        {
            let key = name.trim().to_ascii_lowercase();

            if let Some(partition) = patched_partitions.get(&key) {
                patch_partition_block(block, partition, block_tag)
            } else {
                block.to_string()
            }
        } else {
            block.to_string()
        };

        result.push_str(&patched_block);
        cursor = end;
    }

    result.push_str(&source_text[cursor..]);
    result
}

fn patch_partition_block(
    block: &str,
    partition: &ScatterPartition,
    block_tag: &str,
) -> String {
    let mut out = block.to_string();

    if let Some(value) = &partition.file_name {
        out = replace_or_insert_tag_text(&out, block_tag, "file_name", value);
    }

    if let Some(value) = &partition.is_download {
        out = replace_or_insert_tag_text(&out, block_tag, "is_download", value);
    }

    if let Some(value) = &partition.is_upgradable {
        out = replace_or_insert_tag_text(&out, block_tag, "is_upgradable", value);
    }

    out
}

fn replace_or_insert_tag_text(block: &str, block_tag: &str, tag: &str, value: &str) -> String {
    let lower = block.to_ascii_lowercase();
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");

    if let Some(start) = lower.find(&start_tag) {
        let value_start = start + start_tag.len();

        if let Some(relative_end) = lower[value_start..].find(&end_tag) {
            let value_end = value_start + relative_end;

            return format!(
                "{}{}{}",
                &block[..value_start],
                value,
                &block[value_end..]
            );
        }
    }

    let close_tag = format!("</{block_tag}>");

    if let Some(end) = lower.rfind(&close_tag) {
        return format!(
            "{}    <{tag}>{value}</{tag}>\n{}",
            &block[..end],
            &block[end..]
        );
    }

    block.to_string()
}

fn write_lpmbox_flash_xml(
    source_flash_xml: &Path,
    work_flash_xml: &Path,
    work_scatter_xml: &Path,
) -> Result<()> {
    let source_text = read_text_lossy(source_flash_xml)?;
    let flash_dir = source_flash_xml
        .parent()
        .unwrap_or_else(|| Path::new("."));

    let scatter_ref = relative_scatter_reference(flash_dir, work_scatter_xml);
    let patched_text = replace_flash_scatter_reference(&source_text, &scatter_ref)?;

    fs::write(work_flash_xml, patched_text.as_bytes())?;

    Ok(())
}

fn relative_scatter_reference(flash_dir: &Path, scatter_path: &Path) -> String {
    let Some(scatter_name) = scatter_path.file_name().and_then(|name| name.to_str()) else {
        return scatter_path.display().to_string();
    };

    if scatter_path.parent() == Some(flash_dir) {
        return scatter_name.to_string();
    }

    if let Some(parent) = flash_dir.parent() {
        if scatter_path.parent() == Some(parent) {
            return format!("../{scatter_name}");
        }
    }

    scatter_path.display().to_string()
}

fn replace_flash_scatter_reference(source_text: &str, scatter_ref: &str) -> Result<String> {
    let lower = source_text.to_ascii_lowercase();

    let start_tag = "<scatter>";
    let end_tag = "</scatter>";

    let start = lower.find(start_tag).ok_or_else(|| {
        LpmError::InvalidFirmwareFolder("flash.xml에서 <scatter> 항목을 찾지 못했습니다.".to_string())
    })?;

    let value_start = start + start_tag.len();

    let relative_end = lower[value_start..].find(end_tag).ok_or_else(|| {
        LpmError::InvalidFirmwareFolder("flash.xml에서 </scatter> 항목을 찾지 못했습니다.".to_string())
    })?;

    let value_end = value_start + relative_end;

    Ok(format!(
        "{}{}{}",
        &source_text[..value_start],
        scatter_ref,
        &source_text[value_end..]
    ))
}

fn extract_zip_file_to_dir(zip_path: &Path, target_dir: &Path) -> Result<()> {
    let file = fs::File::open(zip_path)?;

    let mut archive = zip::ZipArchive::new(file).map_err(|err| {
        LpmError::InvalidFirmwareFolder(format!(
            "모델별 lk/dtbo ZIP 열기 실패: {} / {err}",
            zip_path.display()
        ))
    })?;

    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(|err| {
            LpmError::InvalidFirmwareFolder(format!(
                "모델별 lk/dtbo ZIP 항목 읽기 실패: {} / {err}",
                zip_path.display()
            ))
        })?;

        let Some(enclosed_name) = file.enclosed_name().map(|path| path.to_path_buf()) else {
            continue;
        };

        let out_path = target_dir.join(enclosed_name);

        if file.is_dir() {
            fs::create_dir_all(&out_path)?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut out_file = fs::File::create(&out_path)?;
        io::copy(&mut file, &mut out_file)?;
    }

    Ok(())
}

fn download_binary_file(url: &str, out_path: &Path) -> Result<()> {
    let response = ureq::get(url)
        .set("User-Agent", "Mozilla/5.0")
        .call()
        .map_err(|err| {
            LpmError::InvalidFirmwareFolder(format!(
                "모델별 lk/dtbo ZIP 다운로드 실패: {url} / {err}"
            ))
        })?;

    if !(200..300).contains(&response.status()) {
        return Err(LpmError::InvalidFirmwareFolder(format!(
            "모델별 lk/dtbo ZIP 다운로드 HTTP 오류: {} / {url}",
            response.status()
        )));
    }

    let mut reader = response.into_reader();
    let mut file = fs::File::create(out_path)?;

    io::copy(&mut reader, &mut file)?;

    let size = fs::metadata(out_path)?.len();

    if size == 0 {
        return Err(LpmError::InvalidFirmwareFolder(format!(
            "모델별 lk/dtbo ZIP 파일 크기가 0입니다: {}",
            out_path.display()
        )));
    }

    Ok(())
}

fn find_file_recursively(root: &Path, file_name: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(root).ok()?;

    for entry in entries.filter_map(|entry| entry.ok()) {
        let path = entry.path();

        if path.is_file() {
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.eq_ignore_ascii_case(file_name))
                .unwrap_or(false)
            {
                return Some(path);
            }

            continue;
        }

        if path.is_dir() {
            if let Some(found) = find_file_recursively(&path, file_name) {
                return Some(found);
            }
        }
    }

    None
}

#[cfg(test)]
mod unified_lk_scatter_tests{
    use super::*;
    fn part(name:&str,file:&str,enabled:bool)->ScatterPartition{
        ScatterPartition{
            name:name.into(),file_name:Some(file.into()),
            is_download:Some(if enabled{"true"}else{"false"}.into()),
            partition_index:None,is_upgradable:Some("true".into()),
            partition_type:None,storage:None,linear_start_addr:None,
            physical_start_addr:None,partition_size:None
        }
    }
    #[test]
    fn mt6835_scatters_always_disable_lk_b(){
        for model in ["TB335FC","TB336FU","TB365FC","TB361FU"]{
            let mut p=vec![part("lk_a","original",false),part("lk_b","original",true),
                part("dtbo_a","dtbo_a",true)];
            apply_unified_patched_lk_to_scatter(
                &mut p,model,lk_patch::LPMBOX_PATCHED_LK_FILENAME
            ).unwrap();
            assert_eq!(p[0].file_name.as_deref(),Some("LPMBOX_lk.img"));
            assert_eq!(p[0].is_download.as_deref(),Some("true"));
            assert_eq!(p[1].file_name.as_deref(),Some("LPMBOX_lk.img"));
            assert_eq!(p[1].is_download.as_deref(),Some("false"));
            assert_eq!(p[1].is_upgradable.as_deref(),Some("false"));
            assert_eq!(p[2].file_name.as_deref(),Some("dtbo_a"));
            let xml="<partition_index><partition_name>lk_b</partition_name><file_name>lk_b</file_name><is_download>true</is_download></partition_index>";
            let map=HashMap::from([("lk_b".into(),p[1].clone())]);
            let out=patch_scatter_xml_text(xml,&map);
            assert!(out.contains("<file_name>LPMBOX_lk.img</file_name>"));
            assert!(out.contains("<is_download>false</is_download>"));
        }
    }
    #[test]
    fn tb375_tb373_both_scatter_entries_target_verified_lk(){
        for model in ["TB375FC","TB373FU"]{
            let mut p=vec![part("lk_a","original",false),part("lk_b","original",false)];
            apply_unified_patched_lk_to_scatter(
                &mut p,model,lk_patch::LPMBOX_PATCHED_LK_FILENAME
            ).unwrap();
            for slot in p{
                assert_eq!(slot.file_name.as_deref(),Some("LPMBOX_lk.img"));
                assert_eq!(slot.is_download.as_deref(),Some("true"));
            }
        }
    }
    #[test]
    fn even_baseline_prc_never_downloads_mt6835_lk_b(){
        for model in ["TB335FC","TB336FU","TB365FC","TB361FU"]{
            let mut partitions=vec![
                part("lk_a","lk_a",true),part("lk_b","lk_b",true),
                part("boot_a","boot_a",true)
            ];
            assert_eq!(protect_mt6835_lk_b(&mut partitions,model),2);
            assert_eq!(partitions[0].is_download.as_deref(),Some("true"));
            assert_eq!(partitions[1].file_name.as_deref(),Some("lk_b"));
            assert_eq!(partitions[1].is_download.as_deref(),Some("false"));
            assert_eq!(partitions[1].is_upgradable.as_deref(),Some("false"));
            assert_eq!(partitions[2].is_download.as_deref(),Some("true"));
            assert_eq!(protect_mt6835_lk_b(&mut partitions,model),0);
        }
        let mut different=vec![part("lk_b","lk_b",true)];
        assert_eq!(protect_mt6835_lk_b(&mut different,"TB373FU"),0);
    }
    #[test]
    fn invalid_names_and_missing_slots_are_rejected(){
        assert!(apply_unified_patched_lk_to_scatter(
            &mut [part("lk_a","lk_a",true),part("lk_b","lk_b",false)],
            "TB336FU","lk_a").is_err());
        assert!(apply_unified_patched_lk_to_scatter(
            &mut [part("lk_a","lk_a",true)],"TB336FU","LPMBOX_lk.img").is_err());
        assert!(apply_unified_patched_lk_to_scatter(
            &mut [part("lk_a","lk_a",true),part("lk_a","lk_a",true),
                part("lk_b","lk_b",true)],"TB336FU","LPMBOX_lk.img").is_err());
    }
}
