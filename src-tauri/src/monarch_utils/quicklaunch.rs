use anyhow::{bail, Context, Result};
use tauri::{AppHandle, Manager};

use crate::monarch_utils::monarch_settings::get_settings_state;
use crate::monarch_utils::monarch_windows::MiniWindow;
use tracing::{error, info};

use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

/// This function returns if quicklaunch is allowed to run or not.
pub fn quicklaunch_is_enabled() -> bool {
    get_settings_state().quicklaunch.enabled
}

/// Builds a new quicklaunch window.
/// Starts as hidden unitl user presses quicklaunch shortcut.
pub async fn init_quicklaunch(handle: &AppHandle) -> Result<()> {
    let window = MiniWindow::new(
        "quicklaunch",
        "/src/quicklaunch/quicklaunch.html",
        854.0,
        480.0,
    );

    window
        .build_window(&handle)
        .await
        .with_context(|| "monarch_utils::commands::init_quicklaunch() -> ")?;

    // Currently this code snippet basically just disables window decorations for a
    // cleaner quicklaunch look
    window
        .set_quicklaunch_stuff(&handle)
        .with_context(|| "monarch_utils::commands::init_quicklaunch() -> ")?;

    window
        .hide_window(&handle)
        .with_context(|| "monarch_utils::commands::init_quicklaunch() -> ")?;

    // Setup shortcut handlers for quicklaunch
    let show_shortcut = Shortcut::new(Some(Modifiers::SUPER), Code::Backspace);
    let hide_shortcut = Shortcut::new(None, Code::Escape);

    handle.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |_app, shortcut, event| {
                info!("Pressed: {shortcut}");

                if shortcut == &show_shortcut {
                    if event.state() == ShortcutState::Pressed {
                        if let Err(e) = show_quicklaunch(_app) {
                            error!("Shortcut handler error! Error while executing show_quicklaunch() | Err: {e}");
                        }
                    }
                }
                if shortcut == &hide_shortcut {
                    if event.state() == ShortcutState::Pressed {
                        if let Err(e) = hide_quicklaunch(_app) {
                            error!("Shortcut handler error! Error while executing hide_quicklaunch() | Err: {e}");
                        }
                    }
                }
            })
            .build(),
    ).with_context(|| "quicklaunch::init_quicklaunch() Failed to init quicklaunch! | Err: ")?;

    handle.global_shortcut().register(show_shortcut)?;
    info!("Registered shortcut: {show_shortcut}");

    handle.global_shortcut().register(hide_shortcut)?;
    info!("Registered shortcut: {hide_shortcut}");

    info!("Finished initializing quicklaunch!");
    Ok(())
}

/// Show quicklaunch and focus quicklaunch window
pub fn show_quicklaunch(handle: &AppHandle) -> Result<()> {
    if let Some(window) = handle.get_webview_window("quicklaunch") {
        if let Err(e) = window.show() {
            error!("monarch_utils::commands::show_quicklaunch() Failed to show quicklaunch! | Err: {e}");
            bail!("Failed to hide quicklaunch window!");
        }
        if let Err(e) = window.set_focus() {
            error!("monarch_utils::commands::show_quicklaunch() Failed to set focus to quicklaunch! | Err: {e}");
            bail!("Failed to show quicklaunch!");
        }
        return Ok(());
    }
    error!("monarch_utils::commands::show_quicklaunch() Err: handle.get_window() returned None!");
    bail!("Failed to get quicklaunch window!")
}

/// Hide quicklaunch window
pub fn hide_quicklaunch(handle: &AppHandle) -> Result<()> {
    if let Some(window) = handle.get_webview_window("quicklaunch") {
        if let Err(e) = window.hide() {
            error!("monarch_utils::commands::hide_quicklaunch() Failed to hide quicklaunch! | Err: {e}");
            bail!("Failed to hide quicklaunch!");
        }
        return Ok(());
    }
    error!("monarch_utils::commands::show_quicklaunch() Err: handle.get_window() returned None!");
    bail!("Failed to get quicklaunch window!")
}
