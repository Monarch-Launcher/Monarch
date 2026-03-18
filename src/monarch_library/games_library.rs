use anyhow::{Context, Result};
use serde_json::json;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use tracing::error;

use crate::monarch_games::monarchgame::{GameImageType, MonarchGame};
use crate::monarch_utils::monarch_fs::{
    generate_library_image_path, get_library_json_path, get_monarch_games_path, path_exists,
    write_json_content,
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
pub fn get_games() -> Result<Vec<MonarchGame>> {
    let path: PathBuf = get_library_json_path();

    let file: File = fs::File::open(&path).with_context(|| -> String {
        format!(
            "games_library::get_games() Error opening: {file} | Err",
            file = path.display()
        )
    })?;

    let games: Vec<MonarchGame> = serde_json::from_reader(file)
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

    let mut games: Vec<MonarchGame> = serde_json::from_reader(file).with_context(|| {
        "games_library::get_monarchgames() Could not parse json value as Vec<MonarchGame> | Err: "
    })?;

    // Fix for refreshing games with manually added games, which for some reason
    // have no thumbnail path.
    for game in games.iter_mut() {
        if game.thumbnail_path.is_empty() {
            game.thumbnail_path = generate_library_image_path(&game.name, GameImageType::Cover)
                .to_str()
                .unwrap()
                .to_string();
        }
    }

    for game in games.iter_mut() {
        if game.thumbnail_path.is_empty() {
            game.thumbnail_path = generate_library_image_path(&game.name, GameImageType::Artwork)
                .to_str()
                .unwrap()
                .to_string();
        }
    }

    Ok(games)
}

/// Backend functionality for adding a new game that's been installed.
pub fn add_game(game: &MonarchGame) -> Result<()> {
    let mut games: Vec<MonarchGame>;
    match MONARCH_STATE.write() {
        Ok(mut state) => {
            games = state.get_library_games();
            games.push(game.clone());
            state.set_library_games(&games);
        }
        Err(e) => {
            error!(
                "games_library::add_game() Failed to lock on MONARCH_STATE | Err: {}",
                e
            )
        }
    }
    write_monarchgame(game)
}

/// Backend functionality for removing a game from library.json
pub fn remove_game(game: &MonarchGame) -> Result<()> {
    let mut games: Vec<MonarchGame>;
    match MONARCH_STATE.write() {
        Ok(mut state) => {
            games = state.get_library_games();

            for (i, g) in games.iter_mut().enumerate() {
                if g.id == game.id {
                    games.remove(i);
                    break;
                }
            }

            state.set_library_games(&games);
        }
        Err(e) => {
            error!(
                "games_library::remove_game() Failed to lock on MONARCH_STATE | Err: {}",
                e
            )
        }
    }

    let mut monarch_games =
        get_monarchgames().with_context(|| "games_library::remove_game() -> ")?;
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
    match MONARCH_STATE.write() {
        Ok(mut state) => {
            state
                .update_game(&game)
                .with_context(|| "games_library::update_game_properties() -> ")?;
        }
        Err(e) => {
            error!(
                "games_library::update_game_properties() Failed to lock on MONARCH_STATE | Err: {}",
                e
            )
        }
    }
    Ok(())
}
