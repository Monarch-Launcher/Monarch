use anyhow::{bail, Context, Result};
use log::{error, warn};
use tauri::window::{Window, WindowBuilder};
use tauri::{AppHandle, Manager, PhysicalSize, WindowUrl};
use tauri::api::process::{Command, CommandEvent};

static mut APP_HANDLE: Option<Box<AppHandle>> = None; // Global apphadle used by backend when no command
                                                  // was called from frontend.
static STANDARD_HEIGHT: f64 = 1080.0; // Standard monitor resultion used as scale

pub struct MiniWindow {
    name: String,
    url: String,
    width: f64,
    height: f64,
}

impl MiniWindow {
    /// Returns a new "mini-window"
    pub fn new(name: &str, url: &str, width: f64, height: f64) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
            width,
            height,
        }
    }

    /// Builds a window. Must be async on Windows to not deadlock.
    pub async fn build_window(&self, handle: &AppHandle) -> Result<()> {
        let window_url: WindowUrl = if self.url.starts_with("https") {
            WindowUrl::External(self.url.parse().unwrap())
        } else {
            WindowUrl::App(self.url.parse().unwrap())
        };

        let window: Window = WindowBuilder::new(
            handle,
            &self.name,
                    window_url,
            )
            .always_on_top(true)
            .center()
            .decorations(false)
            .focused(true)
            .skip_taskbar(true)
            .visible(true)
            .build()
            .with_context(|| "monarch_windows::build_window() Failed to build window! | Err: ")?;

        let scale: f64 = get_scale(&window);
        let size: PhysicalSize<u32> =
            PhysicalSize::new((self.width * scale) as u32, (self.height * scale) as u32);

        if let Err(e) = window.set_size(size) {
            error!(
                "monarch_windows::build_window() Failed to set new window size! | Err: {:#}",
                e
            );
        }
        if let Err(e) = window.center() {
            error!(
                "monarch_windows::build_window() Failed to center new window! | Err: {:#}",
                e
            );
        }

        Ok(())
    }

    /// Shows a window with specified label
    pub fn show_window(&self, handle: &AppHandle) -> Result<()> {
        let window = handle.get_window(&self.name).with_context(|| {
            format!(
                "monarch_windows::show_window() Failed to find window: {} | Err:",
                self.name
            )
        })?;

        window.show().with_context(|| {
            format!(
                "monarch_windows::show_window() Failed to show window: {} | Err:",
                self.name
            )
        })
    }

    /// Hides a window with specified label
    pub fn hide_window(&self, handle: &AppHandle) -> Result<()> {
        let window = handle.get_window(&self.name).with_context(|| {
            format!(
                "monarch_windows::hide_window() Failed to find window: {} | Err:",
                self.name
            )
        })?;

        window.hide().with_context(|| -> String {
            format!(
                "monarch_windows::hide_window() Failed to hide window: {} | Err:",
                self.name
            )
        })
    }

    pub fn close_window(&self, handle: &AppHandle) -> Result<()> {
        let window = handle.get_window(&self.name).with_context(|| {
            format!(
                "monarch_windows::close_window() Failed to find window: {} | Err:",
                self.name
            )
        })?;

        window.close().with_context(|| {
            format!(
                "monarch_windows::close_window() Failed to close window: {} | Err:",
                self.name
            )
        })
    }
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
            bail!("monarch_windows::run_in_terminal() | Err No backend APP_HANDLE found! (Is None)");
        }
    
        if APP_HANDLE.clone().unwrap().as_ref().get_window("terminal").is_none() {
            warn!("No terminal emulator running! Creating new instance...");
            let window = MiniWindow::new("terminal", "src/terminal/terminal.html", 1280.0, 720.0);
            window.build_window(APP_HANDLE.clone().unwrap().as_ref()).await.with_context(|| "monarch_windows::run_in_terminal() Failed to build terminal window! | Err ")?;
        }

        term_window = APP_HANDLE.clone().unwrap().as_ref().get_window("terminal").unwrap();
        term_window.show()?;
    }

    let child_result = Command::new("script").args(["-c", command]).spawn();

    let (mut rx, mut cmd_child) = match child_result {
        Ok(child) => child,
        Err(e) => {
            if let Err(e) = term_window.close() {
                error!("monarch_windows::run_in_terminal() Failed to close terminal window! | Err {e}");
            }
            bail!("monarch_windows::run_in_terminal() Failed running: {command} in terminal! | Err {e}")
        }
    };

    unsafe {
        let handle_clone = APP_HANDLE.clone();

        // Spawn event_listener to handle input to child
        tauri::async_runtime::spawn(async move {
            handle_clone.unwrap().listen_global("stdin", move |event| {
                if let Some(payload) = event.payload() {
                    
                }
            });
            
            // The .listen_global() is non-blocking. Add code inside closure to send stdin down
            // here.
            //
        });
    }

    // Loop over all other child events (stdout, stderr, termination)
    while let Some(event) = rx.recv().await { // Loop through events
        if let CommandEvent::Stdout(out_line) = &event { // Verify it's a new
                                                                // line
            if let Err(e) = term_window.emit("stdout", &out_line) {
                warn!("monarch_windows::run_in_terminal() Failed to send line: {out_line} to terminal window | Err {e}");
            }
        }
        if let CommandEvent::Stderr(err_line) = &event { // Verify it's a new
            if let Err(e) = term_window.emit("stderr", &err_line) {
                warn!("monarch_windows::run_in_terminal() Failed to send line: {err_line} to terminal window | Err {e}");
            }
        }
        if let CommandEvent::Terminated(_payload) = &event { // Verify it's a new
            if let Err(e) = term_window.close() {
                warn!("monarch_windows::run_in_terminal() Failed to close terminal window! | Err {e}");
            }
            return Ok(()); // Exit if child was terminated
        }
    }
    bail!("monarch_windows::run_in_terminal() Exited before child process terminated!")

}

/// Sets the global APP_HANDLE used by monarch_windows backend.
pub fn set_apphande(handle: AppHandle) {
    unsafe { APP_HANDLE = Some(Box::new(handle)) }
}

/// Returns scale to use based on monitor resolution
fn get_scale(window: &Window) -> f64 {
    if let Ok(monitor_option) = window.current_monitor() {
        match monitor_option {
            Some(monitor) => return monitor.size().height as f64 / STANDARD_HEIGHT,
            None => return 1.0,
        }
    }
    1.0
}
