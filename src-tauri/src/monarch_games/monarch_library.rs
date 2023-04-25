use json::{object, array, JsonValue};
use log::error;
use std::fs;
use serde_json::{value::Value, json};

use super::monarchgame::MonarchGame;
use crate::monarch_utils::monarch_fs::{write_json_content, get_library_json_path};

pub fn write_games(games: Vec<MonarchGame>) {
    let mut games_objects = array![];

    for game in games {
        games_objects.push(game_to_json(game)).unwrap();
    }

    let path = get_library_json_path();

    if let Err(e) = write_json_content(games_objects, &path) {
        error!("Failed to write new library to: {} | Message: {:?}", path, e);
    }
}

pub fn game_to_json(game: MonarchGame) -> JsonValue {
    let data = object! {
        name: game.get_name(),
        id: game.get_id().to_string(),
        platform: game.get_platform(),
        platform_id: game.get_platform_id(),
        thumbnail_path: game.get_thumbnail_path(),
        exec_path: game.get_exec_path(),
    };
    return data
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