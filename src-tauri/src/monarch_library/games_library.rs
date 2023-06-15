use log::error;
use std::fs;
use std::path::PathBuf;
use serde_json::{value::Value, json};

use crate::monarch_games::monarchgame::MonarchGame;
use crate::monarch_utils::monarch_fs::{write_json_content, get_library_json_path};

pub fn write_games(games: Vec<MonarchGame>) -> Result<(), String> {
    let path: PathBuf;

    match get_library_json_path() {
        Ok(json_path) => { path = json_path; }
        Err(e) => {
            error!("Failed to get path to library.json! | Message: {:?}", e);
            return Err("Failed to get path to library.json!".to_string())
        } 
    }

    if let Err(e) = write_json_content(json!(games), path.clone()) {
        error!("Failed to write new library to: {} | Message: {:?}", path.display(), e);
        return Err("Failed to write content to library.json!".to_string())
    }

    Ok(())
}

/// Returns JSON of games from library
pub fn get_games() -> Result<Value, String> {
    let games: Value;
    let path: PathBuf;

    match get_library_json_path() {
        Ok(json_path) => { path = json_path; }
        Err(e) => {
            error!("Failed to get path to library.json! | Message: {:?}", e);
            return Err("Failed to get path to library.json!".to_string())
        } 
    match fs::File::open(path.clone()) {
        Ok(file) => {
            match serde_json::from_reader(file) {
                Ok(json_content) => { games = json_content }
                Err(e) => {
                    error!("Failed to parse json content! | Message: {:?}", e);
                    return Err("Failed to parse library.json content!".to_string())
                }
            }
        }
        Err(e) => {
            error!("Failed to open file: {} | Message: {:?}", path.display(), e);
            return Err("Failed to open library.json!".to_string())
        }
    }
    return Ok(games)
}