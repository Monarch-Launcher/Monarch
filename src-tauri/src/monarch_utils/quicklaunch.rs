use crate::monarch_utils::monarch_windows::MiniWindow;
use anyhow::Result;
use tauri::AppHandle;

/// Builds a new quicklaunch window.
/// Starts as hidden unitl user presses quicklaunch shortcut.
pub async fn init_quicklaunch(handle: &AppHandle) -> Result<()> {
    let window = MiniWindow::new("quicklaunch", "src/quicklaunch/index.html", 854.0, 480.0);
    window.build_window(handle).await?;
    window.hide_window(handle)?;
    Ok(())
}
