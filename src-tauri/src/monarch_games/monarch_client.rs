use super::games::GameType;
use super::stores::StoreType;
use super::{monarchgame::MonarchGame, steam_client};
use crate::monarch_games::monarchgame::MonarchWebGame;
use crate::monarch_library::games_library::write_monarch_games;
use crate::monarch_utils::monarch_fs::{generate_cache_image_path, get_unix_home};
use crate::monarch_utils::monarch_settings::get_settings_state;
use crate::monarch_utils::monarch_state::MONARCH_STATE;
use crate::{monarch_library::games_library, monarch_utils::monarch_fs};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use tauri::AppHandle;
use tracing::{error, info, warn};

pub struct MonarchClient {}

impl MonarchClient {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl StoreType for MonarchClient {
    async fn search_games(&self, name: &str) -> Vec<Box<dyn GameType>> {
        let search_term: String = format!("https://monarch-launcher.com/api/games?search={}", name);
        let response = match reqwest::get(search_term).await {
            Ok(resp) => resp,
            Err(e) => {
                error!("monarch_client::search_games() reqwest::get() failed! | Err: {}", e);
                return Vec::new();
            }
        };

        let resp_content = match response.text().await {
            Ok(content) => content,
            Err(e) => {
                error!("monarch_client::search_games() response.text() failed! | Err: {}", e);
                return Vec::new();
            }
        };

        let web_games: Vec<MonarchWebGame> = match serde_json::from_str(&resp_content) {
            Ok(games) => games,
            Err(e) => {
                error!("monarch_client::search_games() serde_json::from_str() failed! | Err: {}", e);
                return Vec::new();
            }
        };

        let mut monarch_games: Vec<Box<dyn GameType>> = Vec::new();
        for game in web_games {
            let thumbnail_path = String::from(
                generate_cache_image_path(&game.name.clone())
                    .to_str()
                    .unwrap(),
            );
            let mut new_monarchgame = MonarchGame::from(&game);
            new_monarchgame.thumbnail_path = thumbnail_path;
            monarch_games.push(Box::new(new_monarchgame));
        }
        monarch_games
    }

    async fn install_game(&self, _handle: &AppHandle, _game: &MonarchGame) -> Result<()> {
        error!("monarch_client::install_game() Not implemented!");
        bail!("monarch_client::install_game() currently not supported!")
    }

    async fn uninstall_game(&self, _handle: &AppHandle, _game: &MonarchGame) -> Result<()> {
        error!("monarch_client::uninstall_game() Not implemented!");
        bail!("monarch_client::uninstall_game() currently not supported!")
    }

    async fn update_game(&self, _handle: &AppHandle, _game: &MonarchGame) -> Result<()> {
        error!("monarch_client::update_game() Not implemented!");
        bail!("monarch_client::update_game() currently not supported!")
    }

    fn game_is_installed(&self, _handle: &AppHandle, _platform_id: &str) -> bool {
        error!("monarch_client::game_is_installed() Not implemented!");
        false
    }

    fn platform_enabled(&self) -> bool {
        error!("monarch_client::platform_enabled() Not implemented!");
        false
    }

    async fn launch_game(&self, handle: &AppHandle, game: &MonarchGame) -> Result<()> {
        game.launch(handle).await
    }
}

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
            if !steam_client::steamcmd_is_installed() {
                warn!("monarch_client::download_game() SteamCMD not found!");
                info!("Attempting to download and install SteamCMD...");

                steam_client::install_steamcmd(handle)
                    .await
                    .with_context(|| "monarch_client::download_game() -> ")?;
            }

            let mut new_game = steam_client::download_game(handle, name, platform_id)
                .await
                .with_context(|| "monarch_client::download_game() -> ")?;
            new_game.platform = "steamcmd".to_string();
            new_game
        }
        &_ => bail!("monarch_client::download_game() Invalid platform!"),
    };

    games_library::add_game(&new_game).with_context(|| "monarch_client::download_game() -> ")?;

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

            let mut monarch_games = games_library::get_monarchgames().with_context(|| "monarch_client::uninstall_game() -> ")?;

            for (i, game) in monarch_games.clone().iter().enumerate() {
                if game.platform == platform && game.platform_id == platform_id {
                    monarch_games.remove(i);
                    unsafe {
                        MONARCH_STATE.set_library_games(&monarch_games).with_context(|| "monarch_client::uninstall_game() -> ")?;

                        // Replace games with the updated list of library games
                        monarch_games = MONARCH_STATE.get_library_games();
                    }
                    return write_monarch_games(&monarch_games).with_context(|| "monarch_client::uninstall_game() -> ")
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
pub fn get_library() -> Vec<MonarchGame> {
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

    unsafe {
        if let Err(e) = MONARCH_STATE.set_library_games(&games) {
            error!(
                "monarch_client::refresh_library() -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            )
        }

        // Replace games with the updated list of library games
        games = MONARCH_STATE.get_library_games();
    }

    games
}
