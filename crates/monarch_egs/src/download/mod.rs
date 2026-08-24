mod chunk;
mod manager;
mod manifest;

use std::collections::HashMap;

pub use manager::{
    DownloadEvent, DownloadManager, DownloadPlan, DownloadProgress, DownloadReport,
    DownloaderOptions, VerifyReport, VerifyStatus,
};

pub use manifest::get_game_manifest;

/// Raw data handed between manifest retrieval and parsing.
#[derive(Debug, Clone)]
pub struct PrepManifestData {
    pub manifest_urls: Vec<String>,
    pub hash: String,
    pub base_urls: Vec<String>,
    pub manifest_data: Vec<u8>,
    pub secret_keys: HashMap<String, String>,
}

/// A parsed Epic Games build manifest. Contains everything required to
/// download, write and verify a game installation.
#[derive(Debug, Clone)]
pub struct Manifest {
    header_magic: u32,
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

    /// CDN directories the manifests were fetched from. Chunks are fetched
    /// from these same origins.
    base_urls: Vec<String>,
    /// Full manifest URLs, kept for reference / re-downloading.
    manifest_urls: Vec<String>,
    /// AES keys for encrypted chunks, keyed by the uppercase hex
    /// representation of the chunk's `secret_guid`.
    secret_keys: HashMap<String, String>,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            header_magic: 0x44BEC00C,
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
            base_urls: Vec::new(),
            manifest_urls: Vec::new(),
            secret_keys: HashMap::new(),
        }
    }
}

impl Manifest {
    /// CDN base URLs for this build.
    pub fn base_urls(&self) -> &[String] {
        &self.base_urls
    }

    /// Total uncompressed size of all files in this build, in bytes.
    pub fn install_size(&self) -> u64 {
        self.file_manifest_list
            .elements
            .iter()
            .map(|f| f.file_size)
            .sum()
    }

    /// Feature level (build patch version) of the manifest.
    pub fn feature_level(&self) -> u32 {
        self.meta.feature_level
    }

    /// Application identifier, e.g. `Fortnite`.
    pub fn app_name(&self) -> &str {
        &self.meta.app_name
    }

    /// Version string of this build.
    pub fn build_version(&self) -> &str {
        &self.meta.build_version
    }

    /// Executable relative to the install directory, if any.
    pub fn launch_exe(&self) -> &str {
        &self.meta.launch_exe
    }

    /// Extra arguments passed to the launch executable, if any.
    pub fn launch_command(&self) -> &str {
        &self.meta.launch_command
    }

    pub(crate) fn chunks(&self) -> &[ChunkInfo] {
        &self.chunk_data_list.elements
    }

    pub(crate) fn files(&self) -> &[FileManifest] {
        &self.file_manifest_list.elements
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct ChunkDataList {
    version: u32,
    size: u32,
    count: u32,
    elements: Vec<ChunkInfo>,
    _manifest_version: u32,
    _guid_map: Option<HashMap<String, u32>>,
    _guid_int_map: Option<HashMap<u128, u32>>,
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

#[derive(Debug, Clone)]
pub struct ChunkInfo {
    guid: [u32; 4],
    hash: u64,
    sha_hash: [u8; 20],
    window_size: u32,
    file_size: u64,
    _manifest_version: u32,
    _group_num: Option<u32>,
    /// Only populated for feature level >= 22 manifests.
    secret_guid: [u32; 4],
    /// Only populated for feature level >= 22 manifests.
    encryption_tag: [u8; 16],
    /// AES-256 key for this chunk if the build is encrypted.
    secret_key: Option<[u8; 32]>,
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
            secret_guid: [0; 4],
            encryption_tag: [0; 16],
            secret_key: None,
        }
    }
}

impl ChunkInfo {
    /// 128-bit numeric representation of the chunk GUID, usable as a map key.
    pub fn guid_num(&self) -> u128 {
        (self.guid[3] as u128)
            | (self.guid[2] as u128) << 32
            | (self.guid[1] as u128) << 64
            | (self.guid[0] as u128) << 96
    }

    /// Group number, part of the CDN path. Falls back to CRC32(guid) % 100.
    pub fn group_num(&self) -> u32 {
        if let Some(group) = self._group_num {
            return group;
        }
        let mut bytes = [0u8; 16];
        bytes[0..4].copy_from_slice(&self.guid[0].to_le_bytes());
        bytes[4..8].copy_from_slice(&self.guid[1].to_le_bytes());
        bytes[8..12].copy_from_slice(&self.guid[2].to_le_bytes());
        bytes[12..16].copy_from_slice(&self.guid[3].to_le_bytes());
        crc32fast::hash(&bytes) % 100
    }

    /// Decompressed window size of this chunk.
    pub fn window_size(&self) -> u32 {
        self.window_size
    }

    /// Compressed size as reported by the manifest (download size).
    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    pub(crate) fn sha_hash(&self) -> &[u8; 20] {
        &self.sha_hash
    }

    pub(crate) fn encryption_tag(&self) -> &[u8; 16] {
        &self.encryption_tag
    }

    pub(crate) fn secret_key(&self) -> Option<&[u8; 32]> {
        self.secret_key.as_ref()
    }

    /// Relative CDN path of this chunk, e.g.
    /// `ChunksV4/05/8A3E6D2FAB1C90DE1234567890ABCDEF....chunk`.
    pub fn path(&self, feature_level: u32) -> String {
        if feature_level >= 22 {
            let secret_part = if self.secret_guid == [0; 4] {
                "plain".to_string()
            } else {
                base64_url_no_pad(&le_bytes(&self.secret_guid))
            };
            let hash_b64 = base64_url_no_pad(&self.hash.to_le_bytes());
            let guid_b64 = base64_url_no_pad(&le_bytes(&self.guid));
            format!(
                "{}/{}/{:02}/{}_{}.chunk",
                chunk_dir(feature_level),
                secret_part,
                self.group_num(),
                hash_b64,
                guid_b64
            )
        } else {
            format!(
                "{}/{:02}/{:016X}_{}.chunk",
                chunk_dir(feature_level),
                self.group_num(),
                self.hash,
                guid_hex_upper(&self.guid)
            )
        }
    }
}

#[derive(Debug, Clone)]
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
            _path_map: None,
        }
    }
}

#[derive(Debug, Clone)]
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

impl FileManifest {
    /// Path of this file relative to the install directory.
    pub fn filename(&self) -> &str {
        &self.filename
    }

    /// SHA1 hash of the installed file, used for verification.
    pub fn sha1(&self) -> &[u8; 20] {
        &self.hash
    }

    /// Size of the installed file in bytes.
    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    /// Whether the file is marked as an executable in the manifest.
    pub fn is_executable(&self) -> bool {
        self.flags & 0x4 != 0
    }

    pub(crate) fn chunk_parts(&self) -> &[ChunkPart] {
        &self.chunk_parts
    }
}

#[derive(Debug, Clone)]
pub struct ChunkPart {
    guid: [u32; 4],
    offset: u32,
    size: u32,
    file_offset: u32,
}

impl Default for ChunkPart {
    fn default() -> Self {
        Self {
            guid: [0; 4],
            offset: 0,
            size: 0,
            file_offset: 0,
        }
    }
}

impl ChunkPart {
    /// Offset within the decompressed chunk window.
    pub fn offset(&self) -> u32 {
        self.offset
    }

    /// Number of bytes this part contributes to the file.
    pub fn size(&self) -> u32 {
        self.size
    }

    /// Offset within the assembled file.
    pub fn file_offset(&self) -> u32 {
        self.file_offset
    }

    pub(crate) fn guid_num(&self) -> u128 {
        (self.guid[3] as u128)
            | (self.guid[2] as u128) << 32
            | (self.guid[1] as u128) << 64
            | (self.guid[0] as u128) << 96
    }
}

#[derive(Debug, Clone)]
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

/// Chunk sub-directory used by a given manifest feature level.
pub(crate) fn chunk_dir(feature_level: u32) -> &'static str {
    if feature_level >= 22 {
        "ChunksV5"
    } else if feature_level >= 15 {
        "ChunksV4"
    } else if feature_level >= 6 {
        "ChunksV3"
    } else if feature_level >= 3 {
        "ChunksV2"
    } else {
        "Chunks"
    }
}

/// Uppercase hex of a GUID (32 chars), as used in chunk file names.
pub(crate) fn guid_hex_upper(guid: &[u32; 4]) -> String {
    guid.iter()
        .map(|word| format!("{word:08X}"))
        .collect::<String>()
}

fn le_bytes(guid: &[u32; 4]) -> Vec<u8> {
    let mut out = Vec::with_capacity(16);
    for word in guid {
        out.extend_from_slice(&word.to_le_bytes());
    }
    out
}

fn base64_url_no_pad(data: &[u8]) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    URL_SAFE_NO_PAD.encode(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_chunk() -> ChunkInfo {
        let mut chunk = ChunkInfo::default();
        chunk.guid = [0xDEADBEEF, 0x01020304, 0xA0B0C0D0, 0x11223344];
        chunk.hash = 0x1122334455667788;
        chunk.secret_guid = [1, 2, 3, 4];
        chunk
    }

    #[test]
    fn chunk_dir_matches_feature_levels() {
        assert_eq!(chunk_dir(0), "Chunks");
        assert_eq!(chunk_dir(3), "ChunksV2");
        assert_eq!(chunk_dir(6), "ChunksV3");
        assert_eq!(chunk_dir(15), "ChunksV4");
        assert_eq!(chunk_dir(22), "ChunksV5");
        assert_eq!(chunk_dir(100), "ChunksV5");
    }

    #[test]
    fn guid_hex_upper_is_uppercase_concat() {
        assert_eq!(
            guid_hex_upper(&[0xDEADBEEF, 0x00000001, 0x00000000, 0xAABBCCDD]),
            "DEADBEEF0000000100000000AABBCCDD"
        );
    }

    #[test]
    fn legacy_chunk_path_uses_v4_layout() {
        let chunk = sample_chunk();
        let path = chunk.path(15);
        assert!(path.starts_with("ChunksV4/"));
        let parts: Vec<&str> = path.split('/').collect();
        assert_eq!(parts.len(), 3);
        // group number, hash in upper hex, guid in upper hex.
        assert_eq!(parts[1], format!("{:02}", chunk.group_num()));
        assert_eq!(parts[2], format!("{:016X}_{}.chunk", chunk.hash, guid_hex_upper(&chunk.guid)));
    }

    #[test]
    fn v5_chunk_path_uses_secret_and_base64url() {
        let chunk = sample_chunk();
        let path = chunk.path(22);
        assert!(path.starts_with("ChunksV5/"));
        let parts: Vec<&str> = path.split('/').collect();
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[1], base64_url_no_pad(&le_bytes(&chunk.secret_guid)));
        assert_eq!(parts[2], format!("{:02}", chunk.group_num()));
        let name = parts[3];
        let hash_b64 = base64_url_no_pad(&chunk.hash.to_le_bytes());
        let guid_b64 = base64_url_no_pad(&le_bytes(&chunk.guid));
        assert_eq!(name, format!("{hash_b64}_{guid_b64}.chunk"));
    }

    #[test]
    fn zero_secret_guid_is_plain() {
        let mut chunk = sample_chunk();
        chunk.secret_guid = [0; 4];
        let path = chunk.path(22);
        let parts: Vec<&str> = path.split('/').collect();
        assert_eq!(parts[1], "plain");
    }

    #[test]
    fn group_num_crc32_fallback_is_stable() {
        let chunk = sample_chunk();
        assert!(chunk.group_num() < 100);
        // Deterministic for the same guid.
        assert_eq!(chunk.group_num(), sample_chunk().group_num());
    }

    #[test]
    fn guid_num_packs_words_into_u128() {
        let mut chunk = ChunkInfo::default();
        chunk.guid = [1, 2, 3, 4];
        let expected =
            (4u128) | (3u128 << 32) | (2u128 << 64) | (1u128 << 96);
        assert_eq!(chunk.guid_num(), expected);
    }

    #[test]
    fn le_bytes_little_endian_words() {
        assert_eq!(le_bytes(&[0x00000001, 0, 0, 0]), vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn base64_url_encoding_is_unpadded() {
        assert_eq!(base64_url_no_pad(b"Epic Games"), "RXBpYyBHYW1lcw");
        assert!(!base64_url_no_pad(b"Epic Games").contains('='));
    }
}
