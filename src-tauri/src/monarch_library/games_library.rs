use log::error;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use serde_json::{value::Value, json};
use anyhow::{Context, Result, anyhow};

use crate::monarch_games::monarchgame::MonarchGame;
use crate::monarch_utils::monarch_fs::{write_json_content, get_library_json_path, get_monarch_games_path, path_exists};

pub fn write_games(games: Vec<MonarchGame>) -> Result<()> {
    let path: PathBuf = get_library_json_path().with_context(||
        -> String {format!("games_library::write_games() failed! Cannot get path to library.json! | Err")})?;

    write_json_content(json!(games), &path)
        .context(format!("games_library::write_games() failed! Error while writing library to: {file} | Err", file = path.display()))
}

/// Writes new games to monarch_games.json for Monarch to track what games it installed itself.
pub fn write_monarchgame(game: MonarchGame) -> Result<()> {
    let path: PathBuf = get_monarch_games_path().with_context(||
        -> String {format!("games_library::write_monarchgame() failed! Cannot get path to monarch_games.json! | Err")})?;
    
    let mut games: Vec<MonarchGame> = Vec::new();

    if !path_exists(&path) {
        if let Err(e) = fs::File::create(&path) {
            error!("games_library::get_monarchgames() failed! Could not create new file {file} | Error: {e}", file = path.display());
            return Err(anyhow!("Failed to create new monarch_games.json!"))
        }
    }
    else {
        let file: File = fs::File::open(&path).with_context(||
            -> String {format!("games_library::get_monarchgames() failed! Error opening: {file} | Err", file = path.display())})?;
        
        if let Ok(content) = serde_json::from_reader::<fs::File, Vec<MonarchGame>>(file) {
            games = content;
        }
    }

    games.push(game);
    write_json_content(json!(games), &path).context(format!("games_library::write_monarchgame() failed! Error while writing library to: {file} | Err", file = path.display()))
}

/// Returns JSON of games from library
pub fn get_games() -> Result<Value> {
    let path: PathBuf = get_library_json_path().with_context(|| 
    -> String {format!("games_library::get_games() failed! Cannot get path to library.json! | Err")})?;

    let file: File = fs::File::open(&path).with_context(|| 
        -> String {format!("games_library::get_games() failed! Error opening: {file} | Err", file = path.display())})?;

    let games: Value = serde_json::from_reader(file).with_context(||
        -> String {format!("games_library::get_games() failed! Failed to parse json! | Err")})?;
    
    return Ok(games) // Seperate return statement for verbosity
}

/// Returns JSON of games installed by Monarch
pub fn get_monarchgames() -> Result<Vec<MonarchGame>> {
    let path: PathBuf = get_monarch_games_path().with_context(|| 
        -> String {format!("games_library::get_monarchgames() failed! Cannot get path to library.json! | Err")})?;
    
    let file: File = fs::File::open(&path).with_context(|| 
        -> String {format!("games_library::get_monarchgames() failed! Error opening: {file} | Err", file = path.display())})?;

    let games: Vec<MonarchGame> = serde_json::from_reader(file).with_context(|| 
        -> String {format!("games_library::get_monarchgames() failed! Could not parse json value as Vec<MonarchGame> | Er")})?;

    return Ok(games)
}

/// Backend functionality for adding a new game that's been installed.
pub fn add_game(game: MonarchGame) -> Result<()> {
    write_monarchgame(game.clone()).context(format!("games_library::add_game() failed! Could not write new game to monarch_games.json! | Err"));

    let games_json: Value = get_games().with_context(|| 
        -> String {format!("games_library::add_game() failed! get_games() returned error! | Err")})?;

    let games: Vec<MonarchGame> = serde_json::from_value(games_json).with_context(|| 
        -> String {format!("games_library::add_game() failed! Failed to parse json to Vec<MonarchGame>! | Err")})?;

    games.push(game);
    write_games(games)
}
