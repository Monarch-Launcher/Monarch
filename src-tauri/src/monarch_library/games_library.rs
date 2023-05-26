use log::error;
use std::fs;
use serde_json::{value::Value, json};

use crate::monarch_games::monarchgame::MonarchGame;
use crate::monarch_utils::monarch_fs::{write_json_content, get_library_json_path};

pub fn write_games(games: Vec<MonarchGame>) {
    let path = get_library_json_path();

    if let Err(e) = write_json_content(json!(games), &path) {
        error!("Failed to write new library to: {} | Message: {:?}", path, e);
    }
}

/// Returns JSON of games from library
pub fn get_games() -> Value {
    let mut games: Value = json!({});
    let path: String = get_library_json_path();

    match fs::File::open(path.clone()) {
        Ok(file) => {
            match serde_json::from_reader(file) {
                Ok(json_content) => { games = json_content }
                Err(e) => {
                    error!("Failed to parse json content! | Message: {:?}", e);
                }
            }
        }
        Err(e) => {
            error!("Failed to open file: {} | Message: {:?}", path, e);
        }
    }

    return games
}