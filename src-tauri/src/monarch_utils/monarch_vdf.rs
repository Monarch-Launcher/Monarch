/*
    This file is for parsing Valve's .vdf (Valve Data Format) format.
    It is used for reading content related to steam such as the users installed library, library locations in the filesystem, etc.
*/

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use vdf_serde::from_str;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename = "libraryfolders")]
struct LibraryFolders {
    #[serde(rename = "0")]
    folder_0: LibraryLocation,

    #[serde(rename = "1")]
    folder_1: LibraryLocation,
}

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

pub fn parse_library_file(path: &str) {
    let content = fs::read_to_string(path).unwrap();
    
    match from_str::<LibraryFolders>(&content) {
        Ok(structs) => println!("{:?}", structs),
        Err(e) => println!("Structs failed: {:?}", e)
    }
}