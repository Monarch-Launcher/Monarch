use std::{process::Command, path::PathBuf};
use toml::Table;

use super::monarch_logger::get_log_dir;
use super::monarch_settings::{read_settings, write_settings};
use super::housekeeping::clear_all_cache;

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

#[tauri::command]
/// Returns settings read from settings.toml
pub fn get_settings() -> Result<Table, String> {
    read_settings()
}

#[tauri::command]
/// Write setting to settings.toml
pub fn set_setting(header: &str, key: &str, value: &str) -> Result<(), String> {
    write_settings(header, key, value)
}

#[tauri::command]
/// Manually clear all images in the resources/cache directory
pub fn clear_cached_images() {
    clear_all_cache();
}