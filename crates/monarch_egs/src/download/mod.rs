mod manager;
mod manifest;

use std::path::PathBuf;

pub use manager::DownloadManager;
pub use manifest::{get_game_manifest};

pub enum Platform {
    Windows,
    Linux,
}

pub struct DownloadOptions {
    location: PathBuf,
    platform: Platform,
}

pub struct DownloadManifest {
    manifest_urls: Vec<String>,
    hash: String,
    base_urls: Vec<String>,
    manifest_data: Vec<u8>,
}