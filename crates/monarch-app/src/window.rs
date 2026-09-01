//! Windows-only window helpers.

/// Rounds the corners of the top-level window using the Desktop Window Manager.
///
/// On Windows 11 this makes the undecorated window's corners actually rounded
/// by the OS (the pixels outside the corner arc are cut away). On Windows 10
/// the call is a no-op, so the window simply keeps square corners.
#[cfg(target_os = "windows")]
pub fn apply_rounded_corners() {
    std::thread::spawn(|| {
        use windows_sys::Win32::Graphics::Dwm::{
            DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
        };
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
                let round: i32 = DWMWCP_ROUND;
                unsafe {
                    DwmSetWindowAttribute(
                        hwnd,
                        DWMWA_WINDOW_CORNER_PREFERENCE as u32,
                        &round as *const i32 as *const _,
                        std::mem::size_of::<i32>() as u32,
                    );
                }
                return;
            }

            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    });
}