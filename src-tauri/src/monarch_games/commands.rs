use super::steam;
use super::blizzard;
use super::epic;

#[tauri::command]
/// Manually download Steam
async fn steam_downloader() {
    steam::get_steam().await;
}

#[tauri::command]
/// Launch a game via Steam
async fn launch_steam_game(name: &str) {
    steam::launch_game(name).unwrap();
}

#[tauri::command]
/// Search for games on Steam
async fn search_steam_games(name: &str) {
    steam::find_game(name).await.unwrap();
}

#[tauri::command]
/// Manually download Battle.net
async fn blizzard_downloader() {
    get_blizzard().await;
}

#[tauri::command]
async fn launch_blizzard_game(name: &str) {
    blizzard::launch_game(name).unwrap();
}

#[tauri::command]
/// Manually download Epic Games
async fn epic_downloader() {
    get_epic().await;
}