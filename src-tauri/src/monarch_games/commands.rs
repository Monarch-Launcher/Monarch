use super::monarch_client;
use super::monarchgame::MonarchGame;
use super::steam_client as steam;
use crate::monarch_library::games_library;
use crate::monarch_utils::monarch_miniwindow::MiniWindow;
use log::error;
use log::info;
use serde_json::value::Value;
use std::collections::HashMap;
use tauri::AppHandle;

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
pub async fn search_games(name: String) -> HashMap<String, MonarchGame> {
    let games: HashMap<String, MonarchGame> = steam::find_game(&name).await;
    return games
}

#[tauri::command]
/// Manually refreshes the entire Monarch library, currently only supports Steam & Epic Games (kinda) still WIP
pub async fn refresh_library() -> Vec<MonarchGame> {
    let mut games: Vec<MonarchGame> = Vec::new();
    let mut steam_games: Vec<MonarchGame> = steam::get_library().await;

    games.append(&mut steam_games);

    if let Err(e) = games_library::write_games(games.clone()) {
        error!(
            "Failed to write new games to library.json! | Message: {:?}",
            e
        );
    }
    games
}

#[tauri::command]
/// Launch a game
pub fn launch_game(name: String, platform: String, platform_id: String) -> Result<(), String> {
    info!("Launching game: {name}");
    monarch_client::launch_game(&platform, &platform_id)
}

#[tauri::command]
/// Tells Monarch to download specified game
pub async fn download_game(name: String, platform: String, platform_id: String) -> Result<(), String> {
    // For best user experience Monarch downloads all games by itself
    // instead of having to rely on 3rd party launchers.
    info!("Attempting to install: {name}");
    monarch_client::download_game(&platform, &platform_id).await
}

#[tauri::command]
/// Open "Purchase window" for a game
pub async fn open_store(url: String, handle: AppHandle) {
    let window: MiniWindow = MiniWindow::new("store", &url, 1280.0, 720.0);
    window.build_window(&handle).await.unwrap();
    window.show_window(&handle).unwrap();
}
