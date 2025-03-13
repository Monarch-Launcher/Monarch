use core::result::Result;
use log::error;
use std::{path::PathBuf, process::Command};

use super::housekeeping::clear_all_cache;
use super::monarch_credentials::{delete_credentials, set_credentials};
use super::monarch_logger::get_log_dir;
use super::monarch_settings::{
    get_settings_state, read_settings, set_default_settings, set_settings_state, write_settings,
    LauncherSettings, Settings,
};

#[cfg(target_os = "windows")]
#[tauri::command]
/// Use OS default option to open log directory
pub async fn open_logs() -> Result<(), String> {
    let path: PathBuf = get_log_dir();
    if let Err(e) = Command::new("PowerShell")
        .arg("start")
        .arg(path)
        .spawn()
        .unwrap()
    {
        error!("monarch_utils::commands::open_logs() Error opening logs! | Err: {e}");
        return Err(String::from("Error opening logs!"));
    }
    Ok(())
}

#[cfg(target_os = "macos")]
#[tauri::command]
/// Use OS default option to open log directory
pub async fn open_logs() -> Result<(), String> {
    let path: PathBuf = get_log_dir();
    if let Err(e) = Command::new("open").arg(path).spawn() {
        error!("monarch_utils::commands::open_logs() Error opening logs! | Err: {e}");
        return Err(String::from("Error opening logs!"));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
#[tauri::command]
/// Use OS default option to open log directory
pub async fn open_logs() -> Result<(), String> {
    let path: PathBuf = get_log_dir();
    if let Err(e) = Command::new("xdg-open").arg(path).spawn() {
        error!("monarch_utils::commands::open_logs() Error opening logs! | Err: {e}");
        return Err(String::from("Error opening logs!"));
    }
    Ok(())
}

/*
    All settings related commands return the new settings as read by the backend to ensure both
    frontend and backend agree on current settings.
    Settings are wrapped in Result<> type to also tell frontend the success or failure of the command.
    tauri::commands don't return the actual error message. Instead they write an easier error to understand for the user.
*/

#[tauri::command]
/// Returns settings read from settings.toml
pub fn get_settings() -> Settings {
    get_settings_state()
}

#[tauri::command]
/// Write setting to settings.toml
/// Don't return custom error message as they instead return the state of settings according to
/// backend.
pub fn set_settings(settings: Settings) -> Result<Settings, String> {
    match write_settings(settings) {
        Ok(ret_settings) => {
            set_settings_state(ret_settings.clone());
            Ok(ret_settings)
        }
        Err(e) => {
            error!(
                "monarch_utils::commands::set_settings() -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            Err(String::from("Failed to write new settings!"))
        }
    }
}

#[tauri::command]
/// Write default settings to settings.toml
/// Don't return custom error message as they instead return the state of settings according to
/// backend.
pub fn revert_settings() -> Result<Settings, String> {
    match set_default_settings() {
        Ok(ret_settings) => {
            set_settings_state(ret_settings.clone());
            Ok(ret_settings)
        }
        Err(e) => {
            error!(
                "monarch_utils::commands::revert_settings() -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            Err(String::from("Failed to reset to default settings!"))
        }
    }
}

#[tauri::command]
/// Manually clear all images in the resources/cache directory
/// Don't return custom error message as they instead return the state of settings according to
/// backend.
pub fn clear_cached_images() {
    clear_all_cache();
}

#[tauri::command]
/// Set password in secure store
/// TODO: Better error handling if write_settings() fails.
pub fn set_password(platform: String, username: String, password: String) -> Result<(), String> {
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

    if !launcher_settings.username.is_empty() {
        error!("monarch_utils::commands::set_password() | Err: User already defined in settings.",);
        return Err(String::from(
            "Monarch currently does not support more than one saved user!",
        ));
    }

    if let Err(e) = set_credentials(&platform, &username, &password) {
        error!(
            "monarch_utils::commands::set_password() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from("Something went wrong setting new password!"));
    }

    launcher_settings.username = username;
    set_settings_state(settings.clone());
    write_settings(settings).unwrap();
    Ok(())
}

#[tauri::command]
/// Delete password in secure store
/// TODO: Better error handling if write_settings() fails.
pub fn delete_password(platform: String, username: String) -> Result<(), String> {
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

    if let Err(e) = delete_credentials(&platform, &username) {
        error!(
            "monarch_utils::commands::delete_password() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from(
            "Something went wrong while deleting credentials!",
        ));
    }

    launcher_settings.username = username;
    set_settings_state(settings.clone());
    write_settings(settings).unwrap();
    Ok(())
}
