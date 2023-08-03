use std::{process::Command, path::PathBuf};

use super::monarch_logger::get_log_dir;
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

#[cfg(not(target_os = "windows"))]
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
/// Manually clear all images in the resources/cache directory
pub fn clear_cached_images() {
    clear_all_cache();
}