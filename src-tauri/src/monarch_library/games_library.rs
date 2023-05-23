use log::error;
use std::fs;
use serde_json::{value::Value, json};
use serde::{Serialize, Deserialize};

use crate::monarch_games::monarchgame::MonarchGame;
use crate::monarch_utils::monarch_fs::{write_json_content, get_library_json_path};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MonarchLibrary {
    games: Vec<MonarchGame>
}

pub fn write_games(games: Vec<MonarchGame>) {
    let library: MonarchLibrary = MonarchLibrary { games: games };
    let path = get_library_json_path();

    if let Err(e) = write_json_content(json!(library), &path) {
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