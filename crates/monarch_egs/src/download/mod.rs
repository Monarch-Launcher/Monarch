mod manager;
mod manifest;

use std::path::PathBuf;

pub use manager::DownloadManager;
pub use manifest::{Manifest, get_game_manifest};

pub enum Platform {
    Windows,
    Linux,
}

pub struct DownloadOptions {
    location: PathBuf,
    platform: Platform,
}
