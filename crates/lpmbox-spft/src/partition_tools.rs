//! MediaTek-only partition tools, separate from the established ROM flash flow.
//! LTBox's Qualcomm EDL implementation is not compatible with this MTK SPFT backend.
use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) const FLASH_ALLOWED: &[&str] = &[
    "lk_a", "dtbo_a", "boot_a", "init_boot_a", "vendor_boot_a",
    "vbmeta_a", "vbmeta_system_a", "vbmeta_vendor_a", "super",
    "logo_a", "tee_a",
];

pub(crate) fn partition_name(name: &str) -> Result<&str> {
    if name.len() < 3 || name.len() > 48 || !name.bytes().all(|c|
        c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
    {
        return Err(LpmError::Spft(
            "파티션명은 영문 소문자·숫자·밑줄만 허용됩니다(3~48자).".into()
        ));
    }
    Ok(name)
}

pub(crate) fn xml_tag<'a>(s: &'a str, key: &str) -> Option<&'a str> {
    let start_tag = format!("<{key}>");
    let end_tag = format!("</{key}>");
    let start = s.find(&start_tag)? + start_tag.len();
    let end = start + s[start..].find(&end_tag)?;
    Some(s[start..end].trim())
}

pub(crate) fn set_xml_tag(source: &str, tag: &str, value: &str) -> Result<String> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");
    let start = source.find(&start_tag).ok_or_else(|| LpmError::Spft(
        format!("scatter XML에 {tag} 요소가 없습니다.")
    ))? + start_tag.len();
    let end = start + source[start..].find(&end_tag).ok_or_else(|| LpmError::Spft(
        format!("scatter XML의 {tag} 종료 요소가 없습니다.")
    ))?;
    let mut out = String::with_capacity(source.len() + value.len());
    out.push_str(&source[..start]);
    out.push_str(value);
    out.push_str(&source[end..]);
    Ok(out)
}

pub fn prepare_partition_dump(
    selected_path: &Path,
    partition: &str,
    destination_dir: &Path,
) -> Result<ProinfoReadbackPlan> {
    let partition = partition_name(partition)?;
    let (_, image_dir) = resolve_root_and_image_dir(selected_path)?;
    let flash_xml = find_flash_xml(&image_dir)
        .ok_or_else(|| LpmError::FileNotFound("image/download_agent/flash.xml".into()))?;
    require_spft_ascii_path(&flash_xml, "SPFT flash.xml")?;
    let embedded_tool_dir = ensure_embedded_spflashtoolv6_tool()?;
    let spft_exe = embedded_tool_dir.join("SPFlashToolV6.exe");
    let workspace = ascii_readback_workspace()?;
    let log_dir = workspace.join("backup").join("spft_log");
    let config_xml = workspace.join("partition_readback_config.xml");
    fs::create_dir_all(&log_dir)?;
    fs::create_dir_all(destination_dir)?;
    let mut proinfo_out = destination_dir.join(format!("{partition}.img"));
    for i in 1..1000 {
        if !proinfo_out.exists() { break; }
        proinfo_out = destination_dir.join(format!("{partition}_{i:03}.img"));
    }
    if proinfo_out.exists() {
        return Err(LpmError::Spft("파티션 덤프 출력 파일 이름을 확보할 수 없습니다.".into()));
    }
    let staged = workspace.join("backup").join(proinfo_out.file_name()
        .ok_or_else(|| LpmError::Spft("출력 파일 이름 오류".into()))?);
    let config = build_readback_config_xml(&flash_xml, &log_dir, &staged)
        .replace("partition=\"proinfo\"", &format!("partition=\"{partition}\""));
    fs::write(&config_xml, &config)?;
    Ok(ProinfoReadbackPlan {
        root_dir: app_paths::runtime_root(),
        image_dir,
        embedded_tool_used: true,
        embedded_tool_dir,
        spft_exe,
        flash_xml,
        backup_dir: destination_dir.to_path_buf(),
        log_dir,
        config_xml,
        proinfo_out,
        config_xml_size: config.len(),
    })
}

pub fn start_partition_dump(
    selected_path: &Path, partition: &str, destination_dir: &Path
) -> Result<PrearmedProinfoReadback> {
    let plan = prepare_partition_dump(selected_path, partition, destination_dir)?;
    PrearmedProinfoReadback::start_from_plan(plan)
}

pub(crate) fn find_scatter_xml(image_dir: &Path) -> Result<PathBuf> {
    let found = fs::read_dir(image_dir)?.filter_map(|e|e.ok())
        .map(|e|e.path())
        .filter(|p|p.is_file() && p.file_name().and_then(|n|n.to_str())
            .is_some_and(|n|n.ends_with("_Android_scatter.xml")))
        .collect::<Vec<_>>();

    // Firmware packages can contain both MT6835_Android_scatter.xml and a
    // derived LPMBox_MT6835_Android_scatter.xml. The only authoritative
    // target is the original scatter listed in image/download_agent/flash.xml.
    if let Some(flash_xml)=find_flash_xml(image_dir) {
        let config=fs::read_to_string(&flash_xml)?;
        if let Some(target)=xml_tag(&config,"scatter") {
            let normalized=target.trim().replace('\\',"/");
            let prefix_ok=normalized.starts_with("../")
                && !normalized[3..].contains('/');
            let name=normalized.strip_prefix("../").unwrap_or("");
            if !prefix_ok || name.starts_with("LPMBox_")
                || !name.ends_with("_Android_scatter.xml")
                || name.contains("..") {
                return Err(LpmError::Spft(format!(
                    "flash.xml의 원본 scatter 지정이 안전하지 않습니다: {target}"
                )));
            }
            let selected=image_dir.join(name);
            if !selected.is_file() {
                return Err(LpmError::Spft(format!(
                    "flash.xml이 가리키는 원본 scatter가 없습니다: {}",
                    selected.display()
                )));
            }
            return Ok(selected);
        }
    }

    // Older images without an explicit reference must contain one and only
    // one original. Additional derived LPMBox files are never considered
    // an ambiguous original.
    let originals=found.into_iter().filter(|p|p.file_name()
        .and_then(|n|n.to_str())
        .is_some_and(|n|!n.starts_with("LPMBox_")))
        .collect::<Vec<_>>();
    if originals.len()!=1 {
        return Err(LpmError::Spft(format!(
            "image 폴더에 원본 Android scatter.xml이 1개여야 합니다. 원본 후보 {}개.",
            originals.len()
        )));
    }
    Ok(originals[0].clone())
}

fn create_target_scatter(
    original: &str, target: &str, staged_file: &str, file_size: u64
) -> Result<String> {
    let mut rewritten = String::with_capacity(original.len());
    let mut rest = original;
    let mut match_count = 0usize;
    while let Some(start) = rest.find("<partition_index ") {
        rewritten.push_str(&rest[..start]);
        rest = &rest[start..];
        let end = rest.find("</partition_index>").ok_or_else(|| LpmError::Spft(
            "scatter XML partition_index 종료 요소가 누락됐습니다.".into()
        ))? + "</partition_index>".len();
        let mut block = rest[..end].to_string();
        let name = xml_tag(&block, "partition_name").unwrap_or_default().to_string();
        if name == target {
            match_count += 1;
            let size = xml_tag(&block,"partition_size")
                .and_then(|hex|u64::from_str_radix(hex.trim_start_matches("0x"),16).ok())
                .ok_or_else(|| LpmError::Spft("파티션 크기를 확인할 수 없습니다.".into()))?;
            if file_size == 0 || file_size > size {
                return Err(LpmError::Spft(format!(
                    "{target}: 이미지 {file_size}B / 파티션 {size}B. 크기 조건 불충족."
                )));
            }
            block = set_xml_tag(&block, "file_name", staged_file)?;
            block = set_xml_tag(&block, "is_download", "true")?;
            block = set_xml_tag(&block, "operation_type", "UPDATE")?;
        } else {
            block = set_xml_tag(&block, "is_download", "false")?;
        }
        rewritten.push_str(&block);
        rest = &rest[end..];
    }
    rewritten.push_str(rest);
    if match_count != 1 {
        return Err(LpmError::Spft(format!(
            "선택한 {target} 파티션을 scatter에서 단 한 번 찾지 못했습니다: {match_count}개."
        )));
    }
    Ok(rewritten)
}

pub fn execute_partition_flash_streaming<F>(
    selected_path: &Path,
    partition: &str,
    image_path: &Path,
    mut on_event: F,
) -> Result<()>
where F: FnMut(ProinfoLiveEvent),
{
    let partition = partition_name(partition)?;
    if !FLASH_ALLOWED.contains(&partition) {
        return Err(LpmError::Spft(format!(
            "{partition} 플래싱 제한: 검증되지 않은 고위험/보안·기기고유 파티션은 지원하지 않습니다."
        )));
    }
    if !image_path.is_file() {
        return Err(LpmError::FileNotFound(image_path.display().to_string()));
    }
    let (_, image_dir) = resolve_root_and_image_dir(selected_path)?;
    let flash_xml = find_flash_xml(&image_dir)
        .ok_or_else(|| LpmError::FileNotFound("flash.xml".into()))?;
    let source_scatter = find_scatter_xml(&image_dir)?;
    let source_flash = fs::read_to_string(&flash_xml)?;
    let dagent = xml_tag(&source_flash,"dagent")
        .ok_or_else(|| LpmError::Spft("flash.xml에 DA 파일명이 없습니다.".into()))?;
    if dagent.contains('/') || dagent.contains('\\') || dagent.contains("..") {
        return Err(LpmError::Spft("DA 경로에 상위 경로 이동이 포함돼 있습니다.".into()));
    }
    let da_source = flash_xml.parent()
        .ok_or_else(|| LpmError::Spft("DA 원본 경로 오류".into()))?.join(dagent);
    if !da_source.is_file() { return Err(LpmError::FileNotFound(da_source.display().to_string())); }
    let workspace = ascii_readback_workspace()?.join(format!(
        "partition_flash_{}",SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()
    ));
    let agent_dir = workspace.join("download_agent");
    fs::create_dir_all(&agent_dir)?;
    let staged_image = workspace.join("partition.img");
    fs::copy(image_path,&staged_image)?;
    let source_scatter_text = fs::read_to_string(&source_scatter)?;
    let custom_scatter = create_target_scatter(&source_scatter_text, partition,
        "partition.img", fs::metadata(&staged_image)?.len())?;
    let scatter_filename=source_scatter.file_name()
        .ok_or_else(|| LpmError::Spft("scatter 이름 누락".into()))?;
    let staged_scatter = workspace.join(scatter_filename);
    fs::write(&staged_scatter, custom_scatter)?;
    let staged_flash = agent_dir.join("flash.xml");
    fs::copy(&flash_xml, &staged_flash)?;
    let staged_da = agent_dir.join(dagent);
    fs::copy(&da_source,&staged_da)?;

    let target_ref = xml_tag(&source_flash,"scatter")
        .ok_or_else(|| LpmError::Spft("flash.xml의 scatter 지정 누락".into()))?;
    if target_ref != format!("../{}",scatter_filename.to_string_lossy()) {
        return Err(LpmError::Spft(format!(
            "flash.xml 상대 경로가 예상 형식과 다릅니다: {target_ref}"
        )));
    }
    on_event(ProinfoLiveEvent::Log(format!(
        "[파티션 플래싱] {partition}, 원본 {}B, 작업용 scatter {}",
        fs::metadata(image_path)?.len(), staged_scatter.display()
    )));
    on_event(ProinfoLiveEvent::Log(
        "[주의] 이미지 서명, 실기기 모델, AVB/rollback 호환성은 자동 보장하지 않습니다.".into()
    ));
    let embedded_tool_dir=ensure_embedded_spflashtoolv6_tool()?;
    let spft_exe=embedded_tool_dir.join("SPFlashToolV6.exe");
    let mut command=Command::new(&spft_exe);
    command.arg("-a").arg(&staged_da).arg("-f").arg(&staged_flash)
        .arg("-c").arg("download").current_dir(&embedded_tool_dir);
    let result=run_spflashtoolv6_streaming(command, &mut on_event, None,None,None)?;
    if !result.success {
        return Err(LpmError::Spft(format!(
            "선택 파티션 SPFT 플래싱 실패 / exit={:?}: {}",
            result.exit_code,result.stderr_tail
        )));
    }
    on_event(ProinfoLiveEvent::Log(format!(
        "[완료] 파티션 {partition} 플래싱 종료. 이미지 및 파티션 검증은 별도로 필요합니다."
    )));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_scatter_selected_when_derived_scatter_also_exists() {
        use std::time::{SystemTime,UNIX_EPOCH};
        let unique=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let dir=std::env::temp_dir().join(format!(
            "lpmbox_scatter_double_test_{}_{}",std::process::id(),unique));
        let agent=dir.join("download_agent");
        std::fs::create_dir_all(&agent).unwrap();
        let original=dir.join("MT6835_Android_scatter.xml");
        std::fs::write(&original,"<root/>").unwrap();
        std::fs::write(dir.join("LPMBox_MT6835_Android_scatter.xml"),"<root/>").unwrap();
        std::fs::write(agent.join("flash.xml"),
            "<flash-mode><scatter>../MT6835_Android_scatter.xml</scatter></flash-mode>"
        ).unwrap();
        let selected=find_scatter_xml(&dir).unwrap();
        assert_eq!(selected,original);
        std::fs::remove_dir_all(&dir).unwrap();
    }
    #[test]
    fn missing_referenced_original_is_not_replaced_by_derived_scatter() {
        use std::time::{SystemTime,UNIX_EPOCH};
        let unique=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let dir=std::env::temp_dir().join(format!(
            "lpmbox_scatter_missing_test_{}_{}",std::process::id(),unique));
        let agent=dir.join("download_agent");
        std::fs::create_dir_all(&agent).unwrap();
        std::fs::write(dir.join("LPMBox_MT6835_Android_scatter.xml"),"<root/>").unwrap();
        std::fs::write(agent.join("flash.xml"),
            "<flash-mode><scatter>../MT6835_Android_scatter.xml</scatter></flash-mode>"
        ).unwrap();
        assert!(find_scatter_xml(&dir).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }
    #[test]
    fn partition_scatter_writes_only_selected_entry() {
        let doc="<partition_index name=\"SYS0\"><partition_name>boot_a</partition_name><file_name>boot.img</file_name><partition_size>0x400</partition_size><is_download>true</is_download><operation_type>UPDATE</operation_type></partition_index><partition_index name=\"SYS1\"><partition_name>boot_b</partition_name><file_name>boot_b.img</file_name><partition_size>0x400</partition_size><is_download>true</is_download><operation_type>UPDATE</operation_type></partition_index>";
        let out=create_target_scatter(doc,"boot_a","partition.img",512).unwrap();
        assert!(out.contains("<file_name>partition.img</file_name>"));
        assert_eq!(out.matches("<is_download>true</is_download>").count(),1);
        assert_eq!(out.matches("<is_download>false</is_download>").count(),1);
    }
    #[test]
    fn refuses_oversized_and_unknown_targets() {
        let doc="<partition_index name=\"SYS0\"><partition_name>boot_a</partition_name><file_name>boot.img</file_name><partition_size>0x200</partition_size><is_download>true</is_download><operation_type>UPDATE</operation_type></partition_index>";
        assert!(create_target_scatter(doc,"boot_a","x.img",513).is_err());
        assert!(create_target_scatter(doc,"misc","x.img",100).is_err());
    }
}
