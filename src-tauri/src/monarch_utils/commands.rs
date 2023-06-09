use super::monarch_logger::get_log_dir;
use std::{process::Command, path::PathBuf};

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