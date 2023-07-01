use std::{process::Command, path::PathBuf};

use super::monarch_logger::get_log_dir;
use super::monarch_settings::{read_settings, write_settings, MonarchSettings};

#[cfg(target_os = "windows")]
#[tauri::command]
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
pub async fn open_logs() {
   let path: PathBuf = get_log_dir();
   Command::new("xdg-open")
           .arg(path)
           .spawn()
           .unwrap();
}

#[tauri::command]
pub fn get_settings() -> Result<MonarchSettings, String> {
    read_settings()
}

#[tauri::command]
pub fn set_setting(){

}