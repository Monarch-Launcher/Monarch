pub mod commands;
pub mod monarch_client;
pub mod monarchgame;
pub mod steam_client;

#[cfg(target_os = "windows")]
pub mod windows;

//#[cfg(target_os = "linux")]
//pub mod linux;

#[cfg(target_os = "linux")]
pub mod linux;
