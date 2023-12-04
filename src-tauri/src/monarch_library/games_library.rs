use log::error;
use std::fs;
use std::path::PathBuf;
use serde_json::{value::Value, json};

use crate::monarch_games::monarchgame::MonarchGame;
use crate::monarch_utils::monarch_fs::{write_json_content, get_library_json_path, get_monarch_games_path, path_exists};

pub fn write_games(games: Vec<MonarchGame>) -> Result<(), String> {
    let path: PathBuf;

    match get_library_json_path() {
        Ok(json_path) => { path = json_path; }
        Err(e) => {
            error!("games_library::write_games() failed! Cannot get path to library.json! | Error: {e}");
            return Err("Failed to get path to library.json!".to_string())
        } 
    }

    if let Err(e) = write_json_content(json!(games), &path) {
        error!("games_library::write_games() failed! Error while writing library to: {file} | Error: {e}", file = path.display());
        return Err("Failed to write content to library.json!".to_string())
    }

    Ok(())
}

/// Writes new games to monarch_games.json for Monarch to track what games it installed itself.
pub fn write_monarchgame(game: MonarchGame) -> Result<(), String> {
    let path: PathBuf;
    let mut games: Vec<MonarchGame> = Vec::new();

    match get_monarch_games_path() {
        Ok(json_path) => { path = json_path; }
        Err(e) => {
            error!("games_library::write_monarchgame() failed! Cannot get path to library.json! | Error: {e}");
            return Err("Failed to get path to library.json!".to_string())
        } 
    }

    if !path_exists(&path) {
        if let Err(e) = fs::File::create(&path) {
            error!("games_library::get_monarchgames() failed! Could not create new file {file} | Error: {e}", file = path.display());
            return Err(String::from("Failed to create new monarch_games.json!"))
        }
    }
    else {
        match fs::File::open(&path) {
            Ok(file) => {
                match serde_json::from_reader::<fs::File, Vec<MonarchGame>>(file) {
                    Ok(content) => { 
                        games = content;
                    }
                    Err(e) => {
                        error!("games_library::monarchget_games() failed! Failed to parse json! | Error: {e}");
                        return Err("Failed to parse library.json content!".to_string())
                    }
                }
            }
            Err(e) => {
                error!("games_library::get_monarchgames() failed! Error opening: {file} | Error: {e}", file = path.display());
                return Err("Failed to open library.json!".to_string())
            }
        }
    }

    games.push(game);
    if let Err(e) = write_json_content(json!(games), &path) {
        error!("games_library::write_monarchgame() failed! Error while writing library to: {file} | Error: {e}", file = path.display());
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
            error!("games_library::get_games() failed! Cannot get path to library.json! | Error: {e}");
            return Err("Failed to get path to library.json!".to_string())
        }
    }

    match fs::File::open(&path) {
        Ok(file) => {
            match serde_json::from_reader(file) {
                Ok(json_content) => { games = json_content }
                Err(e) => {
                    error!("games_library::get_games() failed! Failed to parse json! | Error: {e}");
                    return Err("Failed to parse library.json content!".to_string())
                }
            }
        }
        Err(e) => {
            error!("games_library::get_games() failed! Error opening: {file} | Error: {e}", file = path.display());
            return Err("Failed to open library.json!".to_string())
        }
    }
    return Ok(games)
}

/// Returns JSON of games installed by Monarch
pub fn get_monarchgames() -> Result<Vec<MonarchGame>, String> {
    let games: Vec<MonarchGame>;
    let path: PathBuf;

    match get_monarch_games_path() {
        Ok(json_path) => { path = json_path; }
        Err(e) => {
            error!("games_library::get_monarchgames() failed! Cannot get path to library.json! | Error: {e}");
            return Err("Failed to get path to library.json!".to_string())
        }
    }
    match fs::File::open(&path) {
        Ok(file) => {
            match serde_json::from_reader(file) {
                Ok(json_content) => { 
                    match serde_json::from_value::<Vec<MonarchGame>>(json_content) {
                        Ok(content) => {
                            return Ok(content)
                        }
                        Err(e) => {
                            error!("games_library::get_monarchgames() failed! Could not parse json value as Vec<MonarchGame> | Error: {e}");
                            return Err(String::from("Failed to parse json to Vec<MonarGame>"))
                        }
                    }
                }
                Err(e) => {
                    error!("games_library::monarchget_games() failed! Failed to parse json! | Error: {e}");
                    return Err("Failed to parse library.json content!".to_string())
                }
            }
        }
        Err(e) => {
            error!("games_library::get_monarchgames() failed! Error opening: {file} | Error: {e}", file = path.display());
            return Err("Failed to open library.json!".to_string())
        }
    }
    return Ok(games)
}

/// Backend functionality for adding a new game that's been installed.
pub fn add_game(game: MonarchGame) -> Result<(), String> {
    if let Err(e) = write_monarchgame(game.clone()) {
        error!("games_library::add_game() failed! Could not write new game to monarch_games.json! | Error: {e}");
        return Err(String::from("Could not write new game to monarch_games.json!"))
    }
    match get_games() {
        Ok(games_json) => {
            match serde_json::from_value::<Vec<MonarchGame>>(games_json) {
                Ok(mut vec) => {
                    vec.push(game);
                    return write_games(vec)
                }
                Err(e) => {
                    error!("games_library::add_game() failed! Failed to parse json to Vec<MonarchGame>! | Error: {e}");
                    return Err(String::from("Failed to add new game!"))
                }
            }
        }
        Err(e) => {
            error!("games_library::add_game() failed! get_games() returned errro! | Error: {e}");
            return Err("Failed to add new game!".to_string())
        }
    }
}
