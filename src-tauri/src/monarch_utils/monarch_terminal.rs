/*
* Monarch currently ships with a bundled Kitty instance.
* This makes it easier to run commandline programs that are
* needed, such as SteamCMD.
*
* Would be nice to replace Kitty wiht an in-house solution
* in future versions.
 */

use anyhow::{bail, Result};
use tauri::api::process::Command;

/// Run a command in a new process and display to the user in a custom terminal window.
pub async fn run_in_terminal(command: &str) -> Result<()> {
    // Spawn new child process
    #[cfg(target_os = "windows")]
    let child_result = Command::new("Start-Process")
        .args(["PowerShell", "-ArgumentList", command])
        .spawn();

    #[cfg(target_os = "macos")]
    let child_result = Command::new("kitty").args(["sh", "-c", command]).spawn();

    #[cfg(target_os = "linux")]
    let child_result = Command::new("kitty").args(["sh", "-c", command]).spawn();

    if let Err(e) = child_result {
        bail!("monarch_terminal::run_in_terminal() failed to run terminal command! | Err: {e}")
    }

    let _pid = child_result.unwrap().1.pid();

    Ok(())
}
