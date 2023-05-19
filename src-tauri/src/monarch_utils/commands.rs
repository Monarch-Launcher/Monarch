use super::monarch_logger::get_log_dir;
use std::process::Command;

#[tauri::command]
pub async fn get_logs() {
   let path: String = get_log_dir();
   Command::new("PowerShell")
           .arg("start")
           .arg(path)
           .spawn();
}