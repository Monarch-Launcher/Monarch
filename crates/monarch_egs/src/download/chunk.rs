use std::io::prelude::*;

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use flate2::read::ZlibDecoder;
use sha1::{Digest, Sha1};

use crate::download::ChunkInfo;
use crate::utils::err::MonarchEgsError;

const CHUNK_MAGIC: u32 = 0xB1FE3AA2;

/// Parses and decompresses a raw chunk fetched from the CDN, verifying its
/// SHA1 hash against the manifest.
///
/// Returns the decompressed (and decrypted, if applicable) chunk data.
pub fn process_chunk(raw: &[u8], chunk: &ChunkInfo) -> Result<Vec<u8>, MonarchEgsError> {
    let header = parse_header(raw)?;

    let mut data: Vec<u8> = raw[header.header_size..].to_vec();

    if header.stored_as & 0x2 != 0 {
        data = decrypt_chunk(&data, chunk, &header)?;
    }

    if header.stored_as & 0x1 != 0 {
        data = zlib_decompress(&data)?;
    }

    // Some builds pad chunks with zeroes up to the window size; strip the
    // padding before hashing/verifying so we compare against the manifest's
    // SHA1 (which is computed over the unpadded window content).
    let expected = chunk.window_size() as usize;
    if data.len() > expected && chunk.window_size() > 0 {
        data.truncate(expected);
    }

    let computed: String = Sha1::digest(&data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let expected_hash: String = chunk
        .sha_hash()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();

    if computed != expected_hash {
        return Err(MonarchEgsError::HashMismatchError(format!(
            "Chunk sha1 mismatch! | Computed: {computed}, Expected: {expected_hash}"
        )));
    }

    Ok(data)
}

struct ChunkHeader {
    header_size: usize,
    stored_as: u8,
}

fn parse_header(raw: &[u8]) -> Result<ChunkHeader, MonarchEgsError> {
    let mut data = raw;

    if data.len() < 21 {
        return Err(MonarchEgsError::ParsingError(
            "Chunk data too short to contain a header".to_string(),
        ));
    }

    let mut buf: [u8; 4] = [0; 4];
    data.read_exact(&mut buf).map_err(chunk_read_err)?;
    let magic = u32::from_le_bytes(buf);
    if magic != CHUNK_MAGIC {
        return Err(MonarchEgsError::ParsingError(format!(
            "Invalid chunk magic! | Expected: 0x{CHUNK_MAGIC:08x}, Got: 0x{magic:08x}"
        )));
    }

    data.read_exact(&mut buf).map_err(chunk_read_err)?;
    let header_version = u32::from_le_bytes(buf);

    data.read_exact(&mut buf).map_err(chunk_read_err)?;
    let header_size = u32::from_le_bytes(buf) as usize;

    data.read_exact(&mut buf).map_err(chunk_read_err)?; // compressed_size

    let mut guid_buf: [u8; 16] = [0; 16];
    data.read_exact(&mut guid_buf).map_err(chunk_read_err)?; // guid

    let mut hash_buf: [u8; 8] = [0; 8];
    data.read_exact(&mut hash_buf).map_err(chunk_read_err)?; // hash

    let mut stored_buf: [u8; 1] = [0; 1];
    data.read_exact(&mut stored_buf).map_err(chunk_read_err)?;
    let stored_as = stored_buf[0];

    // SHA1 + hash type (header version >= 2)
    let mut sha_buf: [u8; 20] = [0; 20];
    data.read_exact(&mut sha_buf).map_err(chunk_read_err)?;
    let mut hash_type_buf: [u8; 1] = [0; 1];
    data.read_exact(&mut hash_type_buf).map_err(chunk_read_err)?;

    // Uncompressed size (header version >= 3)
    if header_version >= 3 {
        data.read_exact(&mut buf).map_err(chunk_read_err)?;
    }

    // Secret GUID + encryption tag (header version >= 4)
    if header_version >= 4 {
        let mut secret_buf: [u8; 32] = [0; 32];
        data.read_exact(&mut secret_buf).map_err(chunk_read_err)?;
    }

    let read = raw.len() - data.len();
    if header_size < read {
        return Err(MonarchEgsError::ParsingError(format!(
            "Chunk header size mismatch! | Header: {header_size}, Read: {read}"
        )));
    }
    if header_size > raw.len() {
        return Err(MonarchEgsError::ParsingError(format!(
            "Chunk header size {header_size} exceeds chunk length {}",
            raw.len()
        )));
    }

    Ok(ChunkHeader {
        header_size,
        stored_as,
    })
}

fn decrypt_chunk(
    data: &[u8],
    chunk: &ChunkInfo,
    header: &ChunkHeader,
) -> Result<Vec<u8>, MonarchEgsError> {
    let key = *chunk
        .secret_key()
        .ok_or_else(|| {
            MonarchEgsError::ParsingError(
                "Chunk is encrypted but no secret key was found in the manifest".to_string(),
            )
        })?;

    // AES-256-GCM: nonce = first 12 bytes of the chunk SHA1, tag stored in the
    // chunk header (encryption tag), ciphertext is everything after the header.
    let nonce_bytes: [u8; 12] = chunk.sha_hash()[..12].try_into().unwrap();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| MonarchEgsError::ParsingError("Invalid AES key length".to_string()))?;

    // If the encryption tag is empty (older encrypted builds), attempt to treat
    // the last 16 bytes as the tag.
    let (ciphertext, tag) = if chunk.encryption_tag() == &[0u8; 16] {
        if data.len() < 16 {
            return Err(MonarchEgsError::ParsingError(
                "Encrypted chunk payload too short".to_string(),
            ));
        }
        let (ct, tag) = data.split_at(data.len() - 16);
        (ct, tag)
    } else {
        (data, chunk.encryption_tag().as_slice())
    };

    let _ = header;
    // `Aead::decrypt` expects the authentication tag appended to the
    // ciphertext, so reassemble ciphertext || tag.
    let mut combined = Vec::with_capacity(ciphertext.len() + tag.len());
    combined.extend_from_slice(ciphertext);
    combined.extend_from_slice(tag);
    cipher
        .decrypt(nonce, combined.as_slice())
        .map_err(|_| {
            MonarchEgsError::HashMismatchError("Failed to decrypt chunk (bad tag)".to_string())
        })
}

fn zlib_decompress(data: &[u8]) -> Result<Vec<u8>, MonarchEgsError> {
    let mut decoder = ZlibDecoder::new(data);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out).map_err(|_| {
        MonarchEgsError::ParsingError("Failed to zlib-decompress chunk data".to_string())
    })?;
    Ok(out)
}

fn chunk_read_err(e: std::io::Error) -> MonarchEgsError {
    MonarchEgsError::ParsingError(format!("Failed to read chunk header: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;

    fn header_size(version: u32) -> usize {
        match version {
            1 => 41,
            2 => 62,
            3 => 66,
            4 => 98,
            _ => panic!("unexpected header version"),
        }
    }

    /// Build a raw chunk: header (version 3) + data payload. `sha` is the hash
    /// the manifest claims for the chunk's *decompressed* window content.
    fn build_chunk(
        payload: &[u8],
        stored_as: u8,
        sha: &[u8; 20],
        window_size: u32,
    ) -> Vec<u8> {
        let hsize = header_size(3);
        let mut raw = Vec::new();
        raw.extend_from_slice(&CHUNK_MAGIC.to_le_bytes());
        raw.extend_from_slice(&3u32.to_le_bytes()); // header_version
        raw.extend_from_slice(&(hsize as u32).to_le_bytes());
        raw.extend_from_slice(&(payload.len() as u32).to_le_bytes()); // compressed_size
        for word in [0xDEADBEEFu32, 0x01020304, 0xA0B0C0D0, 0x11223344] {
            raw.extend_from_slice(&word.to_le_bytes()); // guid
        }
        raw.extend_from_slice(&0x1122334455667788u64.to_le_bytes()); // hash
        raw.push(stored_as);
        raw.extend_from_slice(sha);
        raw.push(1); // hash_type
        raw.extend_from_slice(&window_size.to_le_bytes()); // uncompressed_size
        raw.extend_from_slice(payload);
        assert_eq!(raw.len(), hsize + payload.len());
        raw
    }

    fn chunk_info(sha: &[u8; 20], window_size: u32) -> ChunkInfo {
        let mut chunk = ChunkInfo::default();
        chunk.sha_hash = *sha;
        chunk.window_size = window_size;
        chunk
    }

    fn zlib(payload: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        std::io::Write::write_all(&mut encoder, payload).unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn processes_plain_uncompressed_chunk() {
        let payload = b"some plaintext chunk payload".to_vec();
        let sha: [u8; 20] = Sha1::digest(&payload).into();
        let raw = build_chunk(&payload, 0, &sha, payload.len() as u32);
        let info = chunk_info(&sha, payload.len() as u32);
        let out = process_chunk(&raw, &info).unwrap();
        assert_eq!(out, payload);
    }

    #[test]
    fn processes_zlib_compressed_chunk() {
        let payload = b"repeated repeated repeated chunk data".to_vec();
        let sha: [u8; 20] = Sha1::digest(&payload).into();
        let compressed = zlib(&payload);
        let raw = build_chunk(&compressed, 0x1, &sha, payload.len() as u32);
        let info = chunk_info(&sha, payload.len() as u32);
        let out = process_chunk(&raw, &info).unwrap();
        assert_eq!(out, payload);
    }

    #[test]
    fn strips_zero_padding_to_window_size() {
        let payload = b"padded payload".to_vec();
        let sha: [u8; 20] = Sha1::digest(&payload).into();
        // Simulate a padded chunk: window_size is smaller than the data.
        let mut padded = payload.clone();
        padded.resize(payload.len() + 8, 0);
        let raw = build_chunk(&padded, 0, &sha, payload.len() as u32);
        let info = chunk_info(&sha, payload.len() as u32);
        let out = process_chunk(&raw, &info).unwrap();
        assert_eq!(out, payload);
    }

    #[test]
    fn rejects_hash_mismatch() {
        let payload = b"content".to_vec();
        let correct: [u8; 20] = Sha1::digest(&payload).into();
        let wrong: [u8; 20] = Sha1::digest(b"different content").into();
        let raw = build_chunk(&payload, 0, &correct, payload.len() as u32);
        let info = chunk_info(&wrong, payload.len() as u32);
        let err = process_chunk(&raw, &info).unwrap_err();
        assert!(matches!(err, MonarchEgsError::HashMismatchError(_)));
    }

    #[test]
    fn rejects_bad_magic() {
        let payload = b"content".to_vec();
        let sha: [u8; 20] = Sha1::digest(&payload).into();
        let mut raw = build_chunk(&payload, 0, &sha, payload.len() as u32);
        raw[0] = 0x00;
        let info = chunk_info(&sha, payload.len() as u32);
        let err = process_chunk(&raw, &info).unwrap_err();
        assert!(matches!(err, MonarchEgsError::ParsingError(_)));
    }

    #[test]
    fn rejects_truncated_header() {
        let info = chunk_info(&[0u8; 20], 0);
        let err = process_chunk(&[0u8; 8], &info).unwrap_err();
        assert!(matches!(err, MonarchEgsError::ParsingError(_)));
    }
}
