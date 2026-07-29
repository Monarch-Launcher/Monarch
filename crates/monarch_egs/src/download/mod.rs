mod manager;
mod manifest;

use std::{collections::HashMap, path::PathBuf};

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
    chunk_data_list: ChunkDataList,
    file_manifest_list: FileManifestList,
    custom_fields: CustomFields,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            header_magic: 0x44BEC00C,
            default_serialization_version: 17,
            header_size: 0,
            size_compressed: 0,
            size_uncompressed: 0,
            sha_hash: String::new(),
            stored_as: 0,
            version: 0,
            data: Vec::new(),
            meta: ManifestMetadata::default(),
            chunk_data_list: ChunkDataList::default(),
            file_manifest_list: FileManifestList::default(),
            custom_fields: CustomFields::default(),
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
    app_name: String,
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
            meta_size: 0,
            data_version: 0,
            feature_level: 18,
            is_file_data: false,
            app_id: 0,
            app_name: String::new(),
            build_version: String::new(),
            launch_exe: String::new(),
            launch_command: String::new(),
            prereq_ids: Vec::new(),
            prereq_name: String::new(),
            prereq_path: String::new(),
            prereq_args: String::new(),
            uninstall_action_path: String::new(),
            uninstall_action_args: String::new(),
            build_id: String::new(),
        }
    }
}

#[derive(Debug)]
pub struct ChunkDataList {
    version: u32,
    size: u32,
    count: u32,
    elements: Vec<ChunkInfo>,
    _manifest_version: u32,
    _guid_map: Option<HashMap<String, u32>>,
    _guid_int_map: Option<HashMap<u32, u32>>,
    _path_map: Option<HashMap<String, u32>>,
}

impl Default for ChunkDataList {
    fn default() -> Self {
        Self {
            version: 0,
            size: 0,
            count: 0,
            elements: Vec::new(),
            _manifest_version: 0,
            _guid_map: None,
            _guid_int_map: None,
            _path_map: None,
        }
    }
}

#[derive(Debug)]
pub struct ChunkInfo {
    guid: [u32; 4],
    hash: u64,
    sha_hash: [u8; 20],
    window_size: u32,
    file_size: u64,
    _manifest_version: u32,
    _group_num: Option<u32>,
    _guid_str: Option<String>,
    _guid_num: Option<u32>
}

impl Default for ChunkInfo {
    fn default() -> Self {
        Self {
            guid: [0; 4],
            hash: 0,
            sha_hash: [0; 20],
            window_size: 0,
            file_size: 0,
            _manifest_version: 0,
            _group_num: None,
            _guid_str: None,
            _guid_num: None,
        }
    }
}

#[derive(Debug)]
pub struct FileManifestList {
    version: u32,
    size: u32,
    count: u32,
    elements: Vec<FileManifest>,
    _path_map: Option<HashMap<String, u32>>,
}

impl Default for FileManifestList {
    fn default() -> Self {
        Self {
            version: 0,
            size: 0,
            count: 0,
            elements: Vec::new(),
            _path_map: None
        }
    }
}

#[derive(Debug)]
pub struct FileManifest {
    filename: String,
    symlink_target: String,
    hash: [u8; 20],
    flags: u32,
    install_tags: Vec<String>,
    chunk_parts: Vec<ChunkPart>,
    file_size: u64,
    hash_md5: [u8; 16],
    mime_type: String,
    hash_sha256: [u8; 32],
}

impl Default for FileManifest {
    fn default() -> Self {
        Self {
            filename: String::new(),
            symlink_target: String::new(),
            hash: [0; 20],
            flags: 0,
            install_tags: Vec::new(),
            chunk_parts: Vec::new(),
            file_size: 0,
            hash_md5: [0; 16],
            mime_type: String::new(),
            hash_sha256: [0; 32],
        }
    }
}

#[derive(Debug)]
pub struct ChunkPart {
    guid: [u32; 4],
    offset: u32,
    size: u32,
    file_offset: u32,
    _guid_str: Option<String>,
    _guid_num: Option<u32>
}

impl Default for ChunkPart {
    fn default() -> Self {
        Self {
            guid: [0; 4],
            offset: 0,
            size: 0,
            file_offset: 0,
            _guid_str: None,
            _guid_num: None,
        }
    }
}   

#[derive(Debug)]
pub struct CustomFields {
    size: u32,
    version: u32,
    count: u32,
    _dict: HashMap<String, String>,
}

impl Default for CustomFields {
    fn default() -> Self {
        Self {
            size: 0,
            version: 0,
            count: 0,
            _dict: HashMap::new(),
        }
    }
}