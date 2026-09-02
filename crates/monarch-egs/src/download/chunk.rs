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
