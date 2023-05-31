pub mod monarchgame;
pub mod commands;

#[cfg(target_os="windows")]
pub mod windows;

#[cfg(target_os="linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod linux;