use super::monarchgame::MonarchGame;
use super::monarch_library;
use super::steam;
use super::blizzard;
use super::epic;

/*
---------- General game related functions ----------
*/

#[tauri::command]
/// Search for games on Monarch, currently only support Steam search
pub async fn search_games(name: String) -> Vec<MonarchGame> {
    let mut games: Vec<MonarchGame> = Vec::new();
    let mut steam_games: Vec<MonarchGame> = steam::find_game(&name).await;
    let mut epic_games: Vec<MonarchGame> = epic::find_game(&name).await;

    games.append(&mut steam_games);
    games.append(&mut epic_games);

    return games
}   

#[tauri::command]
/// Returns MonarchGames from library.json
pub async fn get_library() -> Vec<MonarchGame> {
    monarch_library::get_games()
}

#[tauri::command]
/// Manually refreshes the entire Monarch library, currently only supports Steam, still WIP
pub async fn refresh_library() -> Vec<MonarchGame> {
    let mut games: Vec<MonarchGame> = Vec::new();
    let mut steam_games: Vec<MonarchGame> = steam::get_library().await;

    games.append(&mut steam_games);

    monarch_library::write_games(games.clone());
    return games
}

#[tauri::command]
/// Launch a game
pub fn launch_game(name: String, id: String, platform: String) {
    match platform.as_str() {
        "steam" => { steam::launch_game(name.as_str(), id.as_str()); }
        "blizzard" => { blizzard::launch_game(name.as_str(), id.as_str()); }
        "epic" => {}
        "monarch" => {}
        _ => {}
    }
}

#[tauri::command]
/// Open "Download window" for a game
pub fn download_game(name: String, id: String, platform: String) {
    match platform.as_str() {
        "steam" => { steam::download_game(name.as_str(), id.as_str()); }
        "blizzard" => {}
        "epic" => {}
        "monarch" => {}
        _ => {}
    }
}

#[tauri::command]
/// Open "Purchase window" for a game
pub fn purchase_game(name: String, id: String, platform: String) {
    match platform.as_str() {
        "steam" => { steam::purchase_game(name.as_str(), id.as_str()); }
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