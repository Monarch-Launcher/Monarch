use log::{info, error};
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

    if let Ok(file) = fs::File::open(path) {
        if let Ok(json_content) = serde_json::from_reader(file) {
            games = json_content;
        }
    }

    return games
}