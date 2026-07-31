use std::path::{Path, PathBuf};

pub fn runtime_root() -> PathBuf {
    if let Ok(current_dir) = std::env::current_dir() {
        if current_dir.join("Cargo.toml").is_file() && current_dir.join("crates").is_dir() {
            return current_dir;
        }
    }

    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(std::env::temp_dir)
}

fn local_data_root() -> PathBuf {
    dirs::data_local_dir().unwrap_or_else(runtime_root)
}

pub fn config_root() -> PathBuf {
    local_data_root().join("dwas_kr")
}

pub fn lpmbox_data_root() -> PathBuf {
    config_root().join("LPMBox")
}

pub fn legacy_config_root() -> PathBuf {
    local_data_root().join("LPMBox")
}

pub fn language_config_path() -> PathBuf {
    config_root().join("language.txt")
}

pub fn stable_adb_key_dir() -> PathBuf {
    config_root().join("adb")
}

pub fn stable_adb_key_path() -> PathBuf {
    stable_adb_key_dir().join("adbkey")
}

pub fn embedded_tools_root() -> PathBuf {
    lpmbox_data_root().join("embedded_tools")
}

pub fn runtime_adb_key_path() -> PathBuf {
    runtime_root().join("adb").join("adbkey")
}

pub fn ensure_runtime_directories() -> std::io::Result<()> {
    migrate_legacy_user_data()?;
    std::fs::create_dir_all(tool_dir())?;
    std::fs::create_dir_all(tool_download_dir())?;
    std::fs::create_dir_all(backup_root())?;
    std::fs::create_dir_all(log_dir())?;
    std::fs::create_dir_all(stable_adb_key_dir())?;
    std::fs::create_dir_all(lpmbox_data_root())?;
    Ok(())
}

pub fn migrate_legacy_user_data() -> std::io::Result<()> {
    let legacy_root = legacy_config_root();

    if !legacy_root.exists() {
        return Ok(());
    }

    move_file_if_present(
        &legacy_root.join("adb").join("adbkey"),
        &stable_adb_key_path(),
    )?;
    move_file_if_present(
        &legacy_root.join("adb").join("adbkey.pub"),
        &stable_adb_key_path().with_extension("pub"),
    )?;
    move_file_if_present(&legacy_root.join("language.txt"), &language_config_path())?;
    move_directory_contents_if_present(
        &legacy_root.join("embedded_tools"),
        &embedded_tools_root(),
    )?;
    remove_empty_directory(&legacy_root.join("adb"));
    remove_empty_directory(&legacy_root);
    Ok(())
}

fn move_file_if_present(source: &Path, destination: &Path) -> std::io::Result<()> {
    if !source.is_file() {
        return Ok(());
    }

    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if destination.exists() {
        std::fs::remove_file(source)?;
        return Ok(());
    }

    match std::fs::rename(source, destination) {
        Ok(()) => Ok(()),
        Err(_) => {
            std::fs::copy(source, destination)?;
            std::fs::remove_file(source)
        }
    }
}

fn move_directory_contents_if_present(source: &Path, destination: &Path) -> std::io::Result<()> {
    if !source.is_dir() {
        return Ok(());
    }

    if !destination.exists() {
        if std::fs::rename(source, destination).is_ok() {
            return Ok(());
        }
    }

    std::fs::create_dir_all(destination)?;

    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        if source_path.is_dir() {
            move_directory_contents_if_present(&source_path, &destination_path)?;
        } else {
            move_file_if_present(&source_path, &destination_path)?;
        }
    }

    remove_empty_directory(source);
    Ok(())
}

fn remove_empty_directory(path: &Path) {
    if path.is_dir() {
        let _ = std::fs::remove_dir(path);
    }
}

pub fn tool_dir() -> PathBuf {
    runtime_root().join("tools")
}

pub fn tool_download_dir() -> PathBuf {
    tool_dir().join("download")
}

pub fn spflashtoolv6_dir() -> PathBuf {
    embedded_tools_root()
}

pub fn spflashtoolv6_zip_path() -> PathBuf {
    tool_download_dir().join("SP_Flash_Tool_V6.2404_Win.zip")
}

pub fn mtk_driver_dir() -> PathBuf {
    tool_dir().join("MTK-Driver")
}

pub fn mtk_driver_zip_path() -> PathBuf {
    tool_download_dir().join("MTK-Driver-v5.1632.zip")
}

pub fn vc_redist_x86_path() -> PathBuf {
    tool_download_dir().join("VC_redist.x86.exe")
}

pub fn vc_redist_x64_path() -> PathBuf {
    tool_download_dir().join("VC_redist.x64.exe")
}

pub fn log_dir() -> PathBuf {
    runtime_root().join("logs")
}



pub fn backup_root() -> PathBuf {
    runtime_root().join("backup")
}

pub fn spft_log_dir() -> PathBuf {
    backup_root().join("spft_log")
}

pub fn proinfo_backup_path() -> PathBuf {
    backup_root().join("proinfo")
}

pub fn readback_config_xml_path() -> PathBuf {
    runtime_root().join("readback_config.xml")
}

pub fn adb_key_path() -> PathBuf {
    let _ = migrate_legacy_user_data();
    let stable_path = stable_adb_key_path();

    if stable_path.is_file() {
        return stable_path;
    }

    let legacy_path = runtime_adb_key_path();

    if legacy_path.is_file() {
        if let Some(parent) = stable_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if std::fs::rename(&legacy_path, &stable_path).is_ok()
            || (std::fs::copy(&legacy_path, &stable_path).is_ok()
                && std::fs::remove_file(&legacy_path).is_ok())
        {
            let legacy_public_key = legacy_path.with_extension("pub");
            let stable_public_key = stable_path.with_extension("pub");

            if legacy_public_key.is_file() {
                let _ = move_file_if_present(&legacy_public_key, &stable_public_key);
            }

            return stable_path;
        }

        return legacy_path;
    }

    stable_path
}
