/*
* Custom terminal emulator implementation for Monarch to run CMD commands 
* or applications, such as SteamCMD. 
*
* Massive thanks goes to https://github.com/marc2332/tauri-terminal for 
* showing how to write a terminal in Tauri.
*/

use anyhow::{bail, Context, Result};
use log::error;
use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use super::monarch_windows::MiniWindow;
use tauri::{AppHandle, Manager};
use std::sync::{Arc, Mutex};
use tauri::{async_runtime::Mutex as AsyncMutex, State};
use std::io::{BufRead, BufReader, Read, Write};

static mut HANDLE: Option<Arc<Mutex<AppHandle>>> = None;

pub struct AppState {
    pty_pair: Arc<AsyncMutex<PtyPair>>,
    writer: Arc<AsyncMutex<Box<dyn Write + Send>>>,
    reader: Arc<AsyncMutex<BufReader<Box<dyn Read + Send>>>>,
}

/// Initializes the Monarch Terminal instance.
pub async fn init_monarch_terminal(handle: &AppHandle) -> Result<()> {
    let term_window: MiniWindow = MiniWindow::new("terminal", "/src/terminal/terminal.html", 854.0, 480.0);
    term_window.build_window(handle).await.with_context(|| "monarch_terminal::init_monarch_terminal() -> ")?;
    term_window.hide_window(handle).with_context(|| "monarch_terminal::init_monarch_terminal() -> ")?;

    let w = handle.get_window("terminal").unwrap();
    w.set_closable(false)?;

    unsafe {
        HANDLE = Some(Arc::new(Mutex::new(handle.clone())));
    }

    Ok(())
}

/// Run a command in a new process and display to the user in a custom terminal window.
pub async fn run_in_terminal(command: &str) -> Result<()> {
    let handle = unsafe {
        match &HANDLE {
            Some(h) => h,
            None => {
                bail!("monarch_terminal::run_in_terminal() | Err: HANDLE is None!")
            }
        } 
    };

    let locked_handle = handle.lock().unwrap();

    let w = locked_handle.get_window("terminal");

    if w.is_none() {
        error!("monarch_terminal::run_in_terminal() handle.get_window() returned None! Attempting to re-init monarch terminal...");
        bail!("monarch_terminal::run_in_terminal() handle.get_window() returned None, even after reinitializing monarch terminal!")
    }
    
    w.clone().unwrap().show().with_context(|| "monarch_terminal::run_in_terminal() Failed to run window.show() | Err: ")?;

    std::thread::sleep(std::time::Duration::from_secs(3));

    w.unwrap().hide().with_context(|| "monarch_terminal::run_in_terminal() Failed to run window.show() | Err: ")?;
    

    /*
    let pty_system = native_pty_system();

    let mut pair = pty_system.openpty(PtySize {
        rows: 60,
        cols: 160,
        // Not all systems support pixel_width, pixel_height,
        // but it is good practice to set it to something
        // that matches the size of the selected font.  That
        // is more complex than can be shown here in this
        // brief example though!
        pixel_width: 0,
        pixel_height: 0,
    })?;

    // Spawn a shell into the pty
    let mut cmd = CommandBuilder::new_default_prog();
    let shell = cmd.get_shell();

    cmd = CommandBuilder::new(shell);
    cmd.args(vec!["-c", command]);
    let child = pair
        .slave
        .spawn_command(cmd)
        .with_context(|| "Failed to spawn child commnad! | Err: ")?;

    // Read and parse output from the pty with reader
    let mut reader = pair.master.try_clone_reader()?;

    // Send data to the pty by writing to the master
    writeln!(pair.master.take_writer()?, "ls -l\r\n")?;

    */
    Ok(())
}

/*
* TODO: Figure out why tauri-temrinals (working()) function does not
* get mutable borrow issues, while read_from_pty() does.
*/

pub async fn read_from_pty(state: State<'_, AppState>) -> Result<String, ()> {
    Ok(String::new())
}
/* 
pub async fn read_from_pty(state: State<'_, AppState>) -> Result<String, ()> {
    let mut reader = state.reader.lock().await;

    // Read all available text
    // .with_context(|| "monarch_terminal::read_from_pty() Failed to fill buffer! | Err: ")
    let data = reader.fill_buf().map_err(|_| ())?;

    // Send te data to the webview if necessary
    //.with_context(|| "monarch_terminal::read_from_pty() Failed to send data to webview! | Err: ") 
    if data.len() > 0 {
        std::str::from_utf8(data)
            .map(|v| Some(v.to_string())).map_err(|_| ())?;
  }

    reader.consume(data.len());

    //.with_context(|| "monarch_terminal::read_from_pty() Failed to parse bytes as String! | Err: ") 
    let data_str = String::from_utf8(data.to_vec()).unwrap();
    Ok(data_str)
}
*/

async fn working(state: State<'_, AppState>) -> Result<Option<String>, ()> {
    let mut reader = state.reader.lock().await;
    let data = {
        // Read all available text
        let data = reader.fill_buf().map_err(|_| ())?;

        // Send te data to the webview if necessary
        if data.len() > 0 {
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