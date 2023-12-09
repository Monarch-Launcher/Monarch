use std::{process::Command, path::PathBuf};
use log::error;
use toml::Table;
use core::result::Result; // Use normal result instead of anyhow when sending to Frontend. Possibly replace later with anyhow that impls correct traits.

use super::monarch_logger::get_log_dir;
use super::monarch_settings::{read_settings, write_settings, set_default_settings};
use super::housekeeping::clear_all_cache;
use super::monarch_credentials::{set_credentials, delete_credentials};

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
   Command::new("open")
           .arg(path)
           .spawn()
           .unwrap();
}

#[cfg(target_os = "linux")]
#[tauri::command]
/// Use OS default option to open log directory
pub async fn open_logs() {
   let path: PathBuf = get_log_dir();
   Command::new("xdg-open")
           .arg(path)
           .spawn()
           .unwrap();
}

/*
    All settings related commands return the new settings as read by the backend to ensure both
    frontend and backend agree on current settings.
    Settings are wrapped in Result<> type to also tell frontend the success or failure of the command.
*/

#[tauri::command]
/// Returns settings read from settings.toml
pub fn get_settings() -> Result<Table, String> {
    match read_settings() {
        Ok(result) => Ok(result),
        Err(e) => {
            error!("{e}");
            Err(String::from("Failed to read settings!"))
        }
    }
}

#[tauri::command]
/// Write setting to settings.toml
pub fn set_settings(settings: Table) -> Result<Table, Table> {
    write_settings(settings)
}

#[tauri::command]
/// Write default settings to settings.toml
pub fn revert_settings() -> Result<Table, Table> {
    set_default_settings()
}

#[tauri::command]
/// Manually clear all images in the resources/cache directory
pub fn clear_cached_images() {
    clear_all_cache();
}

#[tauri::command]
/// Set password in secure store
pub fn set_password(platform: String, username: String, password: String) -> Result<(), String> {
    if let Err(e) = set_credentials(&platform, &username, &password) {
        error!("{e}");
        return Err(String::from("Failed to set password!"))
    }
    Ok(())
}

#[tauri::command]
/// Delete password in secure store
pub fn delete_password(platform: String, username: String) -> Result<(), String> {
    if let Err(e) = delete_credentials(&platform, &username) {
        error!("{e}");
        return Err(String::from("Failed to delete password!"))
    }
    Ok(())
}