use serde_json::value::Value;
use log::error;

#[cfg(target_os = "windows")]
use super::windows::{steam, blizzard, epic};

#[cfg(not(target_os = "windows"))]
use super::unix::{steam, blizzard, epic};

use super::monarchgame::MonarchGame;
use crate::monarch_library::games_library;

/*
---------- General game related functions ----------
*/

#[tauri::command]
/// Returns MonarchGames from library.json
pub async fn get_library() -> Result<Value, String> {
    games_library::get_games()
}

#[tauri::command]
/// Search for games on Monarch, currently only support Steam search
pub async fn search_games(name: String) -> Vec<MonarchGame> {
    let mut games: Vec<MonarchGame> = Vec::new();
    let mut steam_games: Vec<MonarchGame> = steam::find_game(&name).await;
    let mut blizz_games: Vec<MonarchGame> = blizzard::find_game(&name);
    
    games.append(&mut blizz_games);
    games.append(&mut steam_games);

    return games
}

#[tauri::command]
/// Manually refreshes the entire Monarch library, currently only supports Steam & Epic Games (kinda) still WIP
pub async fn refresh_library() -> Vec<MonarchGame> {
    let mut games: Vec<MonarchGame> = Vec::new();
    let mut steam_games: Vec<MonarchGame> = steam::get_library().await;
    let mut blizzard_games: Vec<MonarchGame> = blizzard::get_library().await;
    let mut epic_games: Vec<MonarchGame> = epic::get_library().await;

    games.append(&mut steam_games);
    games.append(&mut blizzard_games);
    games.append(&mut epic_games);

    if let Err(e) = games_library::write_games(games.clone()) {
        error!("Failed to write new games to library.json! | Message: {:?}", e);
    }
    return games;
}

#[tauri::command]
/// Launch a game
pub fn launch_game(name: String, platform_id: String, platform: String) {
    match platform.as_str() {
        "steam" => {
            steam::launch_game(name.as_str(), platform_id.as_str());
        }
        "blizzard" => {
            blizzard::launch_game(name.as_str(), platform_id.as_str());
        }
        "epic" => {}
        "monarch" => {}
        _ => {}
    }
}

#[tauri::command]
/// Open "Download window" for a game
pub fn download_game(name: String, platform_id: String, platform: String) {
    match platform.as_str() {
        "steam" => {
            steam::download_game(name.as_str(), platform_id.as_str());
        }
        "blizzard" => {}
        "epic" => {}
        "monarch" => {}
        _ => {}
    }
}

#[tauri::command]
/// Open "Purchase window" for a game
pub fn purchase_game(name: String, platform_id: String, platform: String) {
    match platform.as_str() {
        "steam" => {
            steam::purchase_game(name.as_str(), platform_id.as_str());
        }
        "blizzard" => {}
        "epic" => {}
        "monarch" => {}
        _ => {}
    }
}