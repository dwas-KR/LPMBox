// SPDX-License-Identifier: GPL-3.0-only
//! Deletes only LPMBox-managed PC log data. Never deletes settings or ADB keys.
use std::{fs, io, path::Path};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LogCleanupReport {
    pub files_removed: usize,
    pub bytes_freed: u64,
}

#[cfg(windows)]
fn is_reparse(meta: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    meta.file_attributes() & 0x400 != 0
}
#[cfg(not(windows))]
fn is_reparse(meta: &fs::Metadata) -> bool {
    meta.file_type().is_symlink()
}

fn real_directory(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(meta) => {
            if meta.file_type().is_symlink() || is_reparse(&meta) {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "Refusing to follow a symbolic link or reparse point during LPMBox log cleanup",
                ));
            }
            if !meta.is_dir() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "LPMBox log path is not a directory",
                ));
            }
            Ok(true)
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err),
    }
}

fn delete_diagnostics_tree(path: &Path, report: &mut LogCleanupReport) -> io::Result<()> {
    if !real_directory(path)? {
        return Ok(());
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let child = entry.path();
        let meta = fs::symlink_metadata(&child)?;
        if meta.file_type().is_symlink() || is_reparse(&meta) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "LPMBox diagnostic log folder contains a redirected path",
            ));
        }
        if meta.is_dir() {
            delete_diagnostics_tree(&child, report)?;
        } else if meta.is_file() {
            fs::remove_file(&child)?;
            report.files_removed += 1;
            report.bytes_freed += meta.len();
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unknown diagnostic file type",
            ));
        }
    }
    fs::remove_dir(path)?;
    Ok(())
}

fn is_lpmbox_log(name: &str) -> bool {
    let lowered = name.to_ascii_lowercase();
    lowered.starts_with("lpmbox_") && (lowered.ends_with(".txt") || lowered.ends_with(".log"))
}

/// Clears the LPMBox runtime log files and the entire dedicated diagnostics folder.
/// Does not touch sibling files, APK cache, LPMBox configuration, or ADB keys.
pub fn remove_lpmbox_logs(log_dir: &Path, diagnostics: &Path) -> io::Result<LogCleanupReport> {
    if log_dir.file_name().is_none_or(|s| s != "logs")
        || diagnostics.file_name().is_none_or(|s| s != "diagnostics")
        || diagnostics
            .parent()
            .and_then(|p| p.file_name())
            .is_none_or(|s| s != "logs")
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unexpected LPMBox log cleanup paths",
        ));
    }

    let mut report = LogCleanupReport::default();
    if real_directory(log_dir)? {
        for entry in fs::read_dir(log_dir)? {
            let entry = entry?;
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if !is_lpmbox_log(&name) {
                continue;
            }
            let file = entry.path();
            let meta = fs::symlink_metadata(&file)?;
            if meta.file_type().is_symlink() || is_reparse(&meta) || !meta.is_file() {
                continue;
            }
            fs::remove_file(file)?;
            report.files_removed += 1;
            report.bytes_freed += meta.len();
        }
    }
    delete_diagnostics_tree(diagnostics, &mut report)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static ID: AtomicU64 = AtomicU64::new(0);
    fn sandbox() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "lpmbox_logcleanup_{}_{}",
            std::process::id(),
            ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }
    #[test]
    fn logs_deleted_but_other_data_retained() {
        let base = sandbox();
        let logs = base.join("runtime").join("logs");
        let diagnostics = base.join("app").join("logs").join("diagnostics");
        fs::create_dir_all(&logs).unwrap();
        fs::create_dir_all(diagnostics.join("old")).unwrap();
        fs::write(logs.join("LPMBox_2026-11-05.txt"), b"log").unwrap();
        fs::write(logs.join("unrelated.txt"), b"keep").unwrap();
        fs::write(diagnostics.join("old").join("adb.txt"), b"adb").unwrap();
        fs::write(base.join("language.txt"), b"ko").unwrap();
        let result = remove_lpmbox_logs(&logs, &diagnostics).unwrap();
        assert_eq!(result.files_removed, 2);
        assert_eq!(result.bytes_freed, 6);
        assert!(!diagnostics.exists());
        assert!(!logs.join("LPMBox_2026-11-05.txt").exists());
        assert!(logs.join("unrelated.txt").exists());
        assert!(base.join("language.txt").exists());
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn invalid_or_missing_directories_do_not_delete_arbitrary_folders() {
        let base = sandbox();
        let unsafe_dir = base.join("important");
        fs::create_dir_all(&unsafe_dir).unwrap();
        fs::write(unsafe_dir.join("LPMBox_2026.txt"), b"keep").unwrap();
        assert!(remove_lpmbox_logs(&unsafe_dir, &base.join("random")).is_err());
        assert!(unsafe_dir.join("LPMBox_2026.txt").exists());
        let reports = remove_lpmbox_logs(
            &base.join("no-logs").join("logs"),
            &base.join("no-app").join("logs").join("diagnostics"),
        )
        .unwrap();
        assert_eq!(reports, LogCleanupReport::default());
        fs::remove_dir_all(base).unwrap();
    }
}
