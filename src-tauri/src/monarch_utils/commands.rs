use core::result::Result;
use log::{error, info};
use std::env;
use std::{path::PathBuf, process::Command};
use tauri::{AppHandle, Manager};
use toml::Table; // Use normal result instead of anyhow when sending to Frontend. Possibly replace later with anyhow that impls correct traits.

use super::housekeeping::clear_all_cache;
use super::monarch_credentials::{delete_credentials, set_credentials};
use super::monarch_logger::get_log_dir;
use super::monarch_settings::{read_settings, set_default_settings, write_settings};
use super::monarch_windows::MiniWindow;

#[cfg(target_os = "windows")]
#[tauri::command]
/// Use OS default option to open log directory
pub async fn open_logs() -> Result<(), String> {
    let path: PathBuf = get_log_dir();
    if let Err(e) = Command::new("PowerShell").arg("start").arg(path).spawn() {
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
*   Settings related commands
*
*   All settings related commands return the new settings as read by the backend to ensure both
*   frontend and backend agree on current settings.
*   Settings are wrapped in Result<> type to also tell frontend the success or failure of the command.
*   tauri::commands don't return the actual error message. Instead they write an easier error to understand for the user.
*/

#[tauri::command]
/// Returns settings read from settings.toml
pub fn get_settings() -> Result<Table, String> {
    match read_settings() {
        Ok(result) => Ok(result),
        Err(e) => {
            error!(
                "monarch_utils::commands::get_settings() -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            Err(String::from("Something went wrong while reading settings!"))
        }
    }
}

#[tauri::command]
/// Write setting to settings.toml
/// Don't return custom error message as they instead return the state of settings according to
/// backend.
pub fn set_settings(settings: Table) -> Result<Table, Table> {
    let res: Result<Table, Table> = write_settings(settings);

    if res.is_err() {
        error!("monarch_utils::commands::set_settings() -> monarch_settings::write_settings() returned error!");
    }

    res
}

#[tauri::command]
/// Write default settings to settings.toml
/// Don't return custom error message as they instead return the state of settings according to
/// backend.
pub fn revert_settings() -> Result<Table, Table> {
    let res: Result<Table, Table> = set_default_settings();

    if res.is_err() {
        error!("monarch_utils::commands::revert_settings() -> monarch_settings::set_default_settings() returned error!");
    }

    res
}

/*
* User credentials related commands
*/

#[tauri::command]
/// Set password in secure store
pub fn set_password(platform: String, username: String, password: String) -> Result<(), String> {
    if let Err(e) = set_credentials(&platform, &username, &password) {
        error!(
            "monarch_utils::commands::set_password() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from("Something went wrong setting new password!"));
    }
    Ok(())
}

#[tauri::command]
/// Delete password in secure store
pub fn delete_password(platform: String, username: String) -> Result<(), String> {
    if let Err(e) = delete_credentials(&platform, &username) {
        error!(
            "monarch_utils::commands::delete_password() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from(
            "Something went wrong while deleting credentials!",
        ));
    }
    Ok(())
}

/*
* Quicklaunch related commands
*/
#[tauri::command]
/// This function returns if quicklaunch is allowed to run or not.
/// For quicklaunch to run 2 conditions must be met.
/// 1: Quicklaunch is enabled in settings
/// 2: Monarch is not being run under Wayland, which currently
/// lacks support for global shortcuts and makes Monarch hang.
pub fn quicklaunch_is_enabled() -> bool {
    // First check if quicklaunch is disabled in settings
    // TODO: Check if quicklaunch is enabled in settings

    // Then check if Monarch is being run under Wayland
    if cfg!(target_os = "linux") {
        return env::var("WAYLAND_DISPLAY").is_err(); // If WAYLAND_DISPLAY is set at all, assume Wayland is used
    }
    true
}

#[tauri::command]
/// Builds a new quicklaunch window.
/// Starts as hidden unitl user presses quicklaunch shortcut.
pub async fn init_quicklaunch(handle: AppHandle) -> Result<(), String> {
    let window = MiniWindow::new(
        "quicklaunch",
        "/src/quicklaunch/quicklaunch.html",
        854.0,
        480.0,
    );
    if let Err(e) = window.build_window(&handle).await {
        error!(
            "monarch_utils::commands::init_quicklaunch() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from("Failed to build quicklaunch window!"));
    }
    if let Err(e) = window.hide_window(&handle) {
        error!(
            "monarch_utils::commands::init_quicklaunch() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from("Failed to hide quicklaunch window!"));
    }
    info!("Finished initializing quicklaunch!");
    Ok(())
}

#[tauri::command]
/// Show quicklaunch and focus quicklaunch window
pub fn show_quicklaunch(handle: AppHandle) -> Result<(), String> {
    if let Some(window) = handle.get_window("quicklaunch") {
        if let Err(e) = window.show() {
            error!("monarch_utils::commands::show_quicklaunch() Failed to show quicklaunch! | Err: {e}");
            return Err(String::from("Failed to hide quicklaunch window!"));
        }
        if let Err(e) = window.set_focus() {
            error!("monarch_utils::commands::show_quicklaunch() Failed to set focus to quicklaunch! | Err: {e}");
            return Err(String::from("Failed to show quicklaunch!"));
        }
        return Ok(());
    }
    error!("monarch_utils::commands::show_quicklaunch() Err: handle.get_window() returned None!");
    Err(String::from("Failed to get quicklaunch window!"))
}

#[tauri::command]
/// Hide quicklaunch window
pub fn hide_quicklaunch(handle: AppHandle) -> Result<(), String> {
    if let Some(window) = handle.get_window("quicklaunch") {
        if let Err(e) = window.hide() {
            error!("monarch_utils::commands::hide_quicklaunch() Failed to hide quicklaunch! | Err: {e}");
            return Err(String::from("Failed to hide quicklaunch!"));
        }
        return Ok(());
    }
    error!("monarch_utils::commands::show_quicklaunch() Err: handle.get_window() returned None!");
    Err(String::from("Failed to get quicklaunch window!"))
}

/*
* Misc commands
*/

#[tauri::command]
/// Manually clear all images in the resources/cache directory
/// Don't return custom error message as they instead return the state of settings according to
/// backend.
pub fn clear_cached_images() {
    clear_all_cache();
}
