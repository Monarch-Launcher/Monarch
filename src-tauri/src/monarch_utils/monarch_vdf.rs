/*
    This file is for parsing Valve's .vdf (Valve Data Format) format.
    It is used for reading content related to steam such as the users installed library, library locations in the filesystem, etc.
*/

use serde::{Serialize, Deserialize};
use vdf_serde::from_str;
use std::collections::HashMap;
use std::fs;
use log::error;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename = "libraryfolders")]
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
pub fn parse_library_file(path: &str) -> Vec<String> {
    let mut games: Vec<String> = Vec::new();

    match fs::read_to_string(path) {
        Ok(mut content) =>  {
            content = content.replace("\"\"", "\" \" ");

            match from_str::<LibraryFolders>(&content) {
                Ok(libraryfolders) => {
                    games = found_games(libraryfolders);
                }
                Err(e) => error!("Failed to build structs from .vdf file in parse_library_file()! | Message: {:?}", e)
            }
        }
        Err(e) => {
            error!("Failed to open file: {} | Message: {:?}", path, e);
        }
    }

    return games
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
    return games
}