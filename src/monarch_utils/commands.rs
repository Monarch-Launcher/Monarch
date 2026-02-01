use core::result::Result;
use std::path::PathBuf;
use tracing::error;

use crate::monarch_utils::monarch_fs;

use super::housekeeping::clear_all_cache;
use super::monarch_credentials::{delete_credentials, set_credentials};
use super::monarch_logger::get_log_dir;
use super::monarch_settings::{
    get_settings_state, set_default_settings, set_settings_state, write_settings, LauncherSettings,
    Settings,
};

/*
*   Settings related commands
*
*   All settings related commands return the new settings as read by the backend to ensure both
*   frontend and backend agree on current settings.
*   Settings are wrapped in Result<> type to also tell frontend the success or failure of the command.
*   tauri::commands don't return the actual error message. Instead they write an easier error to understand for the user.
*/

pub fn open_logs() -> Result<(), String> {
    let log_path: PathBuf = get_log_dir();
    let res = if cfg!(target_os = "windows") {
        std::process::Command::new("explorer").arg(log_path).spawn()
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(log_path).spawn()
    } else {
        std::process::Command::new("xdg-open").arg(log_path).spawn()
    };

    match res {
        Ok(_) => Ok(()),
        Err(e) => {
            error!(
                "monarch_utils::commands::open_logs() Failed to open logs | Err: {}",
                e
            );
            Err(String::from("Failed to open logs in file manager."))
        }
    }
}

/// Returns settings read from settings.toml
pub fn get_settings() -> Settings {
    get_settings_state()
}

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

/// Write default settings to settings.toml
/// Don't return custom error message as they instead return the state of settings according to
/// backend.
pub fn default_settings() -> Result<Settings, Settings> {
    if let Err(e) = set_default_settings() {
        error!(
            "monarch_utils::commands::default_settings() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
    }
    Ok(get_settings_state())
}

/*
* User credentials related commands
*/

/// Set password in secure store
/// TODO: Better error handling if write_settings() fails.
pub fn set_password(
    platform: String,
    username: String,
    password: String,
) -> Result<Settings, String> {
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
    write_settings(settings.clone()).unwrap();
    Ok(settings)
}

/// Delete password in secure store
/// TODO: Better error handling if write_settings() fails.
pub fn delete_password(platform: String) -> Result<Settings, String> {
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
    write_settings(settings.clone()).unwrap();
    Ok(settings)
}

/// Set secret in secure store
pub fn set_secret(platform: String, secret: String) -> Result<Settings, String> {
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

    if let Err(e) = set_credentials(
        &format!("{platform}-secret"),
        &launcher_settings.username,
        &secret,
    ) {
        error!(
            "monarch_utils::commands::set_secret() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from("Something went wrong setting new secret!"));
    }
    launcher_settings.twofa = true;

    set_settings_state(settings.clone());
    write_settings(settings.clone()).unwrap();
    Ok(settings)
}

/// Delete secret in secure store
pub fn delete_secret(platform: String) -> Result<Settings, String> {
    let mut settings: Settings = get_settings_state();
    let launcher_settings: &mut LauncherSettings = match platform.as_str() {
        "steam" => &mut settings.steam,
        "epic" => &mut settings.epic,
        _ => {
            error!(
                "monarch_utils::commands::delete_secret() | Err: Invalid platform: {}",
                platform
            );
            return Err(String::from(
                "Trying to write user credentials for unknown platform.",
            ));
        }
    };

    if let Err(e) = delete_credentials(&format!("{platform}-secret"), &launcher_settings.username) {
        error!(
            "monarch_utils::commands::delete_password() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from("Something went wrong while deleting secret!"));
    }

    launcher_settings.twofa = false;

    set_settings_state(settings.clone());
    write_settings(settings.clone()).unwrap();
    Ok(settings)
}

/*
* Misc commands
*/

/*
/// Builds a new terminal window.
/// Starts as hidden until Monarch runs commands.
pub async fn open_terminal(handle: AppHandle) {
    create_terminal_window(&handle).await.unwrap();
}

/// Builds a new terminal window.
/// Starts as hidden until Monarch runs commands.
pub async fn close_terminal(handle: AppHandle) {
    close_terminal_window(&handle).await.unwrap();
}

/// Functions for frontend terminal window to read content
/// of terminal command being run.
pub async fn async_read_from_pty() -> Result<Option<String>, ()> {
    match read_from_pty().await {
        Ok(s) => Ok(s),
        Err(_e) => {
            error!("monarch_utils::commands::async_read_from_pty() Recieved error when reading pty! | Err:");
            Err(())
        }
    }
}

pub async fn async_write_to_pty(data: &str) -> Result<(), ()> {
    match write_to_pty(data).await {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}
    */

/// Manually clear all images in the resources/cache directory
/// Don't return custom error message as they instead return the state of settings according to
/// backend.
pub fn clear_cached_images() {
    clear_all_cache();
}

/*
/// Code found at https://github.com/phcode-dev/phoenix-desktop/pull/162/files
/// for implementing zoom.
pub fn zoom_window(window: WebviewWindow, scale_factor: f64) {
    let _ = window.with_webview(move |webview| {
        #[cfg(target_os = "linux")]
        {
            use webkit2gtk::WebViewExt;
            webview.inner().set_zoom_level(scale_factor);
        }

        #[cfg(windows)]
        unsafe {
            // see https://docs.rs/webview2-com/0.19.1/webview2_com/Microsoft/Web/WebView2/Win32/struct.ICoreWebView2Controller.html
            webview.controller().SetZoomFactor(scale_factor).unwrap();
        }

        #[cfg(target_os = "macos")]
        unsafe {
            /*
            TODO: Troubleshoot likely memory issues causing application crash.
            use objc::msg_send;
            let inner = webview.inner();
            let _: () = msg_send![class!(inner), setPageZoom: scale_factor];
            */
        }
    });
}
*/

pub fn get_cache_size() -> Result<u64, String> {
    let cache_dir = monarch_fs::get_resources_cache();
    match fs_extra::dir::get_size(&cache_dir) {
        Ok(size) => Ok(size as u64),
        Err(e) => {
            error!(
                "monarch_utils::commands::get_cache_size() Failed to read size of: {} | Err: {}",
                cache_dir.display(),
                e
            );
            Err(String::from("Failed to read size of cache directory."))
        }
    }
}
