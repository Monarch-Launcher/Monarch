use log::error;
use tauri::window::{Window, WindowBuilder};
use tauri::{AppHandle, Manager, PhysicalSize, WindowUrl};
use anyhow::{Result, anyhow, Context};
use serde::{Serialize, Deserialize};

static STANDARD_HEIGHT: f64 = 1080.0; // Standard monitor resultion used as scale

#[derive(Serialize, Deserialize)]
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
    pub async fn build_window(&self, handle: &AppHandle) -> Result<()> {
        let window_url: WindowUrl;

        if self.url.contains("http") { // Assume that all websites will be http(s).
            window_url = WindowUrl::External(self.url.parse().unwrap());
        } else { // Else treat url as a path to local file to display.
            window_url = WindowUrl::App(self.url.parse().unwrap());
        }
        
        match WindowBuilder::new(
            handle,
            &self.name,
            window_url
        )
        .always_on_top(true)
        .center()
        .decorations(false)
        .focused(true)
        .skip_taskbar(true)
        .visible(false)
        .build()
        {
            Ok(window) => {
                let scale: f64 = get_scale(&window);
                let size: PhysicalSize<u32> =
                    PhysicalSize::new((self.width * scale) as u32, (self.height * scale) as u32);

                if let Err(e) = window.set_size(size) {
                    error!("monarch_windows::build_window() failed! Failed to set new window size! | Err: {:?}", e);
                }
                if let Err(e) = window.center() {
                    error!("monarch_windows::build_window() failed! Failed to center new window! | Err: {:?}", e);
                }
                
                Ok(())
            }
            Err(e) => {
                return Err(anyhow!("monarch_windows::build_window() failed! Failed to build new window! | Err: {e}"))
            }
        }
    }

    /// Shows self
    pub fn show_window(&self, handle: &AppHandle) -> Result<()> {
        let window: Window = handle.get_window(&self.name).with_context(||
            -> String {format!("monarch_windows::show_window() failed! Failed to find window: {} | Err:", self.name)})?;

        return window.show().context(|| -> String {format!("monarch_windows::show_window() failed! Failed to show window: {} | Err:", self.name)}())
    }

    /// Hides self
    pub fn hide_window(&self, handle: &AppHandle) -> Result<()> {
        let window: Window = handle.get_window(&self.name).with_context(||
            -> String {format!("monarch_windows::hide_window() failed! Failed to find window: {} | Err:", self.name)})?;

        return window.hide().context(|| -> String {format!("monarch_windows::hide_window() failed! Failed to hide window: {} | Err:", self.name)}())
    }

    /// Close self
    pub fn close_window(&self, handle: &AppHandle) -> Result<()> {
        let window: Window = handle.get_window(&self.name).with_context(||
            -> String {format!("monarch_windows::close_window() failed! Failed to find window: {} | Err:", self.name)})?;

        return window.close().context(|| -> String {format!("monarch_windows::close_window() failed! Failed to close window: {} | Err:", self.name)}())
    }
}

// Returns scale to use based on monitor resolution
fn get_scale(window: &Window) -> f64 {
    if let Ok(monitor_option) = window.current_monitor() {
        match monitor_option {
            Some(monitor) => return monitor.size().height as f64 / STANDARD_HEIGHT,
            None => return 1.0
        }
    }
    1.0
}

/// Returns whether window of specific name exists.
pub fn window_exists(handle: &AppHandle, name: &str) -> bool {
    return handle.get_window(name).is_some()
}

/// Attempts to kill Quicklauch instance. Used on App exit to ensure quicklaunch instance doesn't run withouth Monarch.
pub fn kill_quicklaunch(handle: &AppHandle) -> Result<()> {
    let window: Window = handle.get_window("quicklaunch").with_context(||
        -> String {format!("monarch_windows::close_window() failed! Failed to find window: quicklaunch | Err:")})?;

    return window.close().context(|| -> String {format!("monarch_windows::close_window() failed! Failed to close window: quicklaunch | Err:")}())
}