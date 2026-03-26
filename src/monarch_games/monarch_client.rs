use super::games::{GameType, SearchResult};
use super::stores::{DownloadOptions, StoreType};
use super::{monarchgame::MonarchGame, steam_client};
use crate::monarch_games::monarchgame::{
    GameImageType, MonarchGameProperties, MonarchWebApiGame, StoreInfo,
};
use crate::monarch_games::stores::SearchFilter;
use crate::monarch_library::library::write_monarch_games;
use crate::monarch_utils::monarch_fs::{generate_cache_image_path, get_unix_home};
use crate::monarch_utils::monarch_settings::get_settings;
use crate::monarch_utils::monarch_state::MONARCH_STATE;
use crate::monarch_utils::{monarch_terminal, monarch_vdf};
use crate::{monarch_library::library, monarch_utils::monarch_fs};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{error, info, warn};

pub struct MonarchClient {}

impl MonarchClient {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl StoreType for MonarchClient {
    async fn search_games(&self, name: &str, _filter: &SearchFilter) -> Vec<Box<dyn SearchResult>> {
        let monarch_url: &'static str = std::env!("MONARCH_URL");
        let search_term: String = format!("{monarch_url}/api/games?search={}", name);
        let response = match reqwest::get(search_term).await {
            Ok(resp) => resp,
            Err(e) => {
                error!(
                    "monarch_client::search_games() reqwest::get() failed! | Err: {}",
                    e
                );
                return Vec::new();
            }
        };

        let resp_content = match response.text().await {
            Ok(content) => content,
            Err(e) => {
                error!(
                    "monarch_client::search_games() response.text() failed! | Err: {}",
                    e
                );
                return Vec::new();
            }
        };

        let mut web_games: Vec<Box<MonarchWebApiGame>> =
            match serde_json::from_str::<Vec<MonarchWebApiGame>>(&resp_content) {
                Ok(games) => games.into_iter().map(Box::new).collect(),
                Err(e) => {
                    error!(
                        "monarch_client::search_games() serde_json::from_str() failed! | Err: {}",
                        e
                    );
                    return Vec::new();
                }
            };

        for game in web_games.iter_mut() {
            let thumbnail_path = String::from(
                generate_cache_image_path(&game.name.clone(), GameImageType::Cover)
                    .to_str()
                    .unwrap(),
            );
            game.thumbnail_path = thumbnail_path;
        }

        web_games
            .into_iter()
            .map(|g| g as Box<dyn SearchResult>)
            .collect()
    }

    async fn install_game(&self, _game: &MonarchGame, _opts: &DownloadOptions) -> Result<()> {
        error!("monarch_client::install_game() Not implemented!");
        bail!("monarch_client::install_game() currently not supported!")
    }

    async fn uninstall_game(&self, _game: &MonarchGame) -> Result<()> {
        error!("monarch_client::uninstall_game() Not implemented!");
        bail!("monarch_client::uninstall_game() currently not supported!")
    }

    async fn update_game(&self, _game: &MonarchGame) -> Result<()> {
        error!("monarch_client::update_game() Not implemented!");
        bail!("monarch_client::update_game() currently not supported!")
    }

    fn game_is_installed(&self, _store_id: &str) -> bool {
        false
    }

    fn store_enabled(&self) -> bool {
        error!("monarch_client::store_enabled() Not implemented!");
        false
    }

    async fn launch_game(&self, game: &MonarchGame) -> Result<()> {
        game.launch().await
    }
}

#[cfg(target_os = "windows")]
use super::windows::steam;

#[cfg(target_os = "macos")]
use super::macos::steam;

#[cfg(target_os = "linux")]
use super::linux::steam;

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
pub async fn launch_game(frontend_game: &MonarchGame) -> Result<()> {
    /*
        if let Err(e) = hide_quicklaunch(handle) {
            warn!("monarch_client::launch_game() Error while hiding quicklaunch. Possibly already hidden. | Err: {e}");
        }
    */

    let mut game: MonarchGame;
    match MONARCH_STATE.read() {
        Ok(state) => {
            game = state
                .get_game(&frontend_game.id)
                .with_context(|| "monarch_client::launch_game() -> ")?;
        }
        Err(e) => {
            error!(
                "monarch_client::launch_game() Failed to lock on MONARCH_STATE | Err: {}",
                e
            );
            bail!(
                "monarch_client::launch_game() Failed to lock on MONARCH_STATE | Err: {}",
                e
            )
        }
    }

    // Check if game should be launched with exectutable, such as
    // the game binary or Proton executable
    if let Some(path) = &game.executable_path {
        info!("Launching game with executable path: {}", path);

        // Reformat the launch command to work on the store
        if cfg!(target_os = "windows") {
            game.executable_path = Some(format!(
                r#"Start-Process "{}""#,
                game.executable_path.unwrap()
            ));
        } else {
            game.executable_path = Some(game.executable_path.unwrap().replace(" ", "\\ "));
        }

        // Run with compatibility layer
        if game.compatibility.is_some() {
            if cfg!(not(target_os = "linux")) {
                bail!("monarch_client::launch_game() User tried launching a game using compatibility layer on OS other than Linux! | Err: Cannot use compatibility layer under anything other than Linux!")
            }

            #[cfg(target_os = "linux")]
            {
                use super::linux;
                return linux::umu::umu_run(&mut game).await;
            };
        }

        // Run without compatibility layer
        let launch_command: String = format!("{}", game.executable_path.unwrap_or_default());

        // Order launch args and command in proper order
        let full_command: String = if game
            .launch_args
            .clone()
            .unwrap_or_default()
            .find("%command%")
            .is_some()
        {
            warn!("Using Steam %command% style launch arguments!");
            game.launch_args
                .unwrap()
                .replace("%command%", &launch_command)
        } else {
            format!(
                "{} {}",
                launch_command,
                game.launch_args.unwrap_or_default()
            )
        };

        let rx = monarch_terminal::spawn_terminal(full_command, HashMap::new(), None);
        let _ = rx.await;

        return Ok(());
    }

    // Otherwise launch via store
    match game.get_store_name().as_str() {
        "steam" => {
            info!("Launching game via steam client: {}", game.get_store_id());
            steam_client::launch_client_game(&game)
                .with_context(|| "monarch_client::launch_game() -> ")
        }
        "steamcmd" => {
            info!("Launching game via steamcmd: {}", game.get_store_id());
            steam_client::launch_cmd_game(&game)
                .await
                .with_context(|| "monarch_client::launch_game() -> ")
        }
        &_ => {
            bail!("monarch_client::launch_game() User tried launching a game on an invalid store: {} | Err: Invalid store!", game.get_store_name())
        }
    }
}

/// Downloads a game into default folder
pub async fn download_game(name: &str, store: &str, store_id: &str) -> Result<Vec<MonarchGame>> {
    let settings_lock = match get_settings() {
        Ok(lock) => lock,
        Err(e) => {
            error!("monarch_client::download_game() Failed to get settings | Err: {e}");
            bail!("monarch_client::download_game() Failed to get settings | Err: {e}");
        }
    };
    let settings = match settings_lock.read() {
        Ok(settings) => settings,
        Err(e) => {
            error!(
                "monarch_client::download_game() Failed to get read lock on settings | Err: {e}"
            );
            bail!("monarch_client::download_game() Failed to get read lock on settings | Err: {e}");
        }
    };

    let mut path: PathBuf = PathBuf::from(&settings.monarch.game_folder);

    if !monarch_fs::path_exists(&path) {
        monarch_fs::create_dir(&path).with_context(|| "monarch_client::download_game() -> ")?;
    }

    path.push(name); // Game specific path
    if !monarch_fs::path_exists(&path) {
        monarch_fs::create_dir(&path).with_context(|| "monarch_client::download_game() -> ")?;
    }

    let new_game: MonarchGame = match store {
        "steam" => {
            // Check if steamcmd is installed
            if !steam_client::steamcmd_is_installed() {
                warn!("monarch_client::download_game() SteamCMD not found!");
                info!("Attempting to download and install SteamCMD...");

                steam_client::install_steamcmd()
                    .await
                    .with_context(|| "monarch_client::download_game() -> ")?;
            }

            let mut new_game = steam_client::download_game(name, store_id)
                .await
                .with_context(|| "monarch_client::download_game() -> ")?;

            new_game.stores.push(StoreInfo {
                name: "steamcmd".to_string(),
                store_id: store_id.to_string(),
                store_url: "".to_string(),
            });
            new_game
        }
        &_ => bail!("monarch_client::download_game() Invalid store!"),
    };

    library::add_game(&new_game).with_context(|| "monarch_client::download_game() -> ")?;

    Ok(get_library()) // Return new library
}

/// Remove an installed game
pub async fn uninstall_game(store: &str, store_id: &str) -> Result<()> {
    match store {
        "steam" => steam_client::uninstall_client_game(store_id),
        "steamcmd" => {
            steam_client::uninstall_game(store_id)
                .await
                .with_context(|| "monarch_client::uninstall_game() -> ")?;

            let mut monarch_games = library::get_monarchgames()
                .with_context(|| "monarch_client::uninstall_game() -> ")?;

            for (i, game) in monarch_games.clone().iter().enumerate() {
                if game.get_store_name() == store && game.get_store_id() == store_id {
                    monarch_games.remove(i);

                    match MONARCH_STATE.write() {
                        Ok(mut state) => {
                            state.set_library_games(&monarch_games);

                            // Replace games with the updated list of library games
                            monarch_games = state.get_library_games();
                        }
                        Err(e) => {
                            error!("monarch_client::uninstall_game() Failed to lock on MONARCH_STATE | Err: {}", e);
                        }
                    }
                    return write_monarch_games(&monarch_games)
                        .with_context(|| "monarch_client::uninstall_game() -> ");
                }
            }
            bail!("monarch_client::update_game() | Err: Game: {store_id} uninstalled, not removed from monarch_games.json, due to not found!")
        }

        &_ => bail!(
            "monarch_client::uninstall_game() | Err: Invalid store passed as argument ( {store} )"
        ),
    }
}

/// Update a game
pub async fn update_game(store: &str, store_id: &str) -> Result<()> {
    match store {
        "steam" => {
            bail!("monarch_client::uninstall_game() | Err: Monarch currently does not support updating games from the steam desktop client!")
        }
        "steamcmd" => steam_client::update_game(store_id)
            .await
            .with_context(|| "monarch_client::uninstall_game() -> "),
        &_ => bail!(
            "monarch_client::uninstall_game() | Err: Invalid store passed as argument ( {store} )"
        ),
    }
}

/// Returns games found in library.json
pub fn get_library() -> Vec<MonarchGame> {
    let mut games: Vec<MonarchGame> = Vec::new();
    match library::get_games() {
        Ok(library) => {
            games = library;
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

    if let Ok(mut monarch_games) = library::get_monarchgames() {
        games.append(&mut monarch_games);
    }

    let mut steam_games: Vec<MonarchGame> = steam_client::get_library().await;
    steam_games = steam_games
        .iter()
        .filter(|game| !games.contains(game))
        .cloned()
        .collect();

    games.append(&mut steam_games);

    match MONARCH_STATE.write() {
        Ok(mut state) => {
            state.set_library_games(&games);
            // Replace games with the updated list of library games
            games = state.get_library_games();
        }
        Err(e) => {
            error!(
                "monarch_client::refresh_library() Failed to lock on MONARCH_STATE | Err: {}",
                e
            );
        }
    }

    games
}

/// Search for the name of a game and return the results.
/// TODO: Add support for things like filters in the future.
/// TODO: Remove unwraps after testing
pub async fn find_games(search_term: &str) -> Vec<MonarchGame> {
    let monarch_url: &'static str = std::env!("MONARCH_URL");
    let search_term: String = format!("{monarch_url}/api/games?search={}", search_term);

    let response = reqwest::get(search_term).await.unwrap();
    let resp_content = response.text().await.unwrap();

    let web_games: Vec<MonarchWebApiGame> = serde_json::from_str(&resp_content).unwrap();

    let mut monarch_games: Vec<MonarchGame> = Vec::new();
    for game in web_games {
        let thumbnail_path = String::from(
            generate_cache_image_path(&game.name.clone(), GameImageType::Cover)
                .to_str()
                .unwrap(),
        );
        let mut new_monarchgame = MonarchGame::from(&game);
        new_monarchgame.thumbnail_path = thumbnail_path;
        monarch_games.push(new_monarchgame);
    }

    monarch_games
}

pub async fn get_game_properties(game: &mut MonarchGame) {
    let mut store = game.get_store_name();
    if store == "steamcmd" {
        store = "steam".to_string();
    }

    let mut properties: MonarchGameProperties = MonarchGameProperties::default();

    if game.is_installed() {
        if store == "steam" {
            match steam::get_default_libraryfolders_location() {
                Ok(p) => {
                    let mut props: MonarchGameProperties =
                        monarch_vdf::get_game_properties_from_manifest(game, &p).into();

                    #[cfg(target_os = "linux")]
                    {
                        match steam_client::get_protondb_rating(&game.get_store_id()).await {
                            Ok((rating, url)) => {
                                props.protondb_rating = rating;
                                props.protondb_url = url;
                            }
                            Err(e) => {
                                error!("monarch_client::get_game_properties() Failed to get ProtonDB rating! | Err: {}", e);
                            }
                        }
                    }

                    if let Ok(state) = MONARCH_STATE.read() {
                        if let Some(g) = state.get_game(&game.id) {
                            props.description = g.summary;
                        }
                    }
                    properties = props;
                }
                Err(e) => {
                    error!("monarch_client::get_game_properties() Failed to get path to Steams libraryfolders.vdf! | Err: {}", e);
                    return;
                }
            }
        }
    } else {
        let monarch_url: &'static str = std::env!("MONARCH_URL");
        let search_term: String = format!("{monarch_url}/api/games?id={}", game.id);
        let response = reqwest::get(search_term).await.unwrap();
        let resp_content = response.text().await.unwrap();
        let web_games: Vec<MonarchWebApiGame> = serde_json::from_str(&resp_content).unwrap();

        if !web_games.is_empty() {
            let web_game: &MonarchWebApiGame = &web_games[0];

            if game.stores.is_empty() {}
            properties.description = web_game.summary.to_string();

            for store in web_game.stores.iter() {
                if store.name == "steam" {}
            }

            #[cfg(target_os = "linux")]
            {
                for store in game.stores.iter() {
                    if store.name == "steam" {
                        match steam_client::get_protondb_rating(&store.store_id).await {
                            Ok((rating, url)) => {
                                properties.protondb_rating = rating;
                                properties.protondb_url = url;
                            }
                            Err(e) => {
                                error!("monarch_client::get_game_properties() Failed to get ProtonDB rating! | Err: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }

    game.properties = properties;
}
