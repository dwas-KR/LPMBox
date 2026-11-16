// SPDX-License-Identifier: GPL-3.0-only
//! Atomic, crash-resistant persistence for the user's theme and language settings.
//! Independently adapted from LTBox 3.3.5's temporary-file and replace pattern.
//! No device operations, firmware files, or installation steps are changed.
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

/// Atomically replace an existing settings file without deleting the old
/// version first. Both files live in the same directory / filesystem.
/// A failed write or rename preserves the previous on-disk settings.
pub fn write_atomically(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;

    let filename = path.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "settings file name is empty")
    })?;
    let mut candidate = None;
    for _ in 0..64 {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let temp = parent.join(format!(
            ".{}.lpmbox-pending-{}-{}",
            filename.to_string_lossy(),
            std::process::id(),
            id
        ));
        match OpenOptions::new().write(true).create_new(true).open(&temp) {
            Ok(file) => {
                candidate = Some((temp, file));
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    let (temp, mut file) = candidate.ok_or_else(|| {
        io::Error::new(io::ErrorKind::AlreadyExists, "cannot allocate temporary settings file")
    })?;
    let _remove_on_failure = TempCleanup(temp.clone());
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    atomic_replace(&temp, path)
}

struct TempCleanup(PathBuf);
impl Drop for TempCleanup {
    fn drop(&mut self) {
        // No-op after a successful rename. Best effort on failure.
        let _ = fs::remove_file(&self.0);
    }
}

/// The Windows MoveFileEx replace-existing operation avoids the old
/// delete-then-rename pattern that could erase preferences after a crash.
#[cfg(windows)]
fn atomic_replace(source: &Path, destination: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn MoveFileExW(from: *const u16, to: *const u16, flags: u32) -> i32;
    }
    const MOVEFILE_REPLACE_EXISTING: u32 = 0x0000_0001;
    const MOVEFILE_WRITE_THROUGH: u32 = 0x0000_0008;
    let from: Vec<u16> = source.as_os_str().encode_wide().chain([0]).collect();
    let to: Vec<u16> = destination.as_os_str().encode_wide().chain([0]).collect();
    let status = unsafe {
        MoveFileExW(from.as_ptr(), to.as_ptr(), MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH)
    };
    if status == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
#[cfg(not(windows))]
fn atomic_replace(source: &Path, destination: &Path) -> io::Result<()> {
    fs::rename(source, destination)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_dir(test: &str) -> PathBuf {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let base = std::env::temp_dir()
            .join(format!("lpmbox-config-test-{}-{test}-{nonce}", std::process::id()));
        fs::create_dir_all(&base).unwrap();
        base
    }

    #[test]
    fn replaces_settings_without_an_intermediate_missing_file() {
        let root = test_dir("replace");
        let p = root.join("profile").join("appearance.json");
        write_atomically(&p, br#"{"theme":"light"}"#).unwrap();
        assert_eq!(fs::read(&p).unwrap(), br#"{"theme":"light"}"#);
        write_atomically(&p, br#"{"theme":"dark"}"#).unwrap();
        assert_eq!(fs::read(&p).unwrap(), br#"{"theme":"dark"}"#);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_replacement_never_removes_existing_preferences() {
        let root = test_dir("failure");
        let stable = root.join("language.txt");
        write_atomically(&stable, b"ko").unwrap();
        let target_is_directory = root.join("existing-directory");
        fs::create_dir(&target_is_directory).unwrap();
        assert!(write_atomically(&target_is_directory, b"en").is_err());
        assert_eq!(fs::read(&stable).unwrap(), b"ko");
        assert!(target_is_directory.is_dir());
        // Failed attempts leave no .lpmbox-pending-* files behind.
        assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
        fs::remove_dir_all(root).unwrap();
    }
}
