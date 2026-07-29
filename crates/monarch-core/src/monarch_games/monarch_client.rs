use super::games::{GameType, SearchResult};
use super::stores::{DownloadOptions, StoreType};
use super::{monarchgame::MonarchGame, steam_client};
use crate::monarch_games::egs_client::EgsClient;
use crate::monarch_games::monarchgame::{
    GameImageType, MonarchGameProperties, MonarchWebApiGame, StoreInfo,
};
use crate::monarch_games::stores::SearchFilter;
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

    library::add_game(&new_game)
        .await
        .with_context(|| "monarch_client::download_game() -> ")?;

    Ok(library::get_games().await.unwrap()) // Return new library
}

/// Remove an installed game
pub async fn uninstall_game(store: &str, store_id: &str) -> Result<()> {
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

/// Returns autodetected games according to Monarch
pub async fn refresh_library() -> Result<Vec<MonarchGame>> {
    info!("Manual refresh of library requested. Refreshing...");
    let games: Vec<MonarchGame> = library::get_games()
        .await
        .with_context(|| "monarch_client::refresh_library() -> ")?;

    let steam_games: Vec<MonarchGame> = steam_client::get_library().await;

    let mut egs_client: EgsClient = EgsClient::new();
    egs_client.load_existing_user().await.unwrap();
    let epic_games: Vec<MonarchGame> = egs_client.get_library().await;

    // Filter out removed games while keeping imported ones.
    // Deduplicate games by their Monarch ID to prevent UNIQUE constraint errors in database.
    let mut merged_games: HashMap<String, MonarchGame> = HashMap::new();

    // 1. First, populate with existing library games that are either imported
    // or still exist in steam_games or epic_games.
    for lg in games {
        let is_steam = steam_games.iter().any(|sg| *sg == lg || sg.id == lg.id);
        let is_epic = epic_games.iter().any(|eg| *eg == lg || eg.id == lg.id);
        if lg.imported || is_steam || is_epic {
            let mut updated_game = lg.clone();
            if is_steam {
                updated_game.is_installed = true;
            }
            merged_games.insert(updated_game.id.clone(), updated_game);
        }
    }

    // 2. Next, merge new Steam games, preserving properties if already present.
    for sg in steam_games {
        match merged_games.get_mut(&sg.id) {
            Some(existing) => {
                let mut updated_sg = sg.clone();
                updated_sg.properties = existing.properties.clone();
                updated_sg.launch_args = existing.launch_args.clone();
                updated_sg.compatibility = existing.compatibility.clone();
                updated_sg.imported = existing.imported || sg.imported;
                if updated_sg.executable_path.is_none() {
                    updated_sg.executable_path = existing.executable_path.clone();
                }
                *existing = updated_sg;
            }
            None => {
                merged_games.insert(sg.id.clone(), sg);
            }
        }
    }

    // 3. Finally, merge Epic games. Refresh asset IDs (catalog_id/app_name) from
    // the launcher assets list so install can use them without another API call.
    for eg in epic_games {
        match merged_games.get_mut(&eg.id) {
            Some(existing) => {
                if let Some(catalog_id) = eg.properties.other.get("catalog_id") {
                    existing
                        .properties
                        .other
                        .insert("catalog_id".to_string(), catalog_id.clone());
                }
                if let Some(app_name) = eg.properties.other.get("app_name") {
                    existing
                        .properties
                        .other
                        .insert("app_name".to_string(), app_name.clone());
                }
            }
            None => {
                merged_games.insert(eg.id.clone(), eg);
            }
        }
    }

    let final_games: Vec<MonarchGame> = merged_games.into_values().collect();

    library::overwrite_games(&final_games)
        .await
        .with_context(|| "monarch_client::refresh_library() -> ")?;

    Ok(final_games)
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

    // Preserve misc KV data (e.g. EGS catalog_id/app_name) written during library scan.
    let preserved_other = game.properties.other.clone();
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

    properties.other = preserved_other;
    game.properties = properties;

    // Keep process-local library cache in sync with computed/enriched properties.
    if let Ok(mut state) = MONARCH_STATE.write() {
        let _ = state.update_game(game.clone());
    }
}
