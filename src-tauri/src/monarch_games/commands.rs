use super::monarchgame::MonarchGame;
use super::steam;
use super::blizzard;
use super::epic;

#[tauri::command]
/// Manually download Steam
pub async fn steam_downloader() {
    steam::get_steam().await;
}

#[tauri::command]
/// Launch a game via Steam
pub async fn launch_steam_game(game: MonarchGame) {
    steam::launch_game(game).await;
}

#[tauri::command]
/// Search for games on Monarch, currently only support Steam search
pub async fn search_games(name: String) {
    steam::find_game(&name).await;
}

#[tauri::command]
/// Manually download Battle.net
pub async fn blizzard_downloader() {
    blizzard::get_blizzard().await;
}

#[tauri::command]
/// Launch a game via Battle.net
pub async fn launch_blizzard_game(game: MonarchGame) {
    blizzard::launch_game(game);
}

#[tauri::command]
/// Manually download Epic Games
pub async fn epic_downloader() {
    epic::get_epic().await;
}