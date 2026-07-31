use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RomRegion {
    Prc,
    Row,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallMode {
    ConvertWipe,
    RowUpdateKeepData,
    ReinstallWipe,
    CountryReset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatchPlanMode {
    ConvertWipe,
    RowUpdateKeepData,
    ReinstallWipe,
    CountryReset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScatterPartition {
    pub name: String,
    pub partition_index: Option<String>,
    pub file_name: Option<String>,
    pub is_download: Option<String>,
    pub is_upgradable: Option<String>,
    pub partition_type: Option<String>,
    pub storage: Option<String>,
    pub linear_start_addr: Option<String>,
    pub physical_start_addr: Option<String>,
    pub partition_size: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredPartitionCheck {
    pub has_proinfo: bool,
    pub has_userdata: bool,
    pub has_super: bool,
    pub has_boot_ab: bool,
    pub has_vendor_boot_ab: bool,
    pub has_init_boot_ab: bool,
    pub has_vbmeta_ab: bool,
    pub has_vbmeta_system_ab: bool,
    pub has_vbmeta_vendor_ab: bool,
    pub has_lk_ab: bool,
    pub has_dtbo_ab: bool,
    pub all_required_ok: bool,
    pub missing_partitions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionPatchAction {
    pub partition: String,
    pub target_file_name: Option<String>,
    pub target_is_download: Option<String>,
    pub target_is_upgradable: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchPlan {
    pub mode: PatchPlanMode,
    pub title: String,
    pub available: bool,
    pub warnings: Vec<String>,
    pub actions: Vec<PartitionPatchAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchedPartitionSnapshot {
    pub mode: PatchPlanMode,
    pub title: String,
    pub available: bool,
    pub changed_count: usize,
    pub partitions: Vec<ScatterPartition>,
    pub summary: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchValidationResult {
    pub mode: PatchPlanMode,
    pub title: String,
    pub passed: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScatterXmlInfo {
    pub root_name: String,
    pub xml_size: usize,
    pub partition_count: usize,
    pub partition_names: Vec<String>,
    pub partitions: Vec<ScatterPartition>,
    pub required_check: RequiredPartitionCheck,
    pub patch_plans: Vec<PatchPlan>,
    pub patched_snapshots: Vec<PatchedPartitionSnapshot>,
    pub patch_validations: Vec<PatchValidationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedFirmwareCheck {
    pub checked: bool,
    pub blocked: bool,
    pub source: String,
    pub model: String,
    pub version: Option<String>,
    pub blocked_versions: Vec<String>,
    pub matched_version: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareInfo {
    pub image_dir: PathBuf,
    pub model: String,
    pub version: Option<String>,
    pub region: RomRegion,
    pub platform: Option<String>,
    pub blocked_firmware_check: BlockedFirmwareCheck,
    pub flash_xml: Option<PathBuf>,
    pub scatter_xml: Option<PathBuf>,
    pub scatter_xml_info: Option<ScatterXmlInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashPlan {
    pub mode: InstallMode,
    pub patch_mode: PatchPlanMode,
    pub title: String,

    pub root_dir: PathBuf,
    pub image_dir: PathBuf,
    pub work_dir: PathBuf,
    pub work_download_agent_dir: PathBuf,

    pub source_flash_xml: PathBuf,
    pub source_scatter: PathBuf,
    pub work_flash_xml: PathBuf,
    pub work_scatter_xml: PathBuf,
    pub da_path: PathBuf,

    pub platform: String,
    pub model: String,
    pub image_version: Option<String>,
    pub image_region: RomRegion,

    pub keep_user_data: bool,
    pub requires_current_slot_stage: bool,
    pub requires_device_stage: bool,
    pub requires_proinfo_patch: bool,

    pub spft_command_preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashPreparedOutput {
    pub firmware: FirmwareInfo,
    pub plan: FlashPlan,
    pub selected_patch_plan: PatchPlan,
    pub selected_patch_validation: Option<PatchValidationResult>,
    pub generated_scatter_info: ScatterXmlInfo,
    pub changed_count: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProinfoReadbackPlan {
    pub root_dir: PathBuf,
    pub image_dir: PathBuf,

    pub embedded_tool_used: bool,
    pub embedded_tool_dir: PathBuf,
    pub spft_exe: PathBuf,

    pub flash_xml: PathBuf,
    pub backup_dir: PathBuf,
    pub log_dir: PathBuf,
    pub config_xml: PathBuf,
    pub proinfo_out: PathBuf,
    pub config_xml_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProinfoReadbackResult {
    pub plan: ProinfoReadbackPlan,
    pub exit_code: Option<i32>,
    pub stdout_tail: String,
    pub stderr_tail: String,
    pub proinfo_exists: bool,
    pub proinfo_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpftProgress {
    pub stage: String,
    pub percent: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProinfoLiveEvent {
    Log(String),
    Progress(SpftProgress),
    Finished(ProinfoReadbackResult),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreloaderDetectResult {
    pub detected: bool,
    pub display_name: Option<String>,
    pub method: Option<String>,
    pub checked_tokens: Vec<String>,
}
