use super::games::{GameType, SearchResult};
use super::stores::{DownloadOptions, StoreType};
use super::{monarchgame::MonarchGame, steam_client};
use crate::monarch_games::egs_client::EgsClient;
use crate::monarch_games::monarchgame::{
    GameImageType, MonarchGameProperties, MonarchWebApiGame, StoreInfo,
};
use crate::monarch_games::stores::SearchFilter;
use crate::monarch_utils::monarch_fs::{generate_cache_image_path, get_unix_home};
use crate::monarch_utils::monarch_settings::Settings;
use crate::monarch_utils::monarch_state::MonarchState;
use crate::monarch_utils::{monarch_http, monarch_sql, monarch_terminal, monarch_vdf};
use crate::{monarch_library::library, monarch_utils::monarch_fs};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::f32::consts::E;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::{error, info, warn};

pub struct MonarchClient {}

impl MonarchClient {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl StoreType for MonarchClient {
    async fn search_games(&self, settings_handle: Arc<RwLock<Settings>>, name: &str, _filter: &SearchFilter) -> Vec<Box<dyn SearchResult>> {
        let monarch_url: &'static str = std::env!("MONARCH_URL");
        let search_term: String = format!("{monarch_url}/api/games?search={}", name);
        let response = match monarch_http::client().get(search_term).send().await {
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
                generate_cache_image_path(settings_handle.clone(), &game.name.clone(), GameImageType::Cover)
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

    async fn install_game(&self, _game: &mut MonarchGame, _opts: &DownloadOptions) -> Result<()> {
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

    async fn launch_game(&mut self, game: &MonarchGame) -> Result<()> {
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
pub async fn launch_game(_state_handle: Arc<RwLock<MonarchState>>, _game_handle: Arc<RwLock<MonarchGame>>) -> Result<()> {
    /*
    let full_command: String;

    match game_handle.write() {
        Ok(mut game) => {
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
                }
            }
        Err(e) => {
            bail!("")
        }
    }

    let rx = monarch_terminal::spawn_terminal(full_command, HashMap::new(), None);
    let _ = rx.await;
     */

    return Ok(());
}

/// Downloads a game into default folder
pub async fn download_game(_name: &str, _store: &str, _store_id: &str) -> Result<()> {
    /*
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

    library::add_game(&new_game)
        .await
        .with_context(|| "monarch_client::download_game() -> ")?;

    Ok(library::get_games().await.unwrap()) // Return new library
*/
    Ok(())
}

/// Remove an installed game
pub async fn uninstall_game(_store: &str, _store_id: &str) -> Result<()> {
    /*
    match store {
        "steam" => steam_client::uninstall_client_game(store_id),
        "steamcmd" => {
            steam_client::uninstall_game(store_id)
                .await
                .with_context(|| "monarch_client::uninstall_game() -> ")?;

            /*
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
                */

            let games: Vec<MonarchGame>;
            match MONARCH_STATE.read() {
                Ok(state) => {
                    games = state.get_library_games();
                }
                Err(e) => {
                    bail!(
                        "monarch_client::uninstall_game() Failed to lock on MONARCH_STATE! | Err: {e}"
                    )
                }
            }

            for game in games.iter() {
                if game.get_store_name() == store && game.get_store_id() == store_id {
                    return library::remove_game(game)
                        .await
                        .with_context(|| "monarch_client::uninstall_game() -> ");
                }
            }
            bail!("monarch_client::uninstall_game() Failed to remove game from library! | Err: Not found!")
        }

        &_ => bail!(
            "monarch_client::uninstall_game() | Err: Invalid store passed as argument ( {store} )"
        ),
    }
 */
    Ok(())
}

/// Update a game
pub async fn update_game(_store: &str, _store_id: &str) -> Result<()> {
    Ok(())
}

/// Returns autodetected games according to Monarch
pub async fn refresh_library(state_handle: Arc<RwLock<MonarchState>>) -> Result<()> {
    info!("Manual refresh of library requested. Refreshing...");

    let mut games: Vec<Arc<RwLock<MonarchGame>>>;
    match state_handle.read() {
        Ok(state) => {
            games = state.get_library_games().to_vec();
        }
        Err(e) => {
            bail!("")
        }
    }

    let mut steam_games: Vec<MonarchGame> = steam_client::get_library().await;

    let mut egs_client: EgsClient = EgsClient::new();
    egs_client.load_existing_user().await.unwrap();
    let mut epic_games: Vec<MonarchGame> = egs_client.get_library().await;

    // Filter out removed games
    games.iter_mut()
        .filter(|game_handle| match game_handle.read() {
            Ok(game) => {
                match game.get_store_name().as_str() {
                    "steam" => {
                        for steam_game in steam_games.iter() {
                            if game.id == steam_game.id {
                                return true
                            }
                        }
                        false
                    }
                    "epicgames" => {
                        for epic_game in epic_games.iter() {
                            if game.id == epic_game.id {
                                return true
                            }
                        }
                        false
                    }
                    _ => {
                        return true
                    }
                }
            }
            Err(e) => {
                true
            }
        })
        .map(|game_handle| game_handle.clone())
        .collect::<Vec<Arc<RwLock<MonarchGame>>>>();

    // Add new Steam games
    for steam_game in steam_games.iter_mut() {
        let mut game_found: bool = false;
        for game_handle in games.iter() {
            if let Ok(mut game) = game_handle.write() {
                if game.id == steam_game.id {
                    steam_game.imported = game.imported.clone();
                    steam_game.properties = game.properties.clone();
                    steam_game.launch_args = game.launch_args.clone();
                    steam_game.compatibility = game.compatibility.clone();
                    if steam_game.executable_path.is_none() {
                        steam_game.executable_path = game.executable_path.clone();
                    }
                    *game = steam_game.clone();
                    game_found = true;
                    break
                }
            }
        }
        if !game_found {
            games.push(Arc::new(RwLock::new(steam_game.clone())));
        }
    }

    // Add new Epic games
    for epic_game in epic_games.iter_mut() {
        let mut game_found: bool = false;
        for game_handle in games.iter() {
            if let Ok(mut game) = game_handle.write() {
                if game.id == epic_game.id {
                    epic_game.imported = game.imported.clone();
                    epic_game.properties = game.properties.clone();
                    epic_game.launch_args = game.launch_args.clone();
                    epic_game.compatibility = game.compatibility.clone();
                    if epic_game.executable_path.is_none() {
                        epic_game.executable_path = game.executable_path.clone();
                    }
                    *game = epic_game.clone();
                    game_found = true;
                    break
                }
            }
        }
        if !game_found {
            games.push(Arc::new(RwLock::new(epic_game.clone())));
        }
    }
    
    let db_pool: Arc<SqlitePool>;
    match state_handle.write() {
        Ok(mut state) => {
            state.set_library_games(&games);
            db_pool = state.get_db_pool_arc();
        }
        Err(e) => {
            bail!("")
        }
    }

    for game_handle in games.iter() {
        let game_clone: MonarchGame;
        match game_handle.read() {
            Ok(game) => {
                game_clone = game.clone();
            }
            Err(e) => {
                error!("");
                continue;
            }
        }
        if let Err(e) = monarch_sql::update_game(&db_pool, &game_clone).await {
            error!("");
        }
    }

    Ok(())
}

/// Search for the name of a game and return the results.
/// TODO: Add support for things like filters in the future.
/// TODO: Remove unwraps after testing
pub async fn find_games(search_term: &str) -> Vec<MonarchGame> {
    let monarch_url: &'static str = std::env!("MONARCH_URL");
    let search_term: String = format!("{monarch_url}/api/games?search={}", search_term);

    let response = monarch_http::client()
        .get(search_term)
        .send()
        .await
        .unwrap();
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

pub async fn get_game_properties(state_handle: Arc<RwLock<MonarchState>>, game: &mut MonarchGame) {
    let mut store = game.get_store_name();
    if store == "steamcmd" {
        store = "steam".to_string();
    }

    // Start from the existing record so enrichment never wipes known install
    // metadata (install_dir / size / version) for stores without a local
    // property source — e.g. Epic Games installs managed by Monarch.
    let preserved_other = game.properties.other.clone();
    let mut properties = game.properties.clone();

    // Heal installs whose install_dir was previously wiped to the default
    // sentinel by older property enrichment.
    if game.is_installed()
        && game.managed_by_monarch
        && (properties.install_dir.is_empty() || properties.install_dir == "Error")
    {
        if let Ok(settings_lock) = get_settings() {
            if let Ok(settings) = settings_lock.read() {
                let fallback = PathBuf::from(&settings.monarch.game_folder).join(&game.name);
                if fallback.is_dir() {
                    info!(
                        "monarch_client::get_game_properties() Recovered install_dir for {}: {}",
                        game.name,
                        fallback.display()
                    );
                    properties.install_dir = fallback.to_string_lossy().to_string();
                }
            }
        }
    }

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
        } else if properties.description.is_empty() || properties.description == "Error" {
            // Non-Steam installs: keep install metadata; only fill description
            // from the game summary when it is still a sentinel/empty value.
            if !game.summary.is_empty() {
                properties.description = game.summary.clone();
            }
        }
    } else {
        let monarch_url: &'static str = std::env!("MONARCH_URL");
        let search_term: String = format!("{monarch_url}/api/games?id={}", game.id);
        let response = match monarch_http::client().get(search_term).send().await {
            Ok(resp) => resp,
            Err(e) => {
                error!(
                    "monarch_client::get_game_properties() Failed to fetch game metadata! | Err: {e}"
                );
                return;
            }
        };
        let resp_content = match response.text().await {
            Ok(text) => text,
            Err(e) => {
                error!(
                    "monarch_client::get_game_properties() Failed to read game metadata response! | Err: {e}"
                );
                return;
            }
        };
        let web_games: Vec<MonarchWebApiGame> = match serde_json::from_str(&resp_content) {
            Ok(games) => games,
            Err(e) => {
                error!(
                    "monarch_client::get_game_properties() Failed to parse game metadata! | Err: {e}"
                );
                return;
            }
        };

        if !web_games.is_empty() {
            let web_game: &MonarchWebApiGame = &web_games[0];
            properties.description = web_game.summary.to_string();

            #[cfg(target_os = "linux")]
            {
                for store_info in game.stores.iter() {
                    if store_info.name == "steam" {
                        match steam_client::get_protondb_rating(&store_info.store_id).await {
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

    properties.other = preserved_other;
    game.properties = properties;

    // Persist enriched properties to SQLite (and refresh the process-local
    // cache inside update_game_properties). Startup rebuilds MONARCH_STATE
    // from the database, so without this the enrichment is lost on restart.
    if let Err(e) = library::update_game_properties(game).await {
        error!(
            "monarch_client::get_game_properties() Failed to persist game properties! | Err: {e}"
        );
    }
}
