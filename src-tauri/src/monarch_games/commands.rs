use super::monarch_client;
use super::monarchgame::MonarchGame;
use super::steam_client as steam;
use crate::monarch_library::games_library;
use crate::monarch_utils::monarch_windows::MiniWindow;
use log::{error, info};
use serde_json::value::Value;
use tauri::AppHandle;

/*
---------- General game related functions ----------
*/

#[tauri::command]
/// Returns MonarchGames from library.json
pub async fn get_library() -> Result<Value, String> {
    match games_library::get_games() {
        Ok(games) => Ok(games),
        Err(e) => {
            error!("{:#}", e);
            Err(String::from("Failed to get library!"))
        }
    }
}

#[tauri::command]
/// Search for games on Monarch, currently only support Steam search
pub async fn search_games(name: String) -> Vec<MonarchGame> {
    let games: Vec<MonarchGame> = steam::find_game(&name).await;
    games
}

#[tauri::command]
/// Manually refreshes the entire Monarch library, currently only supports Steam & Epic Games (kinda) still WIP
pub async fn refresh_library() -> Vec<MonarchGame> {
    monarch_client::refresh_library().await
}

#[tauri::command]
/// Launch a game
pub async fn launch_game(
    name: String,
    platform: String,
    platform_id: String,
) -> Result<(), String> {
    info!("Launching game: {name}");
    if let Err(e) = monarch_client::launch_game(&platform, &platform_id).await {
        error!("{:#}", e);
        return Err(String::from("Failed to launch game!"));
    }
    Ok(())
}

#[tauri::command]
/// Tells Monarch to download specified game
pub async fn download_game(
    name: String,
    platform: String,
    platform_id: String,
) -> Result<Vec<MonarchGame>, String> {
    // For best user experience Monarch downloads all games by itself
    // instead of having to rely on 3rd party launchers.
    info!("Installing: {name}");
    match monarch_client::download_game(&name, &platform, &platform_id).await {
        Ok(new_library) => Ok(new_library),
        Err(e) => {
            error!("{:#}", e);
            Err(String::from("Failed to download game!"))
        }
    }
}

#[tauri::command]
/// Tells Monarch to remove specified game
pub async fn remove_game(
    name: String,
    platform: String,
    platform_id: String,
) -> Result<(), String> {
    info!("Uninstalling: {name}");
    if let Err(e) = monarch_client::uninstall_game(&platform, &platform_id).await {
        error!("{:#}", e);
        return Err(String::from("Failed to uninstall game!"));
    }
    Ok(())
}

#[tauri::command]
/// Open "Purchase window" for a game
pub async fn open_store(url: String, handle: AppHandle) {
    let window: MiniWindow = MiniWindow::new("store", &url, 1280.0, 720.0);
    window.build_window(&handle).await.unwrap();
    window.show_window(&handle).unwrap();
}
