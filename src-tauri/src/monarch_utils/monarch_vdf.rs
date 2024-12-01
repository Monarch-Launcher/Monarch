/*
    This file is for parsing Valve's .vdf (Valve Data Format) format.
    It is used for reading content related to steam such as the users installed library, library locations in the filesystem, etc.
*/
use std::any::Any;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use log::info;
use keyvalues_parser::Vdf;

/// Parses steams libraryfolders.vdf file to structs that can be used to find
/// installed games, folder locations, etc...
pub fn parse_library_file(path: &Path) -> Result<Vec<String>> {
    info!("Parsing Steam file: {}", path.display());

    let mut content: String = fs::read_to_string(path).with_context(|| -> String {
        format!(
            "monarch_vdf::parse_library_file() Failed to open file: {} | Err",
            path.display()
        )
    })?;

    let mut library_folders = match Vdf::parse(&content) {
        Ok(smthn) => smthn,
        Err(e) => {
            println!("{e}");
            bail!("Failed to parse vdf!")
        }
    };

    let mut game_ids: Vec<String> = Vec::new();
    for library_location in library_folders.value.unwrap_obj().values().flatten() {
        if let Some((_, apps)) = library_location.clone().unwrap_obj().get_key_value("apps") {
            for app in apps {
                game_ids.append(&mut (app.clone()
                                         .unwrap_obj()
                                         .0
                                         .keys()
                                         .map(|key| key.to_string())
                                         .collect::<Vec<String>>()));
            }
        }
    }
    Ok(game_ids)
}