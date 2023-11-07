use std::{process::Command, path::PathBuf};
use toml::Table;

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
    read_settings()
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
    set_credentials(&platform, &username, &password)
}

#[tauri::command]
/// Delete password in secure store
pub fn delete_password(platform: String, username: String) -> Result<(), String> {
    delete_credentials(&platform, &username)
}