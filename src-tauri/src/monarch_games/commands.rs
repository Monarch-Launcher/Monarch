use super::monarchgame::MonarchGame;
use super::{monarch_client, steam_client};
use anyhow::Result;
use rand::rng;
use rand::seq::SliceRandom;
use serde_json::value::Value;
use tauri::AppHandle;
use tracing::{error, info};

use crate::monarch_library::games_library;
use crate::monarch_utils::monarch_vdf::{get_proton_versions, ProtonVersion};
use crate::monarch_utils::monarch_windows::MiniWindow;


#[cfg(target_os = "windows")]
use super::windows::steam;

#[cfg(target_os = "macos")]
use super::macos::steam;

#[cfg(target_os = "linux")]
use super::linux::steam;

/*
---------- General game related functions ----------
*/

#[tauri::command]
/// Returns MonarchGames from library.json
pub async fn get_library() -> Result<Value, String> {
    match games_library::get_games() {
        Ok(games) => Ok(games),
        Err(e) => {
            error!(
                "monarch_games::commands::get_library -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            Err(String::from("Something went wrong getting library!"))
        }
    }
}

#[tauri::command]
pub async fn get_home_recomendations() -> Result<Value, String> {
    match games_library::get_games() {
        Ok(games) => {
            let mut games_vec: Vec<MonarchGame> = serde_json::from_value(games.clone()).unwrap();
            if games_vec.len() > 4 {
                games_vec.shuffle(&mut rng());
                let recomended_games: &[MonarchGame] = &games_vec[0..4];
                Ok(serde_json::to_value(recomended_games).unwrap_or_default())
            } else {
                return Ok(games);
            }
        }
        Err(e) => {
            error!(
                "monarch_games::commands::get_library -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            Err(String::from("Something went wrong getting library!"))
        }
    }
}

#[tauri::command]
/// Search for games on Monarch, currently only support Steam search
pub async fn search_games(name: String, useMonarch: bool) -> Vec<MonarchGame> {
    if useMonarch {
        monarch_client::find_games(&name).await
    } else {
        steam_client::find_game(&name).await
    }
}

#[tauri::command]
/// Manually refreshes the entire Monarch library, currently only supports Steam & Epic Games (kinda) still WIP
pub async fn refresh_library() -> Vec<MonarchGame> {
    monarch_client::refresh_library().await
}

#[tauri::command]
/// Launch a game
pub async fn launch_game(handle: AppHandle, mut game: MonarchGame) -> Result<(), String> {
    info!("Launching game: {}", game.name);
    if let Err(e) = monarch_client::launch_game(&handle, &mut game).await {
        error!(
            "monarch_games::commands::launch_game() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(format!(
            "Something went wrong while launching: {}",
            game.name
        ));
    }
    Ok(())
}

#[tauri::command]
/// Tells Monarch to download specified game
pub async fn download_game(
    handle: AppHandle,
    name: String,
    platform: String,
    platform_id: String,
) -> Result<Vec<MonarchGame>, String> {
    // For best user experience Monarch downloads all games by itself
    // instead of having to rely on 3rd party launchers.
    info!("Installing: {name}");
    match monarch_client::download_game(&handle, &name, &platform, &platform_id).await {
        Ok(new_library) => Ok(new_library),
        Err(e) => {
            error!(
                "monarch_games::commands::download_game() -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            Err(format!("Something went wrong while downloading: {name}"))
        }
    }
}

#[tauri::command]
/// Tells Monarch to download specified game
pub async fn update_game(
    handle: AppHandle,
    name: String,
    platform: String,
    platform_id: String,
) -> Result<(), String> {
    info!("Updating: {name}");
    match monarch_client::update_game(&handle, &platform, &platform_id).await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!(
                "monarch_games::commands::check_for_game_update() -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            Err(format!("Something went wrong while updating: {name} \nMake sure game is installed via Monarch if you want to update."))
        }
    }
}

#[tauri::command]
/// Tells Monarch to remove specified game
pub async fn remove_game(
    handle: AppHandle,
    name: String,
    platform: String,
    platform_id: String,
) -> Result<(), String> {
    info!("Uninstalling: {name}");
    if let Err(e) = monarch_client::uninstall_game(&handle, &platform, &platform_id).await {
        error!(
            "monarch_games::commands::remove_game() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(format!("Something went wrong while removing: {name}"));
    }
    Ok(())
}

#[tauri::command]
pub async fn move_game_to_monarch(
    handle: AppHandle,
    name: String,
    platform: String,
    platform_id: String,
) -> Result<(), String> {
    info!("Moving {name} from {platform} to Monarch...");

    // First remove the game from old platform
    if let Err(e) = monarch_client::uninstall_game(&handle, &platform, &platform_id).await {
        error!(
            "monarch_games::commands::move_game_to_monarch() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(format!("Something went wrong while removing: {name}"));
    }

    // Then reinstall on Monarch
    if let Err(e) = monarch_client::download_game(&handle, &name, &platform, &platform_id).await {
        error!(
            "monarch_games::commands::move_game_to_monarch() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(format!("Something went wrong while downloading: {name}"));
    }

    info!("Finished moving {name} to Monarch");
    Ok(())
}

#[tauri::command]
/// Open "Purchase window" for a game
pub async fn open_store(url: String, handle: AppHandle) -> Result<(), String> {
    let window: MiniWindow = MiniWindow::new("store", &url, 1280.0, 720.0);
    if let Err(e) = window.build_window(&handle).await {
        error!(
            "monarch_games::commands::open_store() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from(
            "Something went wrong while opening store page!",
        ));
    }

    if let Err(e) = window.show_window(&handle) {
        error!(
            "monarch_games::commands::open_store() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from(
            "Something went wrong while opening store page!",
        ));
    }
    Ok(())
}

#[tauri::command]
/// Updates the properties of a game in the library.
pub async fn update_game_properties(game: MonarchGame) -> Result<(), String> {
    match games_library::update_game_properties(&game) {
        Ok(_) => Ok(()),
        Err(e) => {
            error!(
                "monarch_games::commands::update_game_properties() -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            Err(String::from(
                "Something went wrong while updating game properties!",
            ))
        }
    }
}

#[tauri::command]
pub fn proton_versions() -> Result<Vec<ProtonVersion>, String> {
    #[cfg(not(target_os = "linux"))]
    return Ok(vec![]);

    // Get libraryfolders.vdf
    let library_path = match steam::get_default_libraryfolders_location() {
        Ok(p) => p,
        Err(e) => {
            error!(
                "monarch_games::commands::proton_versions() -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            return Err(String::from(
                "Something went wrong while getting proton versions!",
            ));
        }
    };

    // Then get proton versions
    match get_proton_versions(&library_path) {
        Ok(p) => Ok(p),
        Err(e) => {
            error!(
                "monarch_games::commands::proton_versions() -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            Err(String::from(
                "Something went wrong while getting proton versions!",
            ))
        }
    }
}
