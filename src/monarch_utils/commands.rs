use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use iced::advanced::graphics::text::cosmic_text::skrifa::setting;
use tracing::error;
use anyhow::{Result, bail};

use crate::monarch_utils::{monarch_fs, monarch_settings};
use super::housekeeping::clear_all_cache;
use super::monarch_credentials::{delete_credentials, set_credentials};
use super::monarch_logger::get_log_dir;
use super::monarch_settings::{
    LauncherSettings,
    Settings,
};

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

/// Returns settings read from settings.toml
pub fn get_settings() -> Result<Arc<RwLock<Settings>>> {
    monarch_settings::get_settings()
}

/// Write setting to settings.toml
/// Don't return custom error message as they instead return the state of settings according to
/// backend.
pub fn write_settings(settings: &Settings) -> Result<()> {
    monarch_settings::write_settings(&settings)
}

/*
* User credentials related commands
*/

/// Set password in secure store
/// TODO: Better error handling if write_settings() fails.
pub fn set_password(
    platform: String,
    username: String,
    password: String,
) -> Result<()> {
    let settings_lock: Arc<RwLock<Settings>>= get_settings().unwrap();

    let result = match settings_lock.write() {
        Ok(mut settings) => {
            let launcher_settings: &mut LauncherSettings = match platform.as_str() {
                    "steam" => &mut settings.steam,
                    "epic" => &mut settings.epic,
                    _ => {
                        error!(
                            "monarch_utils::commands::set_password() | Err: Invalid platform: {}",
                            platform
                        );
                        bail!("Trying to write user credentials for unknown platform.");
                    }
                };

                if !launcher_settings.username.is_empty() {
                    error!("monarch_utils::commands::set_password() | Err: User already defined in settings.",);
                    bail!("Monarch currently does not support more than one saved user!");
                }

                if let Err(e) = set_credentials(&platform, &username, &password) {
                    error!(
                        "monarch_utils::commands::set_password() -> {}",
                        e.chain().map(|e| e.to_string()).collect::<String>()
                    );
                    bail!("Something went wrong setting new password!");
                }

                launcher_settings.username = username;
                write_settings(&settings).unwrap();
                Ok(())
            }
        Err(e) => {
            error!("monarch_utils::commands::set_password() Failed to aquire write lock on Settings! | Err: {e}");
            bail!("Failed to lock on Settings!")
        }
    };

    result
}

/// Delete password in secure store
/// TODO: Better error handling if write_settings() fails.
pub fn delete_password(platform: String) -> Result<Settings, String> {
    let mut settings_lock: Arc<RwLock<Settings>>= get_settings().unwrap();

    let result = match settings_lock.write() {
        Ok(mut settings) => {
let launcher_settings: &mut LauncherSettings = match platform.as_str() {
        "steam" => &mut settings.steam,
        "epic" => &mut settings.epic,
        _ => {
            error!(
                "monarch_utils::commands::set_password() | Err: Invalid platform: {}",
                platform
            );
            return Err(String::from(
                "Trying to write user credentials for unknown platform.",
            ));
        }
    };

    if let Err(e) = delete_credentials(&platform, &launcher_settings.username) {
        error!(
            "monarch_utils::commands::delete_password() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from(
            "Something went wrong while deleting credentials!",
        ));
    }

    launcher_settings.username = String::new();
    set_settings_state(settings.clone());
    write_settings(settings.clone()).unwrap();
    Ok(settings)
        }
        Err(e) => {

        }
    };
    
    result
}

/// Set secret in secure store
pub fn set_secret(platform: String, secret: String) -> Result<Settings, String> {
    let mut settings: Settings = get_settings_state();
    let launcher_settings: &mut LauncherSettings = match platform.as_str() {
        "steam" => &mut settings.steam,
        "epic" => &mut settings.epic,
        _ => {
            error!(
                "monarch_utils::commands::set_password() | Err: Invalid platform: {}",
                platform
            );
            return Err(String::from(
                "Trying to write user credentials for unknown platform.",
            ));
        }
    };

    if let Err(e) = set_credentials(
        &format!("{platform}-secret"),
        &launcher_settings.username,
        &secret,
    ) {
        error!(
            "monarch_utils::commands::set_secret() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from("Something went wrong setting new secret!"));
    }
    launcher_settings.twofa = true;

    set_settings_state(settings.clone());
    write_settings(settings.clone()).unwrap();
    Ok(settings)
}

/// Delete secret in secure store
pub fn delete_secret(platform: String) -> Result<Settings, String> {
    let mut settings: Settings = get_settings_state();
    let launcher_settings: &mut LauncherSettings = match platform.as_str() {
        "steam" => &mut settings.steam,
        "epic" => &mut settings.epic,
        _ => {
            error!(
                "monarch_utils::commands::delete_secret() | Err: Invalid platform: {}",
                platform
            );
            return Err(String::from(
                "Trying to write user credentials for unknown platform.",
            ));
        }
    };

    if let Err(e) = delete_credentials(&format!("{platform}-secret"), &launcher_settings.username) {
        error!(
            "monarch_utils::commands::delete_password() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from("Something went wrong while deleting secret!"));
    }

    launcher_settings.twofa = false;

    set_settings_state(settings.clone());
    write_settings(settings.clone()).unwrap();
    Ok(settings)
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
