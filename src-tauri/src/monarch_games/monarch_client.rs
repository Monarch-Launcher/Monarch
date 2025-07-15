use super::{monarchgame::MonarchGame, steam_client};
use crate::monarch_games::monarchgame::MonarchWebGame;
use crate::monarch_library::games_library::write_monarch_games;
use crate::monarch_utils::monarch_fs::{generate_cache_image_path, get_unix_home};
use crate::monarch_utils::monarch_settings::get_settings_state;
use crate::{monarch_library::games_library, monarch_utils::monarch_fs};
use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use tauri::AppHandle;
use tracing::{error, info, warn};

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
pub async fn launch_game(handle: &AppHandle, platform: &str, platform_id: &str) -> Result<()> {
    match platform {
        "steam" => steam_client::launch_client_game(platform_id),
        "steamcmd" => steam_client::launch_cmd_game(handle, platform_id).await,
        &_ => {
            bail!("monarch_client::launch_game() User tried launching a game on an invalid platform: {platform} | Err: Invalid platform!")
        }
    }
}

/// Downloads a game into default folder
pub async fn download_game(
    handle: &AppHandle,
    name: &str,
    platform: &str,
    platform_id: &str,
) -> Result<Vec<MonarchGame>> {
    let mut path: PathBuf = PathBuf::from(get_settings_state().monarch.game_folder);

    if !monarch_fs::path_exists(&path) {
        monarch_fs::create_dir(&path).with_context(|| "monarch_client::download_game() -> ")?;
    }

    path.push(name); // Game specific path
    if !monarch_fs::path_exists(&path) {
        monarch_fs::create_dir(&path).with_context(|| "monarch_client::download_game() -> ")?;
    }

    let new_game: MonarchGame = match platform {
        "steam" => {
            // Check if steamcmd is installed
            if !steam_client::is_installed() {
                warn!("monarch_client::download_game() SteamCMD not found!");
                info!("Attempting to download and install SteamCMD...");

                steam_client::download_and_install(handle)
                    .await
                    .with_context(|| "monarch_client::download_game() -> ")?;
            }

            steam_client::download_game(handle, name, platform_id)
                .await
                .with_context(|| "monarch_client::download_game() -> ")?
        }
        &_ => bail!("monarch_client::download_game() Invalid platform!"),
    };

    games_library::add_game(new_game).with_context(|| "monarch_client::download_game() -> ")?;

    Ok(get_library()) // Return new library
}

/// Remove an installed game
pub async fn uninstall_game(handle: &AppHandle, platform: &str, platform_id: &str) -> Result<()> {
    match platform {
        "steam" => {
            steam_client::uninstall_client_game(platform_id)
        }
        "steamcmd" => {
            steam_client::uninstall_game(handle, platform_id)
            .await
            .with_context(|| "monarch_client::uninstall_game() -> ")?;

            let mut monarch_games = games_library::get_monarchgames().with_context(|| "monarch_client::update_game() -> ")?;

            for (i, game) in monarch_games.clone().iter().enumerate() {
                if game.platform == platform && game.platform_id == platform_id {
                    monarch_games.remove(i);
                    return write_monarch_games(monarch_games).with_context(|| "monarch_client::update_game() -> ")
                }
            }
            bail!("monarch_client::update_game() | Err: Game: {platform_id} uninstalled, not removed from monarch_games.json, due to not found!")
        }

        &_ => bail!("monarch_client::uninstall_game() | Err: Invalid platform passed as argument ( {platform} )")
    }
}

/// Update a game
pub async fn update_game(handle: &AppHandle, platform: &str, platform_id: &str) -> Result<()> {
    match platform {
        "steam" => {
            bail!("monarch_client::uninstall_game() | Err: Monarch currently does not support updating games from the steam desktop client!")
        }
        "steamcmd" => {
            steam_client::update_game(handle, platform_id)
            .await
            .with_context(|| "monarch_client::uninstall_game() -> ")
        }
        &_ => bail!("monarch_client::uninstall_game() | Err: Invalid platform passed as argument ( {platform} )")
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
    steam_games = steam_games
        .iter()
        .filter(|game| !games.contains(game))
        .cloned()
        .collect();

    games.append(&mut steam_games);

    if let Err(e) = games_library::write_games(games.clone()) {
        error!("monarch_client::refresh_library() -> {e}");
    }
    games
}

/// Search for the name of a game and return the results.
/// TODO: Add support for things like filters in the future.
/// TODO: Remove unwraps after testing
pub async fn find_games(search_term: &str) -> Vec<MonarchGame> {
    let search_term: String = format!(
        "https://monarch-launcher.com/api/games?search={}",
        search_term
    );
    let response = reqwest::get(search_term).await.unwrap();
    let resp_content = response.text().await.unwrap();

    let web_games: Vec<MonarchWebGame> = serde_json::from_str(&resp_content).unwrap();

    let mut monarch_games: Vec<MonarchGame> = Vec::new();
    for game in web_games {
        let thumbnail_path = String::from(
            generate_cache_image_path(&game.name.clone())
                .to_str()
                .unwrap(),
        );
        let new_monarchgame = MonarchGame::new(
            &game.name,
            game.id,
            &game.platform,
            &game.platform_id,
            &game.store_page,
            "N/A",
            &thumbnail_path,
        );
        new_monarchgame.download_thumbnail(game.cover_url).await; // Do not await, this allows image to download concurrently as other monarchgames are parsed
        monarch_games.push(new_monarchgame);
    }

    monarch_games
}
