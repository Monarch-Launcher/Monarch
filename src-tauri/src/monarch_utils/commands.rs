use core::result::Result;
use log::{error, info};
use std::env;
use std::{path::PathBuf, process::Command};
use tauri::{AppHandle, Manager};

use super::housekeeping::clear_all_cache;
use super::monarch_credentials::{delete_credentials, set_credentials};
use super::monarch_logger::get_log_dir;
use super::monarch_settings::{
    get_settings_state, set_default_settings, set_settings_state, write_settings, LauncherSettings,
    Settings,
};
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
pub fn get_settings() -> Settings {
    get_settings_state()
}

#[tauri::command]
/// Write setting to settings.toml
/// Don't return custom error message as they instead return the state of settings according to
/// backend.
pub fn set_settings(settings: Settings) -> Result<Settings, String> {
    match write_settings(settings) {
        Ok(ret_settings) => {
            set_settings_state(ret_settings.clone());
            Ok(ret_settings)
        }
        Err(e) => {
            error!(
                "monarch_utils::commands::set_settings() -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            Err(String::from("Failed to write new settings!"))
        }
    }
}

#[tauri::command]
/// Write default settings to settings.toml
/// Don't return custom error message as they instead return the state of settings according to
/// backend.
pub fn revert_settings() -> Result<Settings, String> {
    match set_default_settings() {
        Ok(ret_settings) => {
            set_settings_state(ret_settings.clone());
            Ok(ret_settings)
        }
        Err(e) => {
            error!(
                "monarch_utils::commands::revert_settings() -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            Err(String::from("Failed to reset to default settings!"))
        }
    }
}

/*
* User credentials related commands
*/

#[tauri::command]
/// Set password in secure store
/// TODO: Better error handling if write_settings() fails.
pub fn set_password(platform: String, username: String, password: String) -> Result<(), String> {
    let mut settings: Settings = get_settings_state();
    let launcher_settings: &mut LauncherSettings = match platform.as_str() {
        "steam" => &mut settings.steam,
        "epic" => &mut settings.epic,
        _ => {
            error!(
                "monarch_utils::commands::set_password() | Err: Invalid platform: {}",
                platform
            );
            return Err(String::from(
                "Trying to write user credentials for unknown platform.",
            ));
        }
    };

    if !launcher_settings.username.is_empty() {
        error!("monarch_utils::commands::set_password() | Err: User already defined in settings.",);
        return Err(String::from(
            "Monarch currently does not support more than one saved user!",
        ));
    }

    if let Err(e) = set_credentials(&platform, &username, &password) {
        error!(
            "monarch_utils::commands::set_password() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from("Something went wrong setting new password!"));
    }

    launcher_settings.username = username;
    set_settings_state(settings.clone());
    write_settings(settings).unwrap();
    Ok(())
}

#[tauri::command]
/// Delete password in secure store
/// TODO: Better error handling if write_settings() fails.
pub fn delete_password(platform: String) -> Result<(), String> {
    let mut settings: Settings = get_settings_state();
    let launcher_settings: &mut LauncherSettings = match platform.as_str() {
        "steam" => &mut settings.steam,
        "epic" => &mut settings.epic,
        _ => {
            error!(
                "monarch_utils::commands::set_password() | Err: Invalid platform: {}",
                platform
            );
            return Err(String::from(
                "Trying to write user credentials for unknown platform.",
            ));
        }
    };

    if let Err(e) = delete_credentials(&platform, &launcher_settings.username) {
        error!(
            "monarch_utils::commands::delete_password() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from(
            "Something went wrong while deleting credentials!",
        ));
    }

    launcher_settings.username = String::new();
    set_settings_state(settings.clone());
    write_settings(settings).unwrap();
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
    // First check if Monarch is being run under Wayland
    if cfg!(target_os = "linux") {
        return env::var("WAYLAND_DISPLAY").is_err(); // If WAYLAND_DISPLAY is set at all, assume Wayland is used
    }

    // Then check if quicklaunch is disabled in settings
    get_settings_state().quicklaunch.enabled
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

    // Currently this code snippet basically just disables window decorations for a
    // cleaner quicklaunch look
    if let Err(e) = window.set_quicklaunch_stuff(&handle) {
        error!(
            "monarch_utils::commands::init_quicklaunch() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from(
            "Failed to set quicklaunch specific properties!",
        ));
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
