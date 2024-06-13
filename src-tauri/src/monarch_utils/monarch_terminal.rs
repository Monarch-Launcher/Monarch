use tauri::api::process::{Command, CommandChild, CommandEvent};
use tauri::{AppHandle, Manager, Window};
use lazy_static::lazy_static;
use std::sync::Mutex;
use anyhow::{bail, Context, Result};
use log::{error, warn};
use crate::monarch_utils::monarch_windows::MiniWindow;

static mut APP_HANDLE: Option<Box<AppHandle>> = None; // Global apphadle used by backend when no command
                                                     // was called from frontend.
lazy_static! {
    static ref RUNNING_PROCESS: Mutex<Option<CommandChild>> = Mutex::new(None);
}

/// Runs specified command in OS terminal.
///
/// This function is OS agnostic, however it currently requires gnome-terminal in Linux.
///
/// This function may contain code injection vaulnerabilities. In that case they will be identified
/// and patched later. It should be fine for now as users can't run arbitrary code through it yet,
/// only Monarch runs specific commands through it. Either they are hard-coded or they are run
/// through another program like Steamcmd, which should perform it's own sanitizing.
pub async fn run_in_terminal(command: &str) -> Result<()> {
    let term_window: Window;
    unsafe {
        if APP_HANDLE.is_none() {
            bail!("monarch_terminal::run_in_terminal() | Err No backend APP_HANDLE found! (Is None)");
        }
    
        if APP_HANDLE.clone().unwrap().as_ref().get_window("terminal").is_none() {
            warn!("No terminal emulator running! Creating new instance...");
            let window = MiniWindow::new("terminal", "src/terminal/terminal.html", 1280.0, 720.0);
            window.build_window(APP_HANDLE.clone().unwrap().as_ref()).await.with_context(|| "monarch_terminal::run_in_terminal() Failed to build terminal window! | Err ")?;
        }

        term_window = APP_HANDLE.clone().unwrap().as_ref().get_window("terminal").unwrap();
        term_window.show()?;
    }


    #[cfg(target_os = "windows")]
    let child_result = Command::new("sh").args(["-i", "-c", command]).spawn();

    #[cfg(target_os = "macos")]
    let child_result = Command::new("sh").args(["-i", "-c", command]).spawn();

    #[cfg(target_os = "linux")]
    let child_result = Command::new("sh").args(["-i", "-c", command]).spawn();

    let mut rx = match child_result {
        Ok(child) => {
            *RUNNING_PROCESS.lock().unwrap() = Some(child.1);
            child.0
        }
        Err(e) => {
            if let Err(e) = term_window.close() {
                error!("monarch_terminal::run_in_terminal() Failed to close terminal window! | Err {e}");
            }
            bail!("monarch_terminal::run_in_terminal() Failed running: {command} in terminal! | Err {e}")
        }
    };

    // Loop over all other child events (stdout, stderr, termination)
    while let Some(event) = rx.recv().await { // Loop through events
        if let CommandEvent::Stdout(out_line) = &event { // Verify it's a new
                                                                // line
            if let Err(e) = term_window.emit("stdout", &out_line) {
                warn!("monarch_terminal::run_in_terminal() Failed to send line: {out_line} to terminal window | Err {e}");
            }
        }
        if let CommandEvent::Stderr(err_line) = &event { // Verify it's a new
            if let Err(e) = term_window.emit("stderr", &err_line) {
                warn!("monarch_terminal::run_in_terminal() Failed to send line: {err_line} to terminal window | Err {e}");
            }
        }
        if let CommandEvent::Terminated(_payload) = &event { // Verify it's a new
            *RUNNING_PROCESS.lock().unwrap() = None;
            if let Err(e) = term_window.close() {
                warn!("monarch_terminal::run_in_terminal() Failed to close terminal window! | Err {e}");
            }
            return Ok(()); // Exit if child was terminated
        }
    }
    bail!("monarch_terminal::run_in_terminal() Exited before child process terminated!")
}

/// Sets the global APP_HANDLE used by monarch_windows backend.
pub fn set_apphande(handle: AppHandle) {
    unsafe { APP_HANDLE = Some(Box::new(handle)) }
}

/// Send stdin: &str to stdin of RUNNING_PROCESS.
/// Another potential code injection vaulnerability.
pub fn write_stdin(stdin: &str) -> Result<()> {
    // If RUNNING_PROCESS is Some
    if let Some(mut child) = RUNNING_PROCESS.lock().unwrap().take() {
        child.write(stdin.as_bytes()).with_context(|| "monarch_terminal::write_stdin() Error while writing to RUNNING_PROCESS stdin! | Err ")?;
        return Ok(())
    }

    // If RUNNING_PROCESS is None
    bail!("monarch_terminal::write_stdin() No currently running process found! | Err RUNNING_PROCESS = None")
}
