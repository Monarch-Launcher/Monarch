use super::monarchgame::MonarchGame;
use super::steam;
use super::blizzard;
use super::epic;

/*
---------- General game related functions ----------
*/

#[tauri::command]
/// Search for games on Monarch, currently only support Steam search
pub async fn search_games(name: String) -> Vec<MonarchGame> {
    return steam::find_game(&name).await
}   

#[tauri::command]
/// Manually refreshes the entire Monarch library, currently only supports Steam, still WIP
pub async fn refresh_library() {
    steam::get_library().await;
}

#[tauri::command]
/// Launch a game
pub fn launch_game(game: MonarchGame) {
    match game.get_platform() {
        "steam" => { steam::launch_game(game); }
        "blizzard" => {}
        "epic" => {}
        "monarch" => {}
        _ => {}
    }
}

#[tauri::command]
/// Open "Download window" for a game
pub fn download_game(game: MonarchGame) {
    match game.get_platform() {
        "steam" => { steam::download_game(game); }
        "blizzard" => {}
        "epic" => {}
        "monarch" => {}
        _ => {}
    }
}

/*
---------- Steam related functions ----------
*/

#[tauri::command]
/// Manually download Steam
pub async fn steam_downloader() {
    steam::get_steam().await;
}

/*
---------- Blizzard related functions ----------
*/

#[tauri::command]
/// Manually download Battle.net
pub async fn blizzard_downloader() {
    blizzard::get_blizzard().await;
}

/*
---------- Epic Games related functions ----------
*/

#[tauri::command]
/// Manually download Epic Games
pub async fn epic_downloader() {
    epic::get_epic().await;
}