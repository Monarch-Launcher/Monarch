use super::steam_client;
use crate::monarch_utils::monarch_fs;
use crate::monarch_utils::monarch_fs::get_appdata_path;
use std::path::PathBuf;
use log::{warn, error, info};

/// Temporary solutio until settings work 100%
/// Creates a directory to store games in
pub fn generate_default_folder() -> PathBuf {
    let mut path: PathBuf;
    
    if cfg!(windows) {
        path = PathBuf::from("C:\\")
    } else {
        path = get_appdata_path().unwrap() // Otherwise put games in Monarchs home folder
    }

    path.push("games");
    path
}

/// Launches a game
pub fn launch_game(platform: &str, platform_id: &str) -> Result<(), String> {
    match platform {
        "steam" => {
            return steam_client::launch_game(platform_id)
        }
        "steamcmd" => {
            return steam_client::launch_cmd_game(platform_id)
        }
        &_ => {
            error!("monarch_client::launch_game() failed! Invalid platform passed as argument: {platform}");
            return Err("Invalid platform!".to_string())
        }
    }
}

/// Downloads a game into default folder
pub async fn download_game(platform: &str, platform_id: &str) -> Result<(), String> {
    let path: PathBuf = generate_default_folder();

    if !monarch_fs::path_exists(&path) {
        monarch_fs::create_dir(&path).unwrap();
    }

    match platform {
        "steam" => {
            if !steam_client::is_installed() {
                warn!("monarch_client::download_game() SteamCMD not found!");
                info!("Attempting to download and install SteamCMD...");
                
                if let Err(e) = steam_client::download_and_install().await {
                    error!("monarch_client::download_game() failed! Error while installing SteamCMD! | Error: {e}");
                    return Err(e)
                }
            }
            steam_client::download_game(platform_id)
        }
        &_ => todo!(),
    }
}
