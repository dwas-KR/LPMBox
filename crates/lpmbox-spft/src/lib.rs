use lpmbox_core::{
    FlashPlan, LpmError, ProinfoLiveEvent, ProinfoReadbackPlan, ProinfoReadbackResult, Result,
    SpftProgress, app_paths,
};
use std::fs;
use std::io::{self, Cursor, Read};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const SPFT_TOOL_ZIP: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/SPFlashToolV6_tool.zip"
));

const SPFT_TOOL_REVISION: &str = "spflashtoolv6_official_v6_2404_20260615_a";

const SPFT_OFFICIAL_ZIP_URL: &str =
    "https://spflashtools.com/wp-content/uploads/SP_Flash_Tool_V6.2404_Win.zip";

pub fn prepare_proinfo_readback(selected_path: &Path) -> Result<ProinfoReadbackPlan> {
    let (_firmware_root_dir, image_dir) = resolve_root_and_image_dir(selected_path)?;
    let root_dir = app_paths::runtime_root();

    let embedded_tool_dir = ensure_embedded_spflashtoolv6_tool()?;
    let spft_exe = embedded_tool_dir.join("SPFlashToolV6.exe");

    if !spft_exe.is_file() {
        return Err(LpmError::FileNotFound(spft_exe.display().to_string()));
    }

    let flash_xml = find_flash_xml(&image_dir).ok_or_else(|| {
        LpmError::FileNotFound(format!(
            "{} 또는 {}",
            image_dir.join("download_agent").join("flash.xml").display(),
            image_dir.join("flash.xml").display()
        ))
    })?;

    let backup_dir = app_paths::backup_root();
    let log_dir = app_paths::spft_log_dir();
    let config_xml = app_paths::readback_config_xml_path();
    let proinfo_out = app_paths::proinfo_backup_path();

    fs::create_dir_all(&backup_dir)?;
    fs::create_dir_all(&log_dir)?;

    if proinfo_out.exists() {
        fs::remove_file(&proinfo_out)?;
    }

    let xml = build_readback_config_xml(&flash_xml, &log_dir, &proinfo_out);
    fs::write(&config_xml, xml.as_bytes())?;

    if !config_xml.is_file() {
        return Err(LpmError::Spft(format!(
            "readback_config.xml 생성 실패: {}",
            config_xml.display()
        )));
    }

    Ok(ProinfoReadbackPlan {
        root_dir,
        image_dir,
        embedded_tool_used: true,
        embedded_tool_dir,
        spft_exe,
        flash_xml,
        backup_dir,
        log_dir,
        config_xml,
        proinfo_out,
        config_xml_size: xml.len(),
    })
}


pub fn execute_proinfo_readback_streaming<F>(
    selected_path: &Path,
    mut on_event: F,
) -> Result<ProinfoReadbackResult>
where
    F: FnMut(ProinfoLiveEvent),
{
    let plan = prepare_proinfo_readback(selected_path)?;

    on_event(ProinfoLiveEvent::Log(
        "SPFlashToolV6 readback 실행 시작".to_string(),
    ));

    let mut command = Command::new(&plan.spft_exe);
    command.arg("-i").arg(&plan.config_xml);
    command.current_dir(&plan.embedded_tool_dir);

    let output = run_spflashtoolv6_streaming(command, &mut on_event)?;

    let proinfo_exists = plan.proinfo_out.is_file();
    let proinfo_size = if proinfo_exists {
        fs::metadata(&plan.proinfo_out)?.len()
    } else {
        0
    };

    let result = ProinfoReadbackResult {
        plan,
        exit_code: output.exit_code,
        stdout_tail: output.stdout_tail,
        stderr_tail: output.stderr_tail,
        proinfo_exists,
        proinfo_size,
    };

    if !output.success {
        return Err(LpmError::Spft(format!(
            "SPFlashToolV6 readback 실패 / exit_code={:?} / stderr={}",
            result.exit_code, result.stderr_tail
        )));
    }

    if !result.proinfo_exists || result.proinfo_size == 0 {
        return Err(LpmError::Spft(format!(
            "SPFlashToolV6 실행은 완료됐지만 proinfo 백업 파일이 생성되지 않았습니다: {}",
            result.plan.proinfo_out.display()
        )));
    }

    Ok(result)
}

pub fn execute_firmware_download_streaming<F>(plan: &FlashPlan, mut on_event: F) -> Result<()>
where
    F: FnMut(ProinfoLiveEvent),
{
    let embedded_tool_dir = ensure_embedded_spflashtoolv6_tool()?;
    let spft_exe = embedded_tool_dir.join("SPFlashToolV6.exe");

    if !spft_exe.is_file() {
        return Err(LpmError::FileNotFound(spft_exe.display().to_string()));
    }

    if !plan.da_path.is_file() {
        return Err(LpmError::FileNotFound(plan.da_path.display().to_string()));
    }

    if !plan.work_flash_xml.is_file() {
        return Err(LpmError::FileNotFound(
            plan.work_flash_xml.display().to_string(),
        ));
    }

    on_event(ProinfoLiveEvent::Log(
        "SPFlashToolV6 download 실행 시작".to_string(),
    ));
    on_event(ProinfoLiveEvent::Log(format!(
        "SPFlashToolV6.exe: {}",
        spft_exe.display()
    )));
    on_event(ProinfoLiveEvent::Log(format!(
        "DA 파일: {}",
        plan.da_path.display()
    )));
    on_event(ProinfoLiveEvent::Log(format!(
        "flash.xml: {}",
        plan.work_flash_xml.display()
    )));

    let mut command = Command::new(&spft_exe);
    command.arg("-a").arg(&plan.da_path);
    command.arg("-f").arg(&plan.work_flash_xml);
    command.arg("-c").arg("download");
    command.current_dir(&embedded_tool_dir);

    let output = run_spflashtoolv6_streaming(command, &mut on_event)?;

    if !output.success {
        return Err(LpmError::Spft(format!(
            "SPFlashToolV6 download 실패 / exit_code={:?} / stderr={}",
            output.exit_code, output.stderr_tail
        )));
    }

    Ok(())
}

struct SpftStreamingOutput {
    success: bool,
    exit_code: Option<i32>,
    stdout_tail: String,
    stderr_tail: String,
}

#[derive(Debug, Clone, Copy)]
enum PipeKind {
    Stdout,
    Stderr,
}

struct PipeLine {
    kind: PipeKind,
    line: String,
}

fn run_spflashtoolv6_streaming<F>(
    mut command: Command,
    on_event: &mut F,
) -> Result<SpftStreamingOutput>
where
    F: FnMut(ProinfoLiveEvent),
{
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    #[cfg(windows)]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = command
        .spawn()
        .map_err(|err| LpmError::Spft(format!("SPFlashToolV6 실행 실패: {err}")))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let (line_tx, line_rx) = mpsc::channel::<PipeLine>();

    if let Some(stdout) = stdout {
        spawn_pipe_reader(stdout, PipeKind::Stdout, line_tx.clone());
    }

    if let Some(stderr) = stderr {
        spawn_pipe_reader(stderr, PipeKind::Stderr, line_tx.clone());
    }

    drop(line_tx);

    let mut stdout_tail = String::new();
    let mut stderr_tail = String::new();
    let mut last_progress: Option<(String, u8)> = None;

    loop {
        while let Ok(pipe_line) = line_rx.try_recv() {
            handle_spflashtoolv6_pipe_line(
                pipe_line,
                &mut stdout_tail,
                &mut stderr_tail,
                &mut last_progress,
                on_event,
            );
        }

        if let Some(status) = child
            .try_wait()
            .map_err(|err| LpmError::Spft(format!("SPFlashToolV6 상태 확인 실패: {err}")))?
        {
            let settle_deadline = Instant::now() + Duration::from_millis(800);

            while Instant::now() < settle_deadline {
                match line_rx.recv_timeout(Duration::from_millis(50)) {
                    Ok(pipe_line) => {
                        handle_spflashtoolv6_pipe_line(
                            pipe_line,
                            &mut stdout_tail,
                            &mut stderr_tail,
                            &mut last_progress,
                            on_event,
                        );
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }

            return Ok(SpftStreamingOutput {
                success: status.success(),
                exit_code: status.code(),
                stdout_tail,
                stderr_tail,
            });
        }

        match line_rx.recv_timeout(Duration::from_millis(100)) {
            Ok(pipe_line) => {
                handle_spflashtoolv6_pipe_line(
                    pipe_line,
                    &mut stdout_tail,
                    &mut stderr_tail,
                    &mut last_progress,
                    on_event,
                );
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {}
        }
    }
}

fn spawn_pipe_reader<R>(reader: R, kind: PipeKind, sender: mpsc::Sender<PipeLine>)
where
    R: Read + Send + 'static,
{
    let _ = thread::Builder::new()
        .name(match kind {
            PipeKind::Stdout => "lpmbox-spflashtoolv6-stdout".to_string(),
            PipeKind::Stderr => "lpmbox-spflashtoolv6-stderr".to_string(),
        })
        .spawn(move || {
            read_pipe_lines(reader, kind, sender);
        });
}

fn read_pipe_lines<R>(mut reader: R, kind: PipeKind, sender: mpsc::Sender<PipeLine>)
where
    R: Read,
{
    let mut buf = [0u8; 4096];
    let mut pending = Vec::<u8>::new();

    loop {
        match reader.read(&mut buf) {
            Ok(0) => {
                flush_pending_line(&mut pending, kind, &sender);
                break;
            }
            Ok(n) => {
                for byte in &buf[..n] {
                    if *byte == b'\n' || *byte == b'\r' {
                        flush_pending_line(&mut pending, kind, &sender);
                    } else {
                        pending.push(*byte);
                    }
                }
            }
            Err(_) => {
                flush_pending_line(&mut pending, kind, &sender);
                break;
            }
        }
    }
}

fn flush_pending_line(pending: &mut Vec<u8>, kind: PipeKind, sender: &mpsc::Sender<PipeLine>) {
    if pending.is_empty() {
        return;
    }

    let line = String::from_utf8_lossy(pending).trim().to_string();
    pending.clear();

    if line.is_empty() {
        return;
    }

    let _ = sender.send(PipeLine { kind, line });
}

fn handle_spflashtoolv6_pipe_line<F>(
    pipe_line: PipeLine,
    stdout_tail: &mut String,
    stderr_tail: &mut String,
    last_progress: &mut Option<(String, u8)>,
    on_event: &mut F,
) where
    F: FnMut(ProinfoLiveEvent),
{
    match pipe_line.kind {
        PipeKind::Stdout => append_tail(stdout_tail, &pipe_line.line, 4000),
        PipeKind::Stderr => append_tail(stderr_tail, &pipe_line.line, 4000),
    }

    if let Some(progress) = parse_spflashtoolv6_progress(&pipe_line.line) {
        let current = (progress.stage.clone(), progress.percent);

        if last_progress.as_ref() != Some(&current) {
            *last_progress = Some(current);
            on_event(ProinfoLiveEvent::Progress(progress));
        }

        return;
    }

    if let Some(message) = parse_spflashtoolv6_status_message(&pipe_line.line) {
        on_event(ProinfoLiveEvent::Log(message));
    }
}

fn parse_spflashtoolv6_progress(line: &str) -> Option<SpftProgress> {
    let percent = extract_percent(line)?;
    let lower = line.to_ascii_lowercase();

    let stage = if lower.contains("da data has been sent") {
        "DA 설정".to_string()
    } else if lower.contains("data readback to pc") {
        "proinfo 백업".to_string()
    } else if let Some(partition) = extract_spft_partition_name(line) {
        partition
    } else if lower.contains("image data has been sent") {
        "image".to_string()
    } else if lower.contains("download")
        || lower.contains("write")
        || lower.contains("format")
        || lower.contains("flash")
    {
        "플래싱".to_string()
    } else {
        "작업 진행".to_string()
    };

    Some(SpftProgress { stage, percent })
}

fn extract_spft_partition_name(line: &str) -> Option<String> {
    let start = line.find('[')?;
    let rest = &line[start + 1..];
    let end = rest.find(']')?;
    let raw = rest[..end].trim();

    let name = raw.split_whitespace().next()?.trim();

    if name.is_empty() {
        return None;
    }

    let count = raw.split_once('[').and_then(|(_, right)| {
        let value = right.trim().trim_end_matches(']').trim();

        if value.contains('/') {
            Some(value.to_string())
        } else {
            None
        }
    });

    if let Some(count) = count {
        Some(format!("{name} ({count})"))
    } else {
        Some(name.to_string())
    }
}

fn parse_spflashtoolv6_status_message(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();

    if lower.contains("connect brom succeeded") {
        return Some("BROM 연결 성공".to_string());
    }

    if lower.contains("enter da mode start") {
        return Some("DA 모드 진입 시작".to_string());
    }

    if lower.contains("enter da mode succeed") {
        return Some("DA 모드 진입 완료".to_string());
    }

    if lower.contains("get-sys-property command execute succeeded") {
        return Some("시스템 속성 확인 완료".to_string());
    }

    if lower.contains("read-partition command execute succeeded") {
        return Some("proinfo 백업 명령 완료".to_string());
    }

    if lower.contains("download command execute succeeded")
        || lower.contains("download command execute done")
    {
        return Some("플래싱 명령 완료".to_string());
    }

    if lower.contains("reboot command execute succeeded") {
        return Some("재시작 명령 완료".to_string());
    }

    if lower.contains("all command exec done") {
        return Some("SPFlashToolV6 작업 완료".to_string());
    }

    if lower.contains("warning") {
        return Some(format!("SPFlashToolV6 경고: {line}"));
    }

    if lower.contains("error") || lower.contains("fail") {
        return Some(format!("SPFlashToolV6 오류: {line}"));
    }

    None
}

fn extract_percent(line: &str) -> Option<u8> {
    let bytes = line.as_bytes();

    for index in 0..bytes.len() {
        if bytes[index] != b'%' {
            continue;
        }

        let mut start = index;

        while start > 0 && bytes[start - 1].is_ascii_digit() {
            start -= 1;
        }

        if start == index {
            continue;
        }

        let value = std::str::from_utf8(&bytes[start..index]).ok()?;
        let percent = value.parse::<u16>().ok()?;

        if percent <= 100 {
            return Some(percent as u8);
        }
    }

    None
}

fn append_tail(target: &mut String, line: &str, max_chars: usize) {
    target.push_str(line);
    target.push('\n');

    if target.chars().count() > max_chars {
        *target = tail_text(target, max_chars);
    }
}

fn ensure_embedded_spflashtoolv6_tool() -> Result<PathBuf> {
    app_paths::ensure_runtime_directories()?;

    let tool_dir = embedded_tool_dir();

    if tool_dir.exists() {
        cleanup_spft_wrapper_dirs(&tool_dir)?;
    }

    let marker = tool_dir.join(".lpmbox_spflashtoolv6_revision");
    let spft_exe = tool_dir.join("SPFlashToolV6.exe");

    if spft_exe.is_file()
        && fs::read_to_string(&marker)
            .map(|value| value.trim() == SPFT_TOOL_REVISION)
            .unwrap_or(false)
    {
        return Ok(tool_dir);
    }

    if tool_dir.exists() {
        fs::remove_dir_all(&tool_dir)?;
    }

    fs::create_dir_all(&tool_dir)?;

    let zip_result = download_official_spflashtoolv6_zip()
        .or_else(|_download_err| cached_official_spflashtoolv6_zip());

    match zip_result {
        Ok(zip_path) => {
            extract_spflashtoolv6_zip_file(&zip_path, &tool_dir)?;
        }

        Err(_download_err) => {
            extract_embedded_spflashtoolv6_zip(&tool_dir)?;
        }
    }

    cleanup_spft_wrapper_dirs(&tool_dir)?;

    fs::write(&marker, SPFT_TOOL_REVISION)?;

    validate_extracted_spflashtoolv6_tool(&tool_dir)?;

    Ok(tool_dir)
}

fn download_official_spflashtoolv6_zip() -> Result<PathBuf> {
    let zip_path = app_paths::spflashtoolv6_zip_path();

    if let Some(parent) = zip_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let temp_zip_path = zip_path.with_extension("zip.download");

    if temp_zip_path.exists() {
        fs::remove_file(&temp_zip_path)?;
    }

    let response = ureq::get(SPFT_OFFICIAL_ZIP_URL)
        .set("User-Agent", "Mozilla/5.0")
        .call()
        .map_err(|err| LpmError::Spft(format!("공식 SPFlashToolV6 다운로드 실패: {err}")))?;

    if !(200..300).contains(&response.status()) {
        return Err(LpmError::Spft(format!(
            "공식 SPFlashToolV6 다운로드 HTTP 오류: {}",
            response.status()
        )));
    }

    let mut reader = response.into_reader();
    let mut file = fs::File::create(&temp_zip_path)?;

    io::copy(&mut reader, &mut file)?;
    drop(file);

    let size = fs::metadata(&temp_zip_path)?.len();

    if size == 0 {
        let _ = fs::remove_file(&temp_zip_path);
        return Err(LpmError::Spft(format!(
            "공식 SPFlashToolV6 ZIP 파일 크기가 0입니다: {}",
            temp_zip_path.display()
        )));
    }

    if zip_path.exists() {
        fs::remove_file(&zip_path)?;
    }

    fs::rename(&temp_zip_path, &zip_path)?;

    Ok(zip_path)
}

fn cached_official_spflashtoolv6_zip() -> Result<PathBuf> {
    let zip_path = app_paths::spflashtoolv6_zip_path();

    if !zip_path.is_file() {
        return Err(LpmError::FileNotFound(zip_path.display().to_string()));
    }

    let size = fs::metadata(&zip_path)?.len();

    if size == 0 {
        return Err(LpmError::Spft(format!(
            "SPFlashToolV6 ZIP 파일 크기가 0입니다: {}",
            zip_path.display()
        )));
    }

    Ok(zip_path)
}

fn extract_spflashtoolv6_zip_file(zip_path: &Path, target_dir: &Path) -> Result<()> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file).map_err(zip_error)?;

    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(zip_error)?;

        let Some(enclosed_name) = file.enclosed_name().map(|path| path.to_path_buf()) else {
            continue;
        };

        let Some(relative_path) = normalize_spft_zip_entry_path(&enclosed_name) else {
            continue;
        };

        let out_path = target_dir.join(relative_path);

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

fn normalize_spft_zip_entry_path(path: &Path) -> Option<PathBuf> {
    let mut parts = Vec::<String>::new();

    for component in path.components() {
        let Component::Normal(value) = component else {
            return None;
        };

        let value = value.to_string_lossy().to_string();

        if value.eq_ignore_ascii_case("__MACOSX")
            || value.eq_ignore_ascii_case("BackupData")
            || value.starts_with('.')
        {
            return None;
        }

        parts.push(value);
    }

    if parts.is_empty() {
        return None;
    }

    if parts.len() == 1 && is_spft_wrapper_folder(&parts[0]) {
        return None;
    }

    let strip_root_folder =
        parts.len() > 1 && is_spft_wrapper_folder(&parts[0]) && !is_spft_root_entry(&parts[0]);

    let useful_parts = if strip_root_folder {
        &parts[1..]
    } else {
        &parts[..]
    };

    if useful_parts.is_empty() {
        return None;
    }

    let mut out = PathBuf::new();

    for part in useful_parts {
        out.push(part);
    }

    Some(out)
}

fn is_spft_root_entry(value: &str) -> bool {
    value.eq_ignore_ascii_case("SPFlashToolV6.exe")
        || value.eq_ignore_ascii_case("flash.dll")
        || value.eq_ignore_ascii_case("Qt5Core.dll")
        || value.eq_ignore_ascii_case("Qt5Gui.dll")
        || value.eq_ignore_ascii_case("Qt5Widgets.dll")
        || value.eq_ignore_ascii_case("plugins")
}

fn is_spft_wrapper_folder(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();

    lower.contains("sp_flash_tool")
        || lower.contains("spflashtool")
        || lower.contains("flash_tool")
        || lower.contains("v6.2404")
}

fn cleanup_spft_wrapper_dirs(tool_dir: &Path) -> Result<()> {
    let Ok(entries) = fs::read_dir(tool_dir) else {
        return Ok(());
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if is_spft_wrapper_folder(name) {
            fs::remove_dir_all(&path)?;
        }
    }

    Ok(())
}

fn embedded_tool_dir() -> PathBuf {
    app_paths::spflashtoolv6_dir()
}

fn extract_embedded_spflashtoolv6_zip(target_dir: &Path) -> Result<()> {
    let reader = Cursor::new(SPFT_TOOL_ZIP);
    let mut archive = zip::ZipArchive::new(reader).map_err(zip_error)?;

    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(zip_error)?;

        let Some(enclosed_name) = file.enclosed_name().map(|path| path.to_path_buf()) else {
            continue;
        };

        if should_skip_tool_entry(&enclosed_name) {
            continue;
        }

        let out_path = target_dir.join(&enclosed_name);

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

fn should_skip_tool_entry(path: &Path) -> bool {
    let Some(first_component) = path.components().next() else {
        return true;
    };

    match first_component {
        Component::Normal(name) => {
            let name = name.to_string_lossy();

            name.eq_ignore_ascii_case("BackupData")
                || name.eq_ignore_ascii_case("__MACOSX")
                || name.starts_with('.')
        }

        _ => true,
    }
}

fn validate_extracted_spflashtoolv6_tool(tool_dir: &Path) -> Result<()> {
    let required = [
        tool_dir.join("SPFlashToolV6.exe"),
        tool_dir.join("flash.dll"),
        tool_dir.join("Qt5Core.dll"),
        tool_dir.join("Qt5Gui.dll"),
        tool_dir.join("Qt5Widgets.dll"),
        tool_dir
            .join("plugins")
            .join("platforms")
            .join("qwindows.dll"),
    ];

    let missing = required
        .iter()
        .filter(|path| !path.is_file())
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();

    if !missing.is_empty() {
        return Err(LpmError::Spft(format!(
            "내장 SPFlashToolV6 추출 후 필수 파일이 없습니다: {}",
            missing.join(" / ")
        )));
    }

    Ok(())
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

    let child_image = selected_path.join("image");

    if child_image.is_dir() {
        return Ok((selected_path.to_path_buf(), child_image));
    }

    Err(LpmError::InvalidFirmwareFolder(format!(
        "image 폴더 또는 image 폴더를 포함한 root 폴더를 선택해야 합니다: {}",
        selected_path.display()
    )))
}

fn file_name_eq(path: &Path, expected: &str) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.eq_ignore_ascii_case(expected))
        .unwrap_or(false)
}

fn find_flash_xml(image_dir: &Path) -> Option<PathBuf> {
    let candidates = [
        image_dir.join("download_agent").join("flash.xml"),
        image_dir.join("flash.xml"),
    ];

    candidates.into_iter().find(|path| path.is_file())
}

fn build_readback_config_xml(flash_xml: &Path, log_dir: &Path, proinfo_out: &Path) -> String {
    let flash_xml = xml_escape(&flash_xml.display().to_string());
    let log_dir = xml_escape(&log_dir.display().to_string());
    let proinfo_out = xml_escape(&proinfo_out.display().to_string());

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<cflashtool-config version="1.0">
  <general>
    <connect-args version="1.0" com_port_name="" baud_rate="" timeout_ms="3600000" com_port_type="USB-PCIE" conn_stage="2nd DA">
      <flash-xml>{flash_xml}</flash-xml>
      <auth-file></auth-file>
      <cert-file></cert-file>
    </connect-args>
    <runtime-parameter version="1.0" da_log_level="INFO" system_os="WINDOWS" battery_status="0" log_channel="USB" checksum_level="1" initialize_dram="YES" usb_speed="0"/>
    <log-info version="1.0" log_path="{log_dir}" clean_hours="48"/>
  </general>
  <commands>
    <READ-PARTITION version="1.0" partition="proinfo">
      <target-file version="1.0" file_type="LOCAL_FILE" file_name="{proinfo_out}"/>
    </READ-PARTITION>
    <REBOOT version="1.0" action="COLD-RESET"/>
  </commands>
</cflashtool-config>"#
    )
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn zip_error(err: zip::result::ZipError) -> LpmError {
    LpmError::Spft(format!("내장 SPFlashToolV6 ZIP 처리 실패: {err}"))
}

fn tail_text(value: &str, max_chars: usize) -> String {
    let chars = value.chars().collect::<Vec<_>>();

    if chars.len() <= max_chars {
        return value.to_string();
    }

    chars[chars.len().saturating_sub(max_chars)..]
        .iter()
        .collect()
}
