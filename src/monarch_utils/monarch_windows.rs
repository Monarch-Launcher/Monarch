use anyhow::{Context, Result};
use tauri::{AppHandle, Manager, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tracing::error;

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
        let window_url: WebviewUrl = if self.url.starts_with("https") {
            WebviewUrl::External(self.url.parse().unwrap())
        } else {
            WebviewUrl::App(self.url.parse().unwrap())
        };

        let window: WebviewWindow = WebviewWindowBuilder::new(handle, &self.name, window_url)
            .always_on_top(false)
            .center()
            .decorations(true)
            .skip_taskbar(false)
            .visible(true)
            .title(&self.name)
            .build()
            .with_context(|| "monarch_windows::build_window() Failed to build window! | Err: ")?;

        let scale: f64 = get_scale(&window);
        let size: PhysicalSize<u32> =
            PhysicalSize::new((self.width * scale) as u32, (self.height * scale) as u32);

        if let Err(e) = window.set_size(size) {
            error!("monarch_windows::build_window() Failed to set new window size! | Err: {e}");
        }
        if let Err(e) = window.center() {
            error!("monarch_windows::build_window() Failed to center new window! | Err: {e}");
        }

        Ok(())
    }

    /// Shows a window with specified label
    pub fn show_window(&self, handle: &AppHandle) -> Result<()> {
        let window = handle.get_webview_window(&self.name).with_context(|| {
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
        let window = handle.get_webview_window(&self.name).with_context(|| {
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

    pub fn _close_window(&self, handle: &AppHandle) -> Result<()> {
        let window = handle.get_webview_window(&self.name).with_context(|| {
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

    pub fn set_quicklaunch_stuff(&self, handle: &AppHandle) -> Result<()> {
        let window = handle.get_webview_window(&self.name).with_context(|| {
            format!(
                "monarch_windows::show_window() Failed to find window: {} | Err:",
                self.name
            )
        })?;

        window.set_decorations(false)?;
        window.set_skip_taskbar(true)?;
        window.set_always_on_top(true)?;

        Ok(())
    }
}

// Returns scale to use based on monitor resolution
fn get_scale(window: &WebviewWindow) -> f64 {
    if let Ok(monitor_option) = window.current_monitor() {
        match monitor_option {
            Some(monitor) => return monitor.size().height as f64 / STANDARD_HEIGHT,
            None => return 1.0,
        }
    }
    1.0
}
