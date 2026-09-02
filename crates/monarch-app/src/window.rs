//! Windows-only window helpers.

#[cfg(target_os = "windows")]
use std::sync::atomic::{AtomicIsize, Ordering};

/// Stored HWND of the main window, set once at startup.
#[cfg(target_os = "windows")]
static MAIN_HWN: AtomicIsize = AtomicIsize::new(0);

/// Rounds the corners of the top-level window using the Desktop Window Manager.
///
/// On Windows 11 this makes the undecorated window's corners actually rounded
/// by the OS (the pixels outside the corner arc are cut away). On Windows 10
/// the call is a no-op, so the window simply keeps square corners.
#[cfg(target_os = "windows")]
pub fn apply_rounded_corners() {
    std::thread::spawn(|| {
        use windows_sys::Win32::System::Threading::GetCurrentProcessId;
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetForegroundWindow, GetWindowThreadProcessId,
        };

        let process_id = unsafe { GetCurrentProcessId() };

        // The main window is opened by the iced daemon shortly after startup,
        // so poll until this process' foreground window appears, then round it.
        for _ in 0..100 {
            let hwnd = unsafe { GetForegroundWindow() };
            if hwnd == 0 {
                std::thread::sleep(std::time::Duration::from_millis(200));
                continue;
            }

            let mut window_process_id: u32 = 0;
            unsafe { GetWindowThreadProcessId(hwnd, &mut window_process_id) };

            if window_process_id == process_id {
                MAIN_HWN.store(hwnd as isize, Ordering::Relaxed);
                set_rounded_corners(true);
                return;
            }

            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    });
}

/// Enable or disable DWM rounded corners on the main window.
///
/// Must be called after [`apply_rounded_corners`] has stored the HWND.
/// On Windows 10 this is a no-op; on Windows 11 it toggles between
/// `DWMWCP_ROUND` and `DWMWCP_DONOTROUND`.
#[cfg(target_os = "windows")]
pub fn set_rounded_corners(enable: bool) {
    use windows_sys::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND, DWMWCP_ROUND,
    };

    let hwnd = MAIN_HWN.load(Ordering::Relaxed) as isize;
    if hwnd == 0 {
        return;
    }

    let preference: i32 = if enable {
        DWMWCP_ROUND
    } else {
        DWMWCP_DONOTROUND
    };
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE as u32,
            &preference as *const i32 as *const _,
            std::mem::size_of::<i32>() as u32,
        );
    }
}

#[cfg(not(target_os = "windows"))]
pub fn apply_rounded_corners() {}

#[cfg(not(target_os = "windows"))]
pub fn set_rounded_corners(_enable: bool) {}
