/*
    This file is for parsing Valve's .vdf (Valve Data Format) format.
    It is used for reading content related to steam such as the users installed library, library locations in the filesystem, etc.
*/

use serde::{Serialize, Deserialize};
use vdf_serde;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename = "libraryfolders")] // Tell serde to look for "libraryfolders" instead of "LibraryFolders" when parsing
struct LibraryFolders(HashMap<String, LibraryLocation>); // Can take a variable amount of libraries

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct LibraryLocation {
    path: String,
    label: String,
    contentid: String,
    totalsize: String,
    update_clean_bytes_tally: String,
    time_last_update_corruption: String,
    apps: HashMap<String, String>,
}

/// Parses steams libraryfolders.vdf file to structs that can be used to find
/// installed games, folder locations, etc...
pub fn parse_library_file(path: &Path) -> Result<Vec<String>> {
    let mut content: String = fs::read_to_string(&path).with_context(|| 
        -> String {format!("monarch_vdf::parse_library_file() failed! Failed to open file: {} | Err", path.display()).into()})?;

    content = content.replace("\"\"", "\" \" "); // Remove blank space interfering with serde
    
    let library_folders = vdf_serde::from_str::<LibraryFolders>(&content).with_context(||
        -> String {format!("monarch_vdf::parse_library_file() failed! Could not automatically parse file content to LibraryFolders using vdf_serde! | Err").into()})?;

    let game_ids: Vec<String> = found_games(library_folders).iter().map(|game| game.to_owned()).collect();
    Ok(game_ids)
}

/// Helper function to extract the installed apps from a LibraryFolders struct.
fn found_games(libraryfolders: LibraryFolders) -> HashSet<String> {
    // Using hashset to ignore any (unlikely) duplicates that could show up.
    let mut games: HashSet<String> = HashSet::new(); 

    for location in libraryfolders.0 {
        for game in location.1.apps {
            games.insert(game.0);
        }
    }
    games
}