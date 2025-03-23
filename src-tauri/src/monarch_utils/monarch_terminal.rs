/*
* Custom terminal emulator implementation for Monarch to run CMD commands
* or applications, such as SteamCMD.
*
* Massive thanks goes to https://github.com/marc2332/tauri-terminal for
* showing how to write a terminal in Tauri.
*/

use super::monarch_windows::MiniWindow;
use anyhow::{bail, Context, Result};
use log::error;
use log::info;
use once_cell::sync::Lazy;
use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use std::io::{BufRead, BufReader, Read, Write};
use std::sync::Arc;
use tauri::async_runtime::Mutex as AsyncMutex;
use tauri::{AppHandle, Manager};

pub struct AppState {
    pty_pair: Arc<AsyncMutex<PtyPair>>,
    writer: Arc<AsyncMutex<Box<dyn Write + Send>>>,
    reader: Arc<AsyncMutex<BufReader<Box<dyn Read + Send>>>>,
}

static mut APPSTATE: Lazy<Option<AppState>> = Lazy::<Option<AppState>>::new(|| None);

/// Run a command in a new process and display to the user in a custom terminal window.
pub async fn run_in_terminal(handle: &AppHandle, command: &str) -> Result<()> {
    info!("Running command in terminal: \n{command}");

    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows: 80,
        cols: 160,
        // Not all systems support pixel_width, pixel_height,
        // but it is good practice to set it to something
        // that matches the size of the selected font.  That
        // is more complex than can be shown here in this
        // brief example though!
        pixel_width: 0,
        pixel_height: 0,
    })?;

    // Read and parse output from the pty with reader
    let reader = pair.master.try_clone_reader().unwrap();
    let writer = pair.master.take_writer().unwrap();

    // Spawn a shell into the pty
    let mut cmd = CommandBuilder::new_default_prog();
    let shell = cmd.get_shell();

    cmd = CommandBuilder::new(shell);
    cmd.args(vec!["-c", command]);
    let mut child = pair
        .slave
        .spawn_command(cmd)
        .with_context(|| "Failed to spawn child commnad! | Err: ")?;

    unsafe {
        *APPSTATE = Some(AppState {
            pty_pair: Arc::new(AsyncMutex::new(pair)),
            writer: Arc::new(AsyncMutex::new(writer)),
            reader: Arc::new(AsyncMutex::new(BufReader::new(reader))),
        });
    };

    if let Err(e) = create_terminal_window(handle).await {
        error!("monarch_terminal::run_in_terminal() -> {e}");
    }

    child
        .wait()
        .with_context(|| "Something went wrong while waiting for child process to finish!")?;

    if let Err(e) = close_terminal_window(handle).await {
        error!("monarch_terminal::run_in_terminal() -> {e}");
    }

    Ok(())
}

/// Creates a new Monarch terminal window, meant to be called from frontend.
pub async fn create_terminal_window(handle: &AppHandle) -> Result<()> {
    let term_window: MiniWindow =
        MiniWindow::new("terminal", "/src/terminal/terminal.html", 854.0, 480.0);
    term_window
        .build_window(handle)
        .await
        .with_context(|| "monarch_terminal::run_in_terminal() -> ")?;

    let w_opt = handle.get_window("terminal");
    let w = match w_opt {
        Some(w) => w,
        None => {
            error!("monarch_terminal::run_in_terminal() handle.get_window() returned None!");
            bail!("monarch_terminal::run_in_terminal() handle.get_window() returned None!")
        }
    };

    w.set_closable(false)?;
    w.show().with_context(|| "monarch_terminal::run_in_terminal() Failed to run window.show() after building terminal window! | Err: ")?;

    Ok(())
}

/// Close terminal window. Meant to be called from frontend.
pub async fn close_terminal_window(handle: &AppHandle) -> Result<()> {
    let w_opt = handle.get_window("terminal");
    match w_opt {
        Some(w) => {
            w.close().with_context(|| "monarch_terminal::run_in_terminal() Failed to run window.hide() after child process exited! | Err: ")?;
            Ok(())
        }
        None => {
            bail!("monarch_terminal::run_in_terminal() No window called 'terminal' found! Must not exist. | Err: handle.get_window(\"terminal\") returned None!")
        }
    }
}

pub async fn read_from_pty() -> Result<Option<String>, ()> {
    unsafe {
        if (*APPSTATE).is_none() {
            return Err(());
        }
    }
    let state = unsafe { (*APPSTATE).as_ref().unwrap() };

    let mut reader = state.reader.lock().await;
    let data = {
        // Read all available text
        let data = reader.fill_buf().map_err(|_| ())?;

        // Send te data to the webview if necessary
        if !data.is_empty() {
            std::str::from_utf8(data)
                .map(|v| Some(v.to_string()))
                .map_err(|_| ())?
        } else {
            None
        }
    };

    if let Some(data) = &data {
        reader.consume(data.len());
    }

    Ok(data)
}

pub async fn write_to_pty(data: &str) -> Result<(), ()> {
    unsafe {
        if (*APPSTATE).is_none() {
            return Err(());
        }
    }
    let state = unsafe { (*APPSTATE).as_ref().unwrap() };

    write!(state.writer.lock().await, "{}", data).map_err(|_| ())
}

