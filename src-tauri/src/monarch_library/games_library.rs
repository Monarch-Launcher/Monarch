use anyhow::{Context, Result};
use serde_json::{json, value::Value};
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use tracing::error;

use crate::monarch_games::monarchgame::MonarchGame;
use crate::monarch_utils::monarch_fs::{
    get_library_json_path, get_monarch_games_path, path_exists, write_json_content,
};
use crate::monarch_utils::monarch_state::MONARCH_STATE;

/// Overwrites library.json
pub fn write_games(games: &[MonarchGame]) -> Result<()> {
    let path: PathBuf = get_library_json_path();
    write_json_content(json!(games), &path).with_context(|| "games_library::write_games() -> ")
}

/// Overwrites list of games found in monarch_games.json
/// Use with caution
pub fn write_monarch_games(games: &[MonarchGame]) -> Result<()> {
    let path: PathBuf = get_monarch_games_path();
    write_json_content(json!(games), &path).with_context(|| "games_library::write_games() -> ")
}

/// Writes new games to monarch_games.json for Monarch to track what games it installed itself.
pub fn write_monarchgame(game: &MonarchGame) -> Result<()> {
    let path: PathBuf = get_monarch_games_path();
    let mut games: Vec<MonarchGame> = Vec::new();

    if !path_exists(&path) {
        fs::File::create(&path).with_context(|| {
            format!(
                "games_library::get_monarchgame() Could not create new file {file} | Err: ",
                file = path.display()
            )
        })?;
    } else {
        let file: File = fs::File::open(&path).with_context(|| {
            format!(
                "games_library::get_monarchgames() Error opening: {file} | Err",
                file = path.display()
            )
        })?;

        if let Ok(content) = serde_json::from_reader::<fs::File, Vec<MonarchGame>>(file) {
            games = content;
        }
    }

    games.push(game.clone());
    write_json_content(json!(games), &path)
        .with_context(|| "games_library::write_monarchgame() -> ")
}

/// Returns JSON of games from library
pub fn get_games() -> Result<Value> {
    let path: PathBuf = get_library_json_path();

    let file: File = fs::File::open(&path).with_context(|| -> String {
        format!(
            "games_library::get_games() Error opening: {file} | Err",
            file = path.display()
        )
    })?;

    let games: Value = serde_json::from_reader(file)
        .with_context(|| "games_library::get_games() Failed to parse json! | Err: ")?;
    Ok(games) // Seperate return statement for verbosity
}

/// Returns Vec of games installed by Monarch
pub fn get_monarchgames() -> Result<Vec<MonarchGame>> {
    let path: PathBuf = get_monarch_games_path();

    let file: File = fs::File::open(&path).with_context(|| -> String {
        format!(
            "games_library::get_monarchgames() Error opening: {file} | Err",
            file = path.display()
        )
    })?;

    let games: Vec<MonarchGame> = serde_json::from_reader(file).with_context(|| {
        "games_library::get_monarchgames() Could not parse json value as Vec<MonarchGame> | Err: "
    })?;
    Ok(games)
}

/// Backend functionality for adding a new game that's been installed.
pub fn add_game(game: &MonarchGame) -> Result<()> {
    let mut games: Vec<MonarchGame>;
    unsafe {
        games = MONARCH_STATE.get_library_games();
        games.push(game.clone());

        if let Err(e) = MONARCH_STATE.set_library_games(&games) {
            error!("games_library::add_game() -> {}", e.chain().map(|e| e.to_string()).collect::<String>());
        }
    }

    write_monarchgame(game)
}

/// Backend functionality for removing a game from library.json 
pub fn remove_game(game: &MonarchGame) -> Result<()> {
    let mut games: Vec<MonarchGame>;
    unsafe {
        games = MONARCH_STATE.get_library_games();

        for (i, g) in games.iter_mut().enumerate() {
            if g.id == game.id {
                games.remove(i);
                break;
            }
        }

        if let Err(e) = MONARCH_STATE.set_library_games(&games) {
            error!("games_library::remove_game() -> {}", e.chain().map(|e| e.to_string()).collect::<String>());
        }
    }

    let mut monarch_games = get_monarchgames().with_context(|| "games_library::remove_game() -> ")?;
    for (i, g) in monarch_games.iter_mut().enumerate() {
        if g.id == game.id {
            monarch_games.remove(i);
            break;
        }
    }
    write_monarch_games(&monarch_games)
}

/// Updates the properties of a game in the library.
pub fn update_game_properties(game: &MonarchGame) -> Result<()> {
    let games_json: Value =
        get_games().with_context(|| "games_library::update_game_properties() -> ")?;

    let mut games: Vec<MonarchGame> = serde_json::from_value(games_json).with_context(|| {
        "games_library::update_game_properties() Failed to parse json to Vec<MonarchGame>! | Err: "
    })?;

    for library_game in games.iter_mut() {
        if library_game.id == game.id {
            library_game.compatibility = game.compatibility.to_string();
            library_game.launch_args = game.launch_args.to_string();
            library_game.executable_path = game.executable_path.to_string();
            break;
        }
    }

    unsafe {
        MONARCH_STATE
            .update_game(&game)
            .with_context(|| "games_library::update_game_properties() -> ")?;
    }
    Ok(())
}
