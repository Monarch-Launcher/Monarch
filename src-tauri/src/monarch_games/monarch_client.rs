use super::{monarchgame::MonarchGame, steam_client};
use crate::monarch_utils::monarch_fs::get_unix_home;
use crate::monarch_utils::monarch_settings::get_monarch_settings;
use crate::{monarch_library::games_library, monarch_utils::monarch_fs};
use anyhow::{anyhow, Context, Result};
use log::{error, info, warn};
use std::path::PathBuf;

/// Generates the default path where Monarch wants to store games.
pub fn generate_default_folder() -> Result<PathBuf> {
    let path: PathBuf = if cfg!(windows) {
        // On windows, generate under C: drive
        PathBuf::from("C:\\")
    } else {
        // Otherwise put games in Monarchs home folder
        get_unix_home().unwrap()
    };

    Ok(path.join("MonarchGames"))
}

/// Launches a game
pub async fn launch_game(platform: &str, platform_id: &str) -> Result<()> {
    match platform {
        "steam" => return steam_client::launch_game(platform_id),
        "steamcmd" => return steam_client::launch_cmd_game(platform_id),
        &_ => {
            return Err(anyhow!(
                "monarch_client::launch_game() User tried launching a game on an invalid platform: {platform} | Err: Invalid platform!"
            ));
        }
    }
}

/// Downloads a game into default folder
pub async fn download_game(
    name: &str,
    platform: &str,
    platform_id: &str,
) -> Result<Vec<MonarchGame>> {
    let mut path: PathBuf = PathBuf::from(
        get_monarch_settings().unwrap()["game_folder"]
            .to_string()
            .trim_matches('"'),
    );
    let new_game: MonarchGame;

    if !monarch_fs::path_exists(&path) {
        monarch_fs::create_dir(&path).context("monarch_client::download_game() -> ".to_string())?;
    }

    path.push(name); // Game specific path
    if !monarch_fs::path_exists(&path) {
        monarch_fs::create_dir(&path).context("monarch_client::download_game() -> ".to_string())?;
    }

    match platform {
        "steam" => {
            if !steam_client::is_installed() {
                warn!("monarch_client::download_game() SteamCMD not found!");
                info!("Attempting to download and install SteamCMD...");

                // Run async on windows
                steam_client::download_and_install()
                    .await
                    .with_context(|| "monarch_client::download_game() -> ")?;
            }

            match steam_client::download_game(name, platform_id).await {
                Ok(game) => new_game = game,
                Err(e) => {
                    return Err(anyhow!("monarch_client::download_game() -> {e}"));
                }
            }
        }
        &_ => {
            return Err(anyhow!("monarch_client::download_game() Invalid platform!"));
        }
    }

    games_library::add_game(new_game).with_context(|| "monarch_client::download_game() -> ")?;

    Ok(get_library()) // Return new library
}

/// Remove an installed game
pub async fn uninstall_game(platform: &str, platform_id: &str) -> Result<()> {
    match platform {
        "steam" => steam_client::uninstall_game(platform_id)
            .await
            .context("monarch_client::uninstall_game() -> "),
        &_ => {
            return Err(anyhow!(
                "monarch_client::uninstall_game() Invalid platform passed as argument!"
            ));
        }
    }
}

/// Returns games found in library.json
fn get_library() -> Vec<MonarchGame> {
    let mut games: Vec<MonarchGame> = Vec::new();
    match games_library::get_games() {
        Ok(library_json) => {
            if let Ok(library) = serde_json::from_value::<Vec<MonarchGame>>(library_json) {
                games = library;
            }
        }
        Err(e) => {
            error!("monarch_client::get_library() -> {e}");
        }
    }

    games
}

/// Returns autodetected games according to Monarch
pub async fn refresh_library() -> Vec<MonarchGame> {
    info!("Manual refresh of library requested. Refreshing...");
    let mut games: Vec<MonarchGame> = Vec::new();

    if let Ok(mut monarch_games) = games_library::get_monarchgames() {
        games.append(&mut monarch_games);
    }

    let mut steam_games: Vec<MonarchGame> = steam_client::get_library().await;
    games.append(&mut steam_games);

    if let Err(e) = games_library::write_games(games.clone()) {
        error!("monarch_client::refresh_library() -> {e}");
    }
    games
}
