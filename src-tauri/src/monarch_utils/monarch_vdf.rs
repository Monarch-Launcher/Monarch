/*
    This file is for parsing Valve's .vdf (Valve Data Format) format.
    It is used for reading content related to steam such as the users installed library, library locations in the filesystem, etc.
*/
use anyhow::{Context, Result};
use keyvalues_serde;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize)]

pub struct LibraryFolders(pub HashMap<String, LibraryFolder>);

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryFolder {
    path: String,
    label: String,
    contentid: String,
    totalsize: String,
    update_clean_bytes_tally: String,
    time_last_update_verified: String,
    apps: HashMap<String, String>,
}

/// Parses steams libraryfolders.vdf file to structs that can be used to find
/// installed games, folder locations, etc...
pub fn parse_library_file(path: &Path) -> Result<Vec<String>> {
    info!("Reading: {}", path.display());

    // Read the file contents
    let contents = fs::read_to_string(path).with_context(|| {
        format!(
            "monarch_vdf::parse_library_file() Failed to read content of: {} | Err: ",
            path.display()
        )
    })?;

    // Parse JSON into the struct
    let folders: LibraryFolders = keyvalues_serde::from_str(&contents).with_context(|| "monarch_vdf::parse_library_file() Failed to parse .vdf content into LibraryFolders struct. | Err: ")?;

    info!("Found {} Steam folder locations.", folders.0.len());

    // Detect all .acf files and extract app_id from each library folder path
    let mut games: Vec<String> = Vec::new();
    for (_, folder) in folders.0 {
        let mut path: PathBuf = PathBuf::from(&folder.path);
        path = path.join("steamapps");

        match &mut get_games_from_manifest_files(&path) {
            Ok(found_games) => {
                games.append(found_games);
            }
            Err(e) => {
                error!(
                    "monarch_vdf::parse_library_file() Failed to get games in: {} | Err: {e}",
                    folder.path
                )
            }
        }
    }

    Ok(games)
}

fn get_games_from_manifest_files(path: &Path) -> Result<Vec<String>> {
    info!("Searching for games in: {}", path.display());

    let mut games: Vec<String> = Vec::new();

    // Read directory entries
    for entry in fs::read_dir(path).with_context(|| {
        "monarch_vdf::get_games_from_manifest_files() Failed to read directory entries. | Err: "
    })? {
        let entry = entry.with_context(|| {
            "monarch_vdf::get_games_from_manifest_files() Failed to read directory entry. | Err: "
        })?;
        let path = entry.path();

        // Extract app_id from all .acf files in diretory
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "acf" {
                    if let Some(file_name) = path.file_stem() {
                        if let Some(name) = file_name.to_str() {
                            games.push(name.split("_").last().unwrap().to_string());
                        }
                    }
                }
            }
        }
    }

    info!("Found IDs: {:?}", games);
    Ok(games)
}
