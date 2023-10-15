/*
    This file is for parsing Valve's .vdf (Valve Data Format) format.
    It is used for reading content related to steam such as the users installed library, library locations in the filesystem, etc.
*/

use serde::{Serialize, Deserialize};
use vdf_serde;
use std::collections::HashMap;
use std::fs;
use log::error;
use std::path::Path;

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
pub fn parse_library_file(path: &Path) -> Vec<String> {
    let mut games: Vec<String> = Vec::new();

    match fs::read_to_string(path) {
        Ok(mut content) =>  {
            content = content.replace("\"\"", "\" \" "); // Remove blank space interfering with serde

            match vdf_serde::from_str::<LibraryFolders>(&content) {
                Ok(libraryfolders) => {
                    games = found_games(libraryfolders);
                }
                Err(e) => error!("monarch_vdf::parse_library_file() failed! Could not automatically parse file content to LibraryFolders using vdf_serde! | Message: {e}")
            }
        }
        Err(e) => {
            error!("monarch_vdf::parse_library_file() failed! Failed to open file: {file} | Message: {e}", file = path.display());
        }
    }

    games
}

/// Helper function to extract the installed apps from a LibraryFolders struct.
fn found_games(libraryfolders: LibraryFolders) -> Vec<String> {
    let mut games: Vec<String> = Vec::new();

    for location in libraryfolders.0 {
        for game in location.1.apps {
            if !games.contains(&game.0) {
                games.push(game.0);
            }
        }
    }
    games
}