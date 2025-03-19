/*
* Monarch currently ships with a bundled Kitty instance.
* This makes it easier to run commandline programs that are
* needed, such as SteamCMD.
*
* TODO: Replace with a better future implementation that allows
* Monarch to more easily read the stdout and progress of commands
* run in terminal.
 */

use anyhow::{bail, Result};
use std::process::Command;

/// Run a command in a new process and display to the user in a custom terminal window.
pub async fn run_in_terminal(command: &str) -> Result<()> {
    // Spawn new child process
    #[cfg(target_os = "windows")]
    let child_result = Command::new("powershell.exe")
        .args(["-noexit", &format!("\"{}\"", command)])
        .spawn();

    #[cfg(target_os = "macos")]
    let child_result = Command::new("kitty").args(["sh", "-c", command]).spawn();

    #[cfg(target_os = "linux")]
    let child_result = Command::new("kitty").args(["sh", "-c", command]).spawn();

    if let Err(e) = child_result {
        bail!("monarch_terminal::run_in_terminal() failed to run terminal command! | Err: {e}")
    }

    // Output can be used in future to view result of command
    let child = child_result.unwrap();
    let _out = child.wait_with_output()?;

    Ok(())
}
