use std::collections::HashMap;

pub mod commands;
pub mod egs_client;
pub mod games;
pub mod integrity;
pub mod monarch_client;
pub mod monarchgame;
pub mod steam_client;
pub mod stores;
pub mod updates;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

pub struct LaunchCommand {
    command: String,
    work_dir: String,
    env: HashMap<String, String>,
}
