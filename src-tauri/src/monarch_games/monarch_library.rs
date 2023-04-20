use json::{object, array, JsonValue};
use log::error;

use super::monarchgame::MonarchGame;
use crate::monarch_utils::monarch_fs::{write_json_content, get_library_json_path};

pub fn write_games(games: Vec<MonarchGame>) {
    let mut games_objects = array![];

    for game in games {
        games_objects.push(parse_game(game)).unwrap();
    }

    let path = get_library_json_path();

    if let Err(e) = write_json_content(games_objects, &path) {
        error!("Failed to write new library to: {} | Message: {:?}", path, e);
    }
}

pub fn parse_game(game: MonarchGame) -> JsonValue {
    let data = object! {
        name: game.get_name(),
        id: game.get_id(),
        platform: game.get_platform(),
        thumbnail_path: game.get_thumbnail_path(),
        exec_path: game.get_exec_path(),
    };
    return data
}

pub fn get_games() -> Vec<MonarchGame> {
    let games: Vec<MonarchGame> = Vec::new();

    return games
}