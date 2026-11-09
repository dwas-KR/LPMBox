// SPDX-License-Identifier: GPL-3.0-only
//! GBST v2.0.0-style cautious cache cleanup, adapted to LPMBox firmware tools.
//! Deletes only interrupted download fragments under the app-owned tools/download.
//! NEVER removes firmware ZIPs, tools, ADB keys, backup or log directories.
use std::{fs, io, path::Path};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CleanupReport { pub files_removed: usize, pub bytes_freed: u64 }

#[cfg(windows)]
fn redirected(meta: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    meta.file_attributes() & 0x400 != 0
}
#[cfg(not(windows))]
fn redirected(meta: &fs::Metadata) -> bool { meta.file_type().is_symlink() }

pub fn clean_transient_downloads(root: &Path) -> io::Result<CleanupReport> {
    if root.file_name().is_none_or(|n| n != "download") ||
        root.parent().and_then(|p| p.file_name()).is_none_or(|n| n != "tools") {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,"Not LPMBox tools/download"));
    }
    let info = match fs::symlink_metadata(root) {
        Ok(info) => info,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return Ok(CleanupReport::default());
        },
        Err(err) => return Err(err),
    };
    if !info.is_dir() || redirected(&info) {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied,"Refusing redirected cache root"));
    }
    let mut result=CleanupReport::default();
    for item in fs::read_dir(root)? {
        let entry=item?;
        let path=entry.path();
        let info=fs::symlink_metadata(&path)?;
        if !info.is_file() || redirected(&info) { continue; }
        let name=entry.file_name().to_string_lossy().to_ascii_lowercase();
        if ![".part",".tmp",".download"].iter().any(|suffix|name.ends_with(suffix)) {
            continue;
        }
        fs::remove_file(&path)?;
        result.files_removed+=1;
        result.bytes_freed+=info.len();
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_transient_files_are_removed() {
        let base=std::env::temp_dir().join(format!("lpmbox_cache_test_{}",std::process::id()));
        let root=base.join("tools/download");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("pending.part"), b"abc").unwrap();
        fs::write(root.join("firmware.zip"), b"safe").unwrap();
        fs::write(root.join("SPFlashToolV6.zip"), b"keep").unwrap();
        let report=clean_transient_downloads(&root).unwrap();
        assert_eq!(report.files_removed,1);
        assert_eq!(report.bytes_freed,3);
        assert!(root.join("firmware.zip").exists());
        assert!(root.join("SPFlashToolV6.zip").exists());
        fs::remove_dir_all(base).unwrap();
    }
    #[test]
    fn refuses_unrelated_folder() {
        let d=std::env::temp_dir().join("not_the_tools_folder");
        assert!(clean_transient_downloads(&d).is_err());
    }
}
