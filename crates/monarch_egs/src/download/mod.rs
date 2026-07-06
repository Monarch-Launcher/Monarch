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

pub struct PrepManifestData {
    manifest_urls: Vec<String>,
    hash: String,
    base_urls: Vec<String>,
    manifest_data: Vec<u8>,
}

#[derive(Debug)]
pub struct Manifest {
    header_magic: u32,
    default_serialization_version: u32,
    header_size: u32,
    size_compressed: u32,
    size_uncompressed: u32,
    sha_hash: String,
    stored_as: u32,
    version: u32,
    data: Vec<u8>,

    meta: ManifestMetadata,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            header_magic: 0x44BEC00C,
            default_serialization_version: 17,
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub struct ManifestMetadata {
    meta_size: u32,
    data_version: u32,
    feature_level: u32,
    is_file_data: bool,
    app_id: u32,
    build_version: String,
    launch_exe: String,
    launch_command: String,
    prereq_ids: Vec<String>,
    prereq_name: String,
    prereq_path: String,
    prereq_args: String,
    uninstall_action_path: String,
    uninstall_action_args: String,
    build_id: String,
}

impl Default for ManifestMetadata {
    fn default() -> Self {
        Self {
            feature_level: 18,
            ..Default::default()
        }
    }
}