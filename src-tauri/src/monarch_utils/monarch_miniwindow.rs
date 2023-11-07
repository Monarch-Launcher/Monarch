use log::{error, warn};
use tauri::window::{Window, WindowBuilder};
use tauri::{AppHandle, Manager, PhysicalSize, WindowUrl};

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
            width: width,
            height: height,
        }
    }

    /// Builds a window. Must be async on Windows to not deadlock.
    pub async fn build_window(&self, handle: &AppHandle) -> Result<(), String> {
        match WindowBuilder::new(
            handle,
            &self.name,
            WindowUrl::External(self.url.parse().unwrap())
        )
        .always_on_top(true)
        .center()
        .decorations(false)
        .focused(true)
        .skip_taskbar(true)
        .visible(true)
        .build()
        {
            Ok(window) => {
                let scale: f64 = get_scale(&window);
                let size: PhysicalSize<u32> =
                    PhysicalSize::new((self.width * scale) as u32, (self.height * scale) as u32);

                if let Err(e) = window.set_size(size) {
                    error!("Failed to set new mini-window size! | Message: {:?}", e);
                }
                if let Err(e) = window.center() {
                    error!("Failed to center new mini-window! | Message: {:?}", e);
                }
                
                Ok(())
            }
            Err(e) => {
                error!("Failed to build new mini-window window! | Message: {:?}", e);
                return Err("Failed to build new mini-window window!".to_string());
            }
        }
    }

    /// Shows a window with specified label
    pub fn show_window(&self, handle: &AppHandle) -> Result<(), String> {
        match handle.get_window(&self.name) {
            Some(window) => match window.show() {
                Ok(_) => return Ok(()),
                Err(e) => {
                    error!(
                        "Failed to show mini-window: {} | Message: {:?}",
                        self.name, e
                    );
                    return Err("Failed to show mini-window!".to_string());
                }
            },
            None => {
                warn!(
                    "Failed to find mini-window: {:?} (Possibly first time creating it)",
                    self.name
                );
                return Err("Failed to find mini-window!".to_string());
            }
        }
    }

    /// Hides a window with specified label
    pub fn hide_window(handle: &AppHandle, label: &str) -> Result<(), String> {
        match handle.get_window(label) {
            Some(window) => match window.hide() {
                Ok(_) => Ok(()),
                Err(e) => {
                    error!("Failed to hide mini-window: {label} | Message: {e}");
                    Err("Failed to hide mini-window!".to_string())
                }
            },
            None => {
                warn!("Failed to find mini-window: {:?}", label);
                Err("Failed to find mini-window!".to_string())
            }
        }
    }

    pub fn close_window(handle: &AppHandle, label: &str) -> Result<(), String> {
        match handle.get_window(label) {
            Some(window) => match window.close() {
                Ok(_) => Ok(()),
                Err(e) => {
                    error!("Failed to close mini-window: {label} | Message: {e}");
                    Err("Failed to close mini-window!".to_string())
                }
            },
            None => {
                warn!("Failed to find mini-window: {label}");
                Err("Failed to find mini-window!".to_string())
            }
        }
    }
}

// Returns scale to use based on monitor resolution
fn get_scale(window: &Window) -> f64 {
    match window.current_monitor() {
        Ok(monitor_opt) => match monitor_opt {
            Some(monitor) => {
                return monitor.size().height as f64 / STANDARD_HEIGHT;
            }
            None => return 1.0,
        },
        Err(e) => {
            error!("window.current_monitor() Failed | Message: {:?}", e);
            return 1.0; // Return default scale of 1
        }
    }
}
