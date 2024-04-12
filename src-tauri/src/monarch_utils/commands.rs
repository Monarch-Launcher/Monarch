use core::result::Result;
use log::error;
use std::{path::PathBuf, process::Command};
use toml::Table; // Use normal result instead of anyhow when sending to Frontend. Possibly replace later with anyhow that impls correct traits.

use super::housekeeping::clear_all_cache;
use super::monarch_credentials::{delete_credentials, set_credentials};
use super::monarch_logger::get_log_dir;
use super::monarch_settings::{read_settings, set_default_settings, write_settings};

#[cfg(target_os = "windows")]
#[tauri::command]
/// Use OS default option to open log directory
pub async fn open_logs() {
    let path: PathBuf = get_log_dir();
    Command::new("PowerShell")
        .arg("start")
        .arg(path)
        .spawn()
        .unwrap();
}

#[cfg(target_os = "macos")]
#[tauri::command]
/// Use OS default option to open log directory
pub async fn open_logs() {
    let path: PathBuf = get_log_dir();
    Command::new("open").arg(path).spawn().unwrap();
}

#[cfg(target_os = "linux")]
#[tauri::command]
/// Use OS default option to open log directory
pub async fn open_logs() {
    let path: PathBuf = get_log_dir();
    Command::new("xdg-open").arg(path).spawn().unwrap();
}

/*
    All settings related commands return the new settings as read by the backend to ensure both
    frontend and backend agree on current settings.
    Settings are wrapped in Result<> type to also tell frontend the success or failure of the command.
    tauri::commands don't return the actual error message. Instead they write an easier error to understand for the user.
*/

#[tauri::command]
/// Returns settings read from settings.toml
pub fn get_settings() -> Result<Table, String> {
    match read_settings() {
        Ok(result) => Ok(result),
        Err(e) => {
            error!("monarch_utils::commands::get_settings() -> {e}");
            Err("Something went wrong while reading settings!".to_string())
        }
    }
}

#[tauri::command]
/// Write setting to settings.toml
/// Don't return custom error message as they instead return the state of settings according to
/// backend.
pub fn set_settings(settings: Table) -> Result<Table, Table> {
    let res: Result<Table, Table> = write_settings(settings);

    if let Err(e) = &res {
        error!("monarch_utils::commands::set_settings() -> {e}");
    }

    res
}

#[tauri::command]
/// Write default settings to settings.toml
/// Don't return custom error message as they instead return the state of settings according to
/// backend.
pub fn revert_settings() -> Result<Table, Table> {
    let res: Result<Table, Table> = set_default_settings();

    if let Err(e) = &res {
        error!("monarch_utils::commands::revert_settings() -> {e}");
    }

    res
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
pub fn set_password(platform: String, username: String, password: String) -> Result<(), String> {
    if let Err(e) = set_credentials(&platform, &username, &password) {
        error!("monarch_utils::commands::set_password() -> {e}");
        return Err(String::from("Something went wrong setting new password!"));
    }
    Ok(())
}

#[tauri::command]
/// Delete password in secure store
pub fn delete_password(platform: String, username: String) -> Result<(), String> {
    if let Err(e) = delete_credentials(&platform, &username) {
        error!("monarch_utils::commands::delete_password() -> {e}");
        return Err("Something went wrong while deleting credentials!".to_string());
    }
    Ok(())
}
