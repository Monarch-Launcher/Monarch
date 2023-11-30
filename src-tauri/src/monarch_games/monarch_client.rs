use super::{steam_client, monarchgame::MonarchGame};
use crate::{monarch_utils::monarch_fs, monarch_library::games_library};
use crate::monarch_utils::monarch_fs::get_appdata_path;
use log::{error, info, warn};
use std::path::PathBuf;

/// Temporary solutio until settings work 100%
/// Creates a directory to store games in
pub fn generate_default_folder() -> PathBuf {
    let mut path: PathBuf;

    if cfg!(windows) {
        path = PathBuf::from("C:\\")
    } else {
        path = get_appdata_path().unwrap() // Otherwise put games in Monarchs home folder
    }

    path.push("MonarchGames");
    path
}

/// Launches a game
pub async fn launch_game(platform: &str, platform_id: &str) -> Result<(), String> {
    match platform {
        "steam" => return steam_client::launch_game(platform_id),
        "steamcmd" => return steam_client::launch_cmd_game(platform_id).await,
        &_ => {
            error!("monarch_client::launch_game() failed! Invalid platform passed as argument: {platform}");
            return Err("Invalid platform!".to_string());
        }
    }
}

/// Downloads a game into default folder
pub async fn download_game(name: &str, platform: &str, platform_id: &str) -> Result<Vec<MonarchGame>, String> {
    let mut path: PathBuf = generate_default_folder(); // Install dir
    let new_game: MonarchGame;

    if !monarch_fs::path_exists(&path) {
        monarch_fs::create_dir(&path).unwrap();
    }

    path.push(name); // Game specific path
    if !monarch_fs::path_exists(&path) {
        monarch_fs::create_dir(&path).unwrap();
    }

    match platform {
        "steam" => {
            if !steam_client::is_installed() {
                warn!("monarch_client::download_game() SteamCMD not found!");
                info!("Attempting to download and install SteamCMD...");

                // Run async on windows
                if let Err(e) = steam_client::download_and_install().await {
                    error!("monarch_client::download_game() failed! Error while installing SteamCMD! | Error: {e}");
                    return Err(e);
                }
            }
            
            match steam_client::download_game(name, platform_id).await {
                Ok(game) => { new_game = game }
                Err(e) => {
                    error!("monarch_client::download_game() failed! Failed to download Steam game! | Error: {e}");
                    return Err("Failed to download Steam game!".to_string())
                }
            }
        }
        &_ => {
            error!("monarch_client::download_game() failed! Invalid platform passed as argument: {platform}");
            return Err("Invalid platform!".to_string());
        }
    }
    
    if let Err(e) = games_library::add_game(new_game) {
        error!("monarch_client::download_game() failed! Error while writing new MonarchGame to library.json! | Error: {e}");
        return Err("Failed to write new game to library.json!".to_string())
    }

    Ok(get_library().await) // Return new library
}

/// Remove an installed game
pub async fn uninstall_game(platform: &str, platform_id: &str) -> Result<(), String> {
    match platform {
        "steamcmd" => {
            steam_client::uninstall_game(platform_id).await
        }
        &_ => {
            error!("monarch_client::uninstall_game() failed! Invalid platform passed as argument: {platform}");
            return Err("Invalid platform!".to_string());
        }
    }
}

/// Returns installed games according to Monarch
pub async fn get_library() -> Vec<MonarchGame> {
    let mut games: Vec<MonarchGame> = Vec::new();
    let mut steam_games: Vec<MonarchGame> = steam_client::get_library().await;

    games.append(&mut steam_games);

    if let Err(e) = games_library::write_games(games.clone()) {
        error!(
            "Failed to write new games to library.json! | Message: {:?}",
            e
        );
    }
    games
}