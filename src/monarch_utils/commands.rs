use anyhow::{bail, Result};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::error;

use super::housekeeping::clear_all_cache;
use super::monarch_credentials::{delete_credentials, set_credentials};
use super::monarch_logger::get_log_dir;
use super::monarch_settings::{LauncherSettings, Settings};
use crate::monarch_utils::{monarch_fs, monarch_settings};

/*
*   Settings related commands
*
*   All settings related commands return the new settings as read by the backend to ensure both
*   frontend and backend agree on current settings.
*   Settings are wrapped in Result<> type to also tell frontend the success or failure of the command.
*   tauri::commands don't return the actual error message. Instead they write an easier error to understand for the user.
*/

pub fn open_logs() -> Result<(), String> {
    let log_path: PathBuf = get_log_dir();
    let res = if cfg!(target_os = "windows") {
        std::process::Command::new("explorer").arg(log_path).spawn()
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(log_path).spawn()
    } else {
        std::process::Command::new("xdg-open").arg(log_path).spawn()
    };

    match res {
        Ok(_) => Ok(()),
        Err(e) => {
            error!(
                "monarch_utils::commands::open_logs() Failed to open logs | Err: {}",
                e
            );
            Err(String::from("Failed to open logs in file manager."))
        }
    }
}

pub fn open_external_link(url: &str) {
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd.exe")
            .arg("/C")
            .arg(format!("start {}", url))
            .spawn();
    }
}

/// Returns settings read from settings.toml
pub fn get_settings() -> Result<Arc<RwLock<Settings>>> {
    monarch_settings::get_settings()
}

/// Write setting to settings.toml
/// Don't return custom error message as they instead return the state of settings according to
/// backend.
pub fn write_settings(settings: &Settings) -> Result<()> {
    monarch_settings::write_settings(settings)
}

/*
* User credentials related commands
*/

/// Set password in secure store
/// TODO: Better error handling if write_settings() fails.
pub fn set_password(
    launcher: &str,
    launcher_settings: &mut LauncherSettings,
    username: &str,
    password: &str,
) -> Result<()> {
    if !launcher_settings.username.is_empty() {
        error!("monarch_utils::commands::set_password() | Err: User already defined in settings.",);
        bail!("Monarch currently does not support more than one saved user!");
    }

    if let Err(e) = set_credentials(launcher, username, password) {
        error!(
            "monarch_utils::commands::set_password() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        bail!("Something went wrong setting new password!");
    }

    Ok(())
}

/// Delete password in secure store
/// TODO: Better error handling if write_settings() fails.
pub fn delete_password(launcher: &str, launcher_settings: &mut LauncherSettings) -> Result<()> {
    if let Err(e) = delete_credentials(&launcher, &launcher_settings.username) {
        error!(
            "monarch_utils::commands::delete_password() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        bail!("Something went wrong while deleting credentials!");
    }
    Ok(())
}

/// Set secret in secure store
pub fn set_secret(
    launcher: &str,
    launcher_settings: &mut LauncherSettings,
    secret: &str,
) -> Result<()> {
    if let Err(e) = set_credentials(
        &format!("{launcher}-secret"),
        &launcher_settings.username,
        secret,
    ) {
        error!(
            "monarch_utils::commands::set_secret() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        bail!("Something went wrong setting new secret!")
    }
    Ok(())
}

/// Delete secret in secure store
pub fn delete_secret(launcher: &str, launcher_settings: &mut LauncherSettings) -> Result<()> {
    if let Err(e) = delete_credentials(&format!("{launcher}-secret"), &launcher_settings.username) {
        error!(
            "monarch_utils::commands::delete_secret() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        bail!("Something went wrong while deleting secret!");
    }
    Ok(())
}

/*
* Misc commands
*/

/// Manually clear all images in the resources/cache directory
/// Don't return custom error message as they instead return the state of settings according to
/// backend.
pub fn clear_cached_images() {
    clear_all_cache();
}

pub fn get_cache_size() -> Result<u64, String> {
    let cache_dir = monarch_fs::get_resources_cache();
    match fs_extra::dir::get_size(&cache_dir) {
        Ok(size) => Ok(size as u64),
        Err(e) => {
            error!(
                "monarch_utils::commands::get_cache_size() Failed to read size of: {} | Err: {}",
                cache_dir.display(),
                e
            );
            Err(String::from("Failed to read size of cache directory."))
        }
    }
}
