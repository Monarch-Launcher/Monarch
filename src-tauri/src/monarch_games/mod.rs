pub mod commands;
pub mod monarch_client;
pub mod monarchgame;
pub mod steam_client;
pub mod stores;
pub mod games;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;
