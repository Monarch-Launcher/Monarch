use std::{process::Command, path::PathBuf};
use toml::{map::Map, Value};

use super::monarch_logger::get_log_dir;
use super::monarch_settings::{read_settings, write_settings};

#[cfg(target_os = "windows")]
#[tauri::command]
/// Opens log folder on windows
pub async fn open_logs() {
   let path: PathBuf = get_log_dir();
   Command::new("PowerShell")
           .arg("start")
           .arg(path)
           .spawn()
           .unwrap();
}

#[cfg(target_os = "linux")]
#[tauri::command]
/// Opens log file on linux
pub async fn open_logs() {
   let path: PathBuf = get_log_dir();
   Command::new("xdg-open")
           .arg(path)
           .spawn()
           .unwrap();
}

#[tauri::command]
/// Returns settings read from settings.toml
pub fn get_settings() -> Result<Map<String, Value>, String> {
    read_settings()
}

#[tauri::command]
/// Write setting to settings.toml
pub fn set_setting(){

}