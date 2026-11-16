//! Read-only live LK partition dump via the existing SP Flash Tool V6 worker.
//! This module never writes any tablet partition. A/B selection is supplied by
//! a freshly queried Android ro.boot.slot_suffix property.
use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn checked_partition(slot: &str) -> Result<&'static str> {
    match slot.trim() {
        "_a" | "a" | "lk_a" => Ok("lk_a"),
        "_b" | "b" | "lk_b" => Ok("lk_b"),
        _ => Err(LpmError::Spft(format!(
            "지원하지 않는 LK 슬롯 '{slot}': ro.boot.slot_suffix가 _a/_b여야 합니다."
        )))
    }
}

fn active_lk_readback_xml(
    flash_xml: &Path, log_dir: &Path, target: &Path, partition: &str,
) -> Result<String> {
    let partition = checked_partition(partition)?;
    let original = build_readback_config_xml(flash_xml, log_dir, target);
    let needle = "partition=\"proinfo\"";
    if original.matches(needle).count() != 1 {
        return Err(LpmError::Spft("SPFT 리드백 템플릿 검증 실패".into()));
    }
    Ok(original.replacen(needle, &format!("partition=\"{partition}\""), 1))
}

pub fn prepare_active_lk_readback(
    selected_path: &Path, running_slot: &str
) -> Result<ProinfoReadbackPlan> {
    let partition = checked_partition(running_slot)?;
    let (_firmware_root_dir, image_dir) = resolve_root_and_image_dir(selected_path)?;
    let root_dir = app_paths::runtime_root();
    let embedded_tool_dir = ensure_embedded_spflashtoolv6_tool()?;
    let spft_exe = embedded_tool_dir.join("SPFlashToolV6.exe");
    if !spft_exe.is_file() {
        return Err(LpmError::FileNotFound(spft_exe.display().to_string()));
    }
    let flash_xml = find_flash_xml(&image_dir).ok_or_else(|| {
        LpmError::FileNotFound(format!(
            "LK 리드백용 flash.xml을 찾지 못했습니다: {}", image_dir.display()
        ))
    })?;
    let workspace = ascii_readback_workspace()?;
    let log_dir = workspace.join("backup").join("spft_log");
    let config_xml = workspace.join("lk_readback_config.xml");
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)
        .map_err(|err| LpmError::Spft(err.to_string()))?
        .as_millis();
    let file_name = format!("{partition}_{nonce}_{}.bin", std::process::id());
    let staged_lk = workspace.join("backup").join(&file_name);
    let backup_dir = app_paths::backup_root().join("lk_readback");
    let lk_out = backup_dir.join(&file_name);

    require_spft_ascii_path(&flash_xml, "LK readback flash.xml")?;
    require_spft_ascii_path(&spft_exe, "SPFlashToolV6.exe")?;
    require_spft_ascii_path(&workspace, "LK readback 임시 폴더")?;
    fs::create_dir_all(&log_dir)?;
    fs::create_dir_all(&backup_dir)?;
    // Do not touch backup/proinfo or any other readback snapshots.
    let xml = active_lk_readback_xml(&flash_xml, &log_dir, &staged_lk, partition)?;
    fs::write(&config_xml, xml.as_bytes())?;
    Ok(ProinfoReadbackPlan {
        root_dir, image_dir, embedded_tool_used: true,
        embedded_tool_dir, spft_exe, flash_xml, backup_dir,
        log_dir, config_xml, proinfo_out: lk_out, config_xml_size: xml.len()
    })
}

pub struct PrearmedActiveLkReadback {
    inner: PrearmedProinfoReadback,
}

impl PrearmedActiveLkReadback {
    pub fn start(selected_path: &Path, running_slot: &str) -> Result<Self> {
        let plan = prepare_active_lk_readback(selected_path, running_slot)?;
        Ok(Self {
            inner: PrearmedProinfoReadback::start_from_plan(plan)?
        })
    }

    pub fn finish<F>(self, on_event: F) -> Result<PathBuf>
    where F: FnMut(ProinfoLiveEvent) {
        let result = self.inner.finish(on_event)?;
        if !result.proinfo_exists || !(3_372_368..=32 * 1024 * 1024)
            .contains(&result.proinfo_size) {
            return Err(LpmError::Spft(format!(
                "활성 LK 리드백 크기 오류: {} ({}바이트), 플래싱을 중단합니다.",
                result.plan.proinfo_out.display(), result.proinfo_size
            )));
        }
        Ok(result.plan.proinfo_out)
    }

    pub fn cancel(self) {
        self.inner.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_active_ab_lk_is_accepted() {
        assert_eq!(checked_partition("_a").unwrap(), "lk_a");
        assert_eq!(checked_partition("_b").unwrap(), "lk_b");
        for invalid in ["", "_c", "proinfo", "lk", "lk_a; FORMAT", "../lk_b"] {
            assert!(checked_partition(invalid).is_err());
        }
    }

    #[test]
    fn generated_lk_readback_uses_read_only_partition_command() {
        let xml = active_lk_readback_xml(
            Path::new(r"G:\ROM\image\download_agent\flash.xml"),
            Path::new(r"G:\LPMBox_SPFT_Readback\backup\spft_log"),
            Path::new(r"G:\LPMBox_SPFT_Readback\backup\lk_a.bin"),
            "lk_a",
        ).unwrap();
        assert!(xml.contains("<READ-PARTITION version=\"1.0\" partition=\"lk_a\">"));
        assert!(xml.contains("<REBOOT version=\"1.0\" action=\"COLD-RESET\"/>"));
        assert!(!xml.contains("partition=\"proinfo\""));
        assert!(!xml.contains("WRITE-PARTITION"));
        assert!(!xml.contains("FORMAT"));
    }
}
