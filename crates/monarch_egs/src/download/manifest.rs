use std::io::prelude::*;
use std::collections::HashMap;
use flate2::read::ZlibDecoder;

use reqwest::Client;
use sha1::{Sha1, Digest};

use crate::{download::{ChunkDataList, ChunkInfo, ChunkPart, CustomFields, FileManifest, FileManifestList, ManifestMetadata}, utils::err::MonarchEgsError};
use tracing::{debug, info, trace, warn};
use super::{Manifest, PrepManifestData};

static CDN_URL: &str = "launcher-public-service-prod06.ol.epicgames.com";

/// Returns a download manifest for Epic Games game of namespace
pub async fn get_game_manifest(
    access_token: &str,
    platform: &str,
    namespace: &str,
    catalog_id: &str,
    app_name: &str,
) -> Result<Manifest, MonarchEgsError> {
    let (manifest_urls, base_urls, hash, secret_keys) =
        get_cdn_urls(access_token, platform, namespace, catalog_id, app_name).await.unwrap();

    let client: Client = Client::new();

    let mut manifest_info: Option<PrepManifestData> = None;

    for url in manifest_urls.iter() {
        info!("Attempting download of {}", url);

        let response = client.get(url).send().await.unwrap();

        if response.status().is_success() {
            let manifest_data: Vec<u8> = response.bytes().await.unwrap().to_vec();

            let computed_hash = Sha1::digest(&manifest_data)
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>();

            if computed_hash != hash {
                return Err(MonarchEgsError::HashMismatchError(format!("Hash mismatch for manifest! | Computed: {:?}, Expected: {}", computed_hash, hash)));
            }

            debug!("Hash checked out!");
            manifest_info = Some(PrepManifestData {
                manifest_urls,
                base_urls,
                hash,
                manifest_data,
                secret_keys,
            });
            break;
        }
    }

    
    if let Some(m) = manifest_info {
        let manifest: Manifest = parse_manifest(&m).await.unwrap();   
        return Ok(manifest);
    }
    
    Err(MonarchEgsError::WebRequestError(format!("All manifest downloads failed!")))
}

// Returns a list of URLs to the manifests
async fn get_cdn_urls(
    access_token: &str,
    platform: &str,
    namespace: &str,
    catalog_id: &str,
    app_name: &str,
) -> Result<(Vec<String>, Vec<String>, String, HashMap<String, String>), MonarchEgsError> {
    let url: String = format!(
        "https://{CDN_URL}/launcher/api/public/assets/v2/platform/{platform}/namespace/{namespace}/catalogItem/{catalog_id}/app/{app_name}/label/Live",
    );

    let client: Client = Client::new();
    let response = client.get(&url).bearer_auth(access_token).send().await.unwrap();
    let response_object: serde_json::Value = response.json().await.unwrap();

    let first_element = match response_object.get("elements") {
        Some(elements) => match elements.get(0) {
            Some(first) => first,
            None => {
                return Err(MonarchEgsError::ParsingError(
                    "'elements' missing index 0".to_string(),
                ));
            }
        },
        None => {
            return Err(MonarchEgsError::ParsingError(
                "Missing 'elements' attribute".to_string(),
            ));
        }
    };

    // Get manifest hash
    let hash: String = match first_element.get("hash") {
        Some(hash) => hash.as_str().unwrap_or_default().to_string(),
        None => {
            return Err(MonarchEgsError::ParsingError(
                "Missing 'hash' attribute".to_string(),
            ));
        }
    };

    // AES keys for encrypted builds, keyed by uppercase hex GUID
    let mut secret_keys: HashMap<String, String> = HashMap::new();
    if let Some(secrets) = first_element.get("secrets").and_then(|s| s.as_object()) {
        for (guid, key) in secrets {
            if let Some(key) = key.as_str() {
                secret_keys.insert(guid.to_uppercase(), key.to_string());
            }
        }
    }

    // Get manifest URLs
    let manifests: Vec<serde_json::Value> = match first_element.get("manifests") {
        Some(manifests) => manifests.as_array().unwrap().to_vec(),
        None => {
            return Err(MonarchEgsError::ParsingError(
                "Missing 'manifests' attribute".to_string(),
            ));
        }
    };

    let mut manifest_urls: Vec<String> = Vec::new();
    let mut base_urls: Vec<String> = Vec::new();
    for manifest in manifests.iter() {
        let mut url: String = manifest.get("uri").unwrap().to_string().replace("\"", "");
        let url_parts: Vec<&str> = url.split('/').collect();
        let base_url: String = url_parts.clone().into_iter().take(url_parts.len() - 1).collect::<Vec<&str>>().join("/");

        if let Some(query_params) = manifest.get("queryParams") {
            let params: String = query_params.as_array()
                .unwrap()
                .iter()
                .map(|value| {
                    let k = value.get("name").unwrap_or_default().to_string();
                    let v = value.get("value").unwrap_or_default().to_string();

                    if k.is_empty() || v.is_empty() {
                        return String::new();
                    }
                    format!("&{}={}", k.replace("\"", ""), v.replace("\"", ""))
                }).collect::<String>();

            url.push_str(&format!("?{}", &params));
        }

        manifest_urls.push(url);
        base_urls.push(base_url);
    }

    debug!("manifest_urls: {:?}", manifest_urls);
    debug!("base_urls: {:?}", base_urls);

    Ok((manifest_urls, base_urls, hash, secret_keys))
}

// Parses a manifest into a Manifest struct
pub async fn parse_manifest(manifest_info: &PrepManifestData) -> Result<Manifest, MonarchEgsError> {
    if manifest_info.manifest_data.starts_with(b"{") {
        return parse_json_manifest(&manifest_info).await;
    }
    return parse_binary_manifest(&manifest_info).await;
}

// Parse the manifest as a JSON object
async fn parse_json_manifest(manifest_info: &PrepManifestData) -> Result<Manifest, MonarchEgsError> {
    let manifest_json: serde_json::Value = serde_json::from_slice(&manifest_info.manifest_data).unwrap();
    trace!("manifest_json: {:?}", manifest_json);

    let mut manifest = Manifest::default();
    manifest.base_urls = manifest_info.base_urls.clone();
    manifest.manifest_urls = manifest_info.manifest_urls.clone();
    manifest.secret_keys = manifest_info.secret_keys.clone();
    manifest.version = manifest_json
        .get("ManifestFileVersion")
        .and_then(|v| v.as_str())
        .and_then(parse_blob_num)
        .unwrap_or(13);
    manifest.meta.feature_level = manifest.version;
    manifest.meta.app_name = manifest_json
        .get("AppNameString")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    manifest.meta.build_version = manifest_json
        .get("BuildVersionString")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    manifest.meta.launch_exe = manifest_json
        .get("LaunchExeString")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    manifest.meta.launch_command = manifest_json
        .get("LaunchCommand")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    manifest.meta.prereq_ids = manifest_json
        .get("PrereqIds")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Ok(manifest)
}

/// Decodes the JSON manifest's `blob_to_num` format: each byte is stored as a
/// 3-digit decimal string, e.g. `018000000000` -> 18.
fn parse_blob_num(value: &str) -> Option<u32> {
    let mut num: u64 = 0;
    let mut shift = 0;
    for chunk in value.as_bytes().chunks(3) {
        let digit = std::str::from_utf8(chunk).ok()?.parse::<u64>().ok()?;
        num |= digit << shift;
        shift += 8;
    }
    u32::try_from(num).ok()
}

// Parse the manifest as a binary object
async fn parse_binary_manifest(manifest_info: &PrepManifestData) -> Result<Manifest, MonarchEgsError> {
    debug!("manifest is binary!");

    let mut manifest: Manifest = Manifest::default();
    
    let mut manifest_data = manifest_info.manifest_data.as_slice();
    let mut buf: [u8; 4] = [0; 4];
    manifest_data.read_exact(&mut buf).expect("Failed to read manifest magic into buf!");

    let magic: u32 = u32::from_le_bytes(buf);
    if magic != manifest.header_magic {
        return Err(MonarchEgsError::ParsingError(format!("Invalid manifest magic! | Expected: 0x{:x}, Got: 0x{:x}", manifest.header_magic, magic)));
    }

    manifest_data.read_exact(&mut buf).expect("Failed to read header_size!");
    manifest.header_size = u32::from_le_bytes(buf);

    manifest_data.read_exact(&mut buf).expect("Failed to read size_uncompressed!");
    manifest.size_uncompressed = u32::from_le_bytes(buf);

    manifest_data.read_exact(&mut buf).expect("Failed to read size_compressed!");
    manifest.size_compressed = u32::from_le_bytes(buf);

    let mut sha_buf: [u8; 20] = [0; 20];
    manifest_data.read_exact(&mut sha_buf).expect("Failed to read sha_hash!");
    manifest.sha_hash = sha_buf
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();

    let mut stored_buf: [u8; 1] = [0; 1];
    manifest_data.read_exact(&mut stored_buf).expect("Failed to read stored_as!");
    manifest.stored_as = stored_buf[0] as u32;

    manifest_data.read_exact(&mut buf).expect("Failed to read version!");
    manifest.version = u32::from_le_bytes(buf);

    // Size is known ahead of time, might as well allocate the memory upfront
    manifest.data = Vec::with_capacity(manifest.size_uncompressed as usize);

    // Is compressed
    if manifest.stored_as == 1 {
        let mut data: Vec<u8> = Vec::with_capacity(manifest.size_compressed as usize);
        manifest_data.read_to_end(&mut data).expect("Failed to read manifest data!");

        let mut decoder = ZlibDecoder::new(&data[..]);
        decoder.read_to_end(&mut manifest.data).expect("Failed to read uncompressed manifest data!");

        let dec_hash = Sha1::digest(&manifest.data)
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        if dec_hash != manifest.sha_hash {
            return Err(MonarchEgsError::HashMismatchError(format!("Hash mismatch for decompressed manifest data! | Computed: {:?}, Expected: {}", dec_hash, manifest.sha_hash)));
        }
        
    // Not compressed
    } else {
        manifest_data.read_to_end(&mut manifest.data).expect("Failed to read manifest data!");
    }
    debug!("Initial manifest parsing done!");

    let mut data = manifest.data.as_slice();

    // Parse manifest metadata from the decompressed payload
    manifest.meta = parse_manifest_meta(&mut data).await.unwrap();
    debug!("Manifest metadata parsing done!");

    manifest.base_urls = manifest_info.base_urls.clone();
    manifest.manifest_urls = manifest_info.manifest_urls.clone();
    manifest.secret_keys = manifest_info.secret_keys.clone();

    // Parse chunk data list from the decompressed payload
    manifest.chunk_data_list = parse_chunk_data_list(&mut data, manifest.meta.feature_level, &manifest.secret_keys).await.unwrap();
    debug!("Chunk data list parsing done!");

    // Parse file manifest list from the decompressed payload
    manifest.file_manifest_list = parse_file_manifest_list(&mut data).await.unwrap();
    debug!("File manifest list parsing done!");

    // Parse file manifest list from the decompressed payload
    manifest.custom_fields = parse_custom_fields(&mut data).await.unwrap();
    debug!("Custom fields parsing done!");

    Ok(manifest)
}

// Parse the manifest metadata from the decompressed payload
async fn parse_manifest_meta(data: &mut &[u8]) -> Result<ManifestMetadata, MonarchEgsError> {
    let mut manifest_meta: ManifestMetadata = ManifestMetadata::default();
    let start_remaining = data.len();

    let mut buf: [u8; 4] = [0; 4];
    data.read_exact(&mut buf).expect("Failed to read meta_size!");
    manifest_meta.meta_size = u32::from_le_bytes(buf);

    let mut byte_buf: [u8; 1] = [0; 1];
    data.read_exact(&mut byte_buf).expect("Failed to read data_version!");
    manifest_meta.data_version = byte_buf[0] as u32;

    data.read_exact(&mut buf).expect("Failed to read feature_level!");
    manifest_meta.feature_level = u32::from_le_bytes(buf);

    data.read_exact(&mut byte_buf).expect("Failed to read is_file_data!");
    manifest_meta.is_file_data = byte_buf[0] == 1;

    data.read_exact(&mut buf).expect("Failed to read app_id!");
    manifest_meta.app_id = u32::from_le_bytes(buf);

    manifest_meta.app_name = read_fstring(data);
    manifest_meta.build_version = read_fstring(data);
    manifest_meta.launch_exe = read_fstring(data);
    manifest_meta.launch_command = read_fstring(data);

    data.read_exact(&mut buf).expect("Failed to read prereq_ids count!");
    let entries = u32::from_le_bytes(buf);
    for _ in 0..entries {
        manifest_meta.prereq_ids.push(read_fstring(data));
    }

    manifest_meta.prereq_name = read_fstring(data);
    manifest_meta.prereq_path = read_fstring(data);
    manifest_meta.prereq_args = read_fstring(data);

    if manifest_meta.data_version >= 1 {
        manifest_meta.build_id = read_fstring(data);
    }

    if manifest_meta.data_version >= 2 {
        manifest_meta.uninstall_action_path = read_fstring(data);
        manifest_meta.uninstall_action_args = read_fstring(data);
    }

    let size_read = (start_remaining - data.len()) as u32;
    if size_read != manifest_meta.meta_size {
        let missing = manifest_meta.meta_size as i64 - size_read as i64;
        warn!(
            "Did not read entire manifest metadata! Version: {}, {} bytes missing, skipping...",
            manifest_meta.data_version, missing
        );
        if missing > 0 {
            let mut skip = vec![0u8; missing as usize];
            data.read_exact(&mut skip).expect("Failed to skip remaining meta bytes!");
        }
        // downgrade version to prevent issues during serialisation
        manifest_meta.data_version = 0;
    }

    Ok(manifest_meta)
}

// Parse the chunk data list from the decompressed payload
async fn parse_chunk_data_list(
    data: &mut &[u8],
    feature_level: u32,
    secret_keys: &HashMap<String, String>,
) -> Result<ChunkDataList, MonarchEgsError> {
    let mut chunk_data_list: ChunkDataList = ChunkDataList::default();
    let start_remaining = data.len();

    chunk_data_list._manifest_version = feature_level;

    let mut buf: [u8; 4] = [0; 4];
    data.read_exact(&mut buf).expect("Failed to read chunk data list size!");
    chunk_data_list.size = u32::from_le_bytes(buf);

    let mut byte_buf: [u8; 1] = [0; 1];
    data.read_exact(&mut byte_buf).expect("Failed to read chunk data list version!");
    chunk_data_list.version = byte_buf[0] as u32;

    data.read_exact(&mut buf).expect("Failed to read chunk data list count!");
    chunk_data_list.count = u32::from_le_bytes(buf);

    // the way this data is stored is rather odd, maybe there's a nicer way to write this...
    for _ in 0..chunk_data_list.count {
        let mut chunk = ChunkInfo::default();
        chunk._manifest_version = feature_level;
        chunk_data_list.elements.push(chunk);
    }

    // guid, doesn't seem to be a standard like UUID but is fairly straightfoward, 4 bytes, 128 bit.
    let mut guid_buf: [u8; 16] = [0; 16];
    for chunk in chunk_data_list.elements.iter_mut() {
        data.read_exact(&mut guid_buf).expect("Failed to read chunk guid!");
        chunk.guid = [
            u32::from_le_bytes(guid_buf[0..4].try_into().unwrap()),
            u32::from_le_bytes(guid_buf[4..8].try_into().unwrap()),
            u32::from_le_bytes(guid_buf[8..12].try_into().unwrap()),
            u32::from_le_bytes(guid_buf[12..16].try_into().unwrap()),
        ];
    }

    // hash is a 64 bit integer, no idea how it's calculated but we don't need to know that.
    let mut hash_buf: [u8; 8] = [0; 8];
    for chunk in chunk_data_list.elements.iter_mut() {
        data.read_exact(&mut hash_buf).expect("Failed to read chunk hash!");
        chunk.hash = u64::from_le_bytes(hash_buf);
    }

    // sha1 hash
    let mut sha_buf: [u8; 20] = [0; 20];
    for chunk in chunk_data_list.elements.iter_mut() {
        data.read_exact(&mut sha_buf).expect("Failed to read chunk sha_hash!");
        chunk.sha_hash = sha_buf;
    }

    // group number, seems to be part of the download path
    for chunk in chunk_data_list.elements.iter_mut() {
        data.read_exact(&mut byte_buf).expect("Failed to read chunk group_num!");
        chunk._group_num = Some(byte_buf[0] as u32);
    }

    // window size is the uncompressed size
    for chunk in chunk_data_list.elements.iter_mut() {
        data.read_exact(&mut buf).expect("Failed to read chunk window_size!");
        chunk.window_size = u32::from_le_bytes(buf);
    }

    // file size is the compressed size that will need to be downloaded
    let mut file_size_buf: [u8; 8] = [0; 8];
    for chunk in chunk_data_list.elements.iter_mut() {
        data.read_exact(&mut file_size_buf).expect("Failed to read chunk file_size!");
        chunk.file_size = i64::from_le_bytes(file_size_buf) as u64;
    }

    // ChunksV5 era manifests store per-chunk encryption data (feature level >= 22)
    if feature_level >= 22 {
        let mut secret_guid_buf: [u8; 16] = [0; 16];
        for chunk in chunk_data_list.elements.iter_mut() {
            data.read_exact(&mut secret_guid_buf).expect("Failed to read chunk secret_guid!");
            chunk.secret_guid = [
                u32::from_le_bytes(secret_guid_buf[0..4].try_into().unwrap()),
                u32::from_le_bytes(secret_guid_buf[4..8].try_into().unwrap()),
                u32::from_le_bytes(secret_guid_buf[8..12].try_into().unwrap()),
                u32::from_le_bytes(secret_guid_buf[12..16].try_into().unwrap()),
            ];
        }

        for _chunk in chunk_data_list.elements.iter_mut() {
            data.read_exact(&mut buf).expect("Failed to read chunk window_size_compressed!");
        }

        let mut tag_buf: [u8; 16] = [0; 16];
        for chunk in chunk_data_list.elements.iter_mut() {
            data.read_exact(&mut tag_buf).expect("Failed to read chunk encryption_tag!");
            chunk.encryption_tag = tag_buf;
        }

        for chunk in chunk_data_list.elements.iter_mut() {
            if let Some(key) = secret_keys.get(&crate::download::guid_hex_upper(&chunk.secret_guid)) {
                chunk.secret_key = hex_to_bytes(key).and_then(|bytes| bytes.try_into().ok());
            }
        }
    }

    let size_read = (start_remaining - data.len()) as u32;
    if size_read != chunk_data_list.size {
        let missing = chunk_data_list.size as i64 - size_read as i64;
        warn!(
            "Did not read entire chunk data list! Version: {}, {} bytes missing, skipping...",
            chunk_data_list.version, missing
        );
        if missing > 0 {
            let mut skip = vec![0u8; missing as usize];
            data.read_exact(&mut skip).expect("Failed to skip remaining chunk data list bytes!");
        }
        // downgrade version to prevent issues during serialisation
        chunk_data_list.version = 0;
    }

    Ok(chunk_data_list)
}

fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

// Helper function to read a fstring from the data
fn read_fstring(data: &mut &[u8]) -> String {
    let mut len_buf: [u8; 4] = [0; 4];
    data.read_exact(&mut len_buf).expect("Failed to read fstring length!");

    let length = i32::from_le_bytes(len_buf);

    // Negative length means UTF-16; positive means ASCII. Length includes null terminator.
    if length < 0 {
        let byte_len = (-length as usize) * 2;
        let mut s = vec![0u8; byte_len];

        data.read_exact(&mut s).expect("Failed to read utf-16 fstring!");
        let utf16: Vec<u16> = s[..s.len().saturating_sub(2)]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();

        String::from_utf16_lossy(&utf16)

    } else if length > 0 {
        let mut s = vec![0u8; length as usize];
        data.read_exact(&mut s).expect("Failed to read ascii fstring!");
        String::from_utf8_lossy(&s[..s.len().saturating_sub(1)]).into_owned()

    } else {
        String::new()
    }
}

// Parse the file manifest list from the decompressed payload
async fn parse_file_manifest_list(data: &mut &[u8]) -> Result<FileManifestList, MonarchEgsError> {
    let mut file_manifest_list: FileManifestList = FileManifestList::default();
    let start_remaining = data.len();

    let mut buf: [u8; 4] = [0; 4];
    data.read_exact(&mut buf).expect("Failed to read file manifest list size!");
    file_manifest_list.size = u32::from_le_bytes(buf);

    let mut byte_buf: [u8; 1] = [0; 1];
    data.read_exact(&mut byte_buf).expect("Failed to read file manifest list version!");
    file_manifest_list.version = byte_buf[0] as u32;

    data.read_exact(&mut buf).expect("Failed to read file manifest list count!");
    file_manifest_list.count = u32::from_le_bytes(buf);

    for _ in 0..file_manifest_list.count {
        file_manifest_list.elements.push(FileManifest::default());
    }

    for fm in file_manifest_list.elements.iter_mut() {
        fm.filename = read_fstring(data);
    }

    // never seen this used in any of the manifests I checked but can't wait for something to break because of it
    for fm in file_manifest_list.elements.iter_mut() {
        fm.symlink_target = read_fstring(data);
    }

    // For files this is actually the SHA1 instead of whatever it is for chunks...
    let mut hash_buf: [u8; 20] = [0; 20];
    for fm in file_manifest_list.elements.iter_mut() {
        data.read_exact(&mut hash_buf).expect("Failed to read file manifest hash!");
        fm.hash = hash_buf;
    }

    // Flags, the only one I've seen is for executables
    for fm in file_manifest_list.elements.iter_mut() {
        data.read_exact(&mut byte_buf).expect("Failed to read file manifest flags!");
        fm.flags = byte_buf[0] as u32;
    }

    // install tags, no idea what they do, I've only seen them in the Fortnite manifest
    for fm in file_manifest_list.elements.iter_mut() {
        data.read_exact(&mut buf).expect("Failed to read install_tags count!");
        let elem = u32::from_le_bytes(buf);
        for _ in 0..elem {
            fm.install_tags.push(read_fstring(data));
        }
    }

    // Each file is made up of "Chunk Parts" that can be spread across the "chunk stream"
    let mut guid_buf: [u8; 16] = [0; 16];
    for fm in file_manifest_list.elements.iter_mut() {
        data.read_exact(&mut buf).expect("Failed to read chunk_parts count!");
        let elem = u32::from_le_bytes(buf);
        let mut offset: u32 = 0;
        for _ in 0..elem {
            let mut chunkp = ChunkPart::default();
            let part_start_remaining = data.len();

            data.read_exact(&mut buf).expect("Failed to read chunk part size!");
            let part_size = u32::from_le_bytes(buf);

            data.read_exact(&mut guid_buf).expect("Failed to read chunk part guid!");
            chunkp.guid = [
                u32::from_le_bytes(guid_buf[0..4].try_into().unwrap()),
                u32::from_le_bytes(guid_buf[4..8].try_into().unwrap()),
                u32::from_le_bytes(guid_buf[8..12].try_into().unwrap()),
                u32::from_le_bytes(guid_buf[12..16].try_into().unwrap()),
            ];

            data.read_exact(&mut buf).expect("Failed to read chunk part offset!");
            chunkp.offset = u32::from_le_bytes(buf);

            data.read_exact(&mut buf).expect("Failed to read chunk part size field!");
            chunkp.size = u32::from_le_bytes(buf);

            chunkp.file_offset = offset;
            offset += chunkp.size;
            fm.chunk_parts.push(chunkp);

            let part_read = (part_start_remaining - data.len()) as u32;
            if part_read < part_size {
                let missing = (part_size - part_read) as usize;
                warn!("Did not read {} bytes from chunk part!", missing);
                let mut skip = vec![0u8; missing];
                data.read_exact(&mut skip).expect("Failed to skip remaining chunk part bytes!");
            }
        }
    }

    // MD5 hash + MIME type (Manifest feature level 19)
    if file_manifest_list.version >= 1 {
        let mut md5_buf: [u8; 16] = [0; 16];
        for fm in file_manifest_list.elements.iter_mut() {
            data.read_exact(&mut buf).expect("Failed to read has_md5!");
            let has_md5 = u32::from_le_bytes(buf);
            if has_md5 != 0 {
                data.read_exact(&mut md5_buf).expect("Failed to read hash_md5!");
                fm.hash_md5 = md5_buf;
            }
        }

        for fm in file_manifest_list.elements.iter_mut() {
            fm.mime_type = read_fstring(data);
        }
    }

    // SHA256 hash (Manifest feature level 20)
    if file_manifest_list.version >= 2 {
        let mut sha256_buf: [u8; 32] = [0; 32];
        for fm in file_manifest_list.elements.iter_mut() {
            data.read_exact(&mut sha256_buf).expect("Failed to read hash_sha256!");
            fm.hash_sha256 = sha256_buf;
        }
    }

    // we have to calculate the actual file size ourselves
    for fm in file_manifest_list.elements.iter_mut() {
        fm.file_size = fm.chunk_parts.iter().map(|c| c.size as u64).sum();
    }

    let size_read = (start_remaining - data.len()) as u32;
    if size_read != file_manifest_list.size {
        let missing = file_manifest_list.size as i64 - size_read as i64;
        warn!(
            "Did not read entire file data list! Version: {}, {} bytes missing, skipping...",
            file_manifest_list.version, missing
        );
        if missing > 0 {
            let mut skip = vec![0u8; missing as usize];
            data.read_exact(&mut skip).expect("Failed to skip remaining file manifest list bytes!");
        }
        // downgrade version to prevent issues during serialisation
        file_manifest_list.version = 0;
    }

    Ok(file_manifest_list)
}

async fn parse_custom_fields(data: &mut &[u8]) -> Result<CustomFields, MonarchEgsError> {
    let mut custom_fields: CustomFields = CustomFields::default();
    let start_remaining = data.len();

    let mut buf: [u8; 4] = [0; 4];
    data.read_exact(&mut buf).expect("Failed to read custom fields size!");
    custom_fields.size = u32::from_le_bytes(buf);

    let mut byte_buf: [u8; 1] = [0; 1];
    data.read_exact(&mut byte_buf).expect("Failed to read custom fields version!");
    custom_fields.version = byte_buf[0] as u32;

    data.read_exact(&mut buf).expect("Failed to read custom fields count!");
    custom_fields.count = u32::from_le_bytes(buf);

    let mut keys = Vec::with_capacity(custom_fields.count as usize);
    for _ in 0..custom_fields.count {
        keys.push(read_fstring(data));
    }

    let mut values = Vec::with_capacity(custom_fields.count as usize);
    for _ in 0..custom_fields.count {
        values.push(read_fstring(data));
    }

    custom_fields._dict = keys.into_iter().zip(values).collect();

    let size_read = (start_remaining - data.len()) as u32;
    if size_read != custom_fields.size {
        let missing = custom_fields.size as i64 - size_read as i64;
        warn!(
            "Did not read entire custom fields part! Version: {}, {} bytes missing, skipping...",
            custom_fields.version, missing
        );
        if missing > 0 {
            let mut skip = vec![0u8; missing as usize];
            data.read_exact(&mut skip).expect("Failed to skip remaining custom fields bytes!");
        }
        // downgrade version to prevent issues during serialisation
        custom_fields.version = 0;
    }

    Ok(custom_fields)
}