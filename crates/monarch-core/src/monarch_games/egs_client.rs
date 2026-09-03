use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use tokio::task::{self, JoinHandle};
use tracing::{error, info, warn};

use monarch_egs::{
    check_platform_support, get_game_manifest, Manifest, Session, SupportedPlatforms, User,
};

use crate::monarch_games::games::SearchResult;
use crate::monarch_games::monarchgame::{GameImageType, MonarchGame, MonarchWebApiGame};
use crate::monarch_games::stores::DownloadOptions;
use crate::monarch_games::stores::SearchFilter;
use crate::monarch_utils::monarch_fs::{
    generate_cache_image_path, generate_library_image_path, get_monarch_home,
};
use crate::monarch_utils::monarch_game_downloader::DownloadJob;
use crate::monarch_utils::monarch_http;
use crate::monarch_utils::monarch_settings::get_settings;
use crate::monarch_utils::monarch_state::MONARCH_STATE;

use super::stores::StoreType;

#[cfg(target_os = "linux")]
use super::linux::egs;

#[cfg(target_os = "linux")]
use super::linux::umu::umu_run;

#[cfg(target_os = "windows")]
use super::windows::egs;

pub struct EgsClient {
    user: User,
}

impl EgsClient {
    pub fn new() -> Self {
        Self { user: User::new() }
    }

    // Gets the existing user from monarch_egs.json file
    pub async fn load_existing_user(&mut self) -> Result<()> {
        let session: Session = self
            .load_session_from_file()
            .with_context(|| "egs::load_existing_user() -> ")?;

        self.user = User::load_stored_user(session).await;

        match get_settings() {
            Ok(settings_lock) => match settings_lock.read() {
                Ok(settings) => {
                    self.user.set_display_name(&settings.epic.username);
                }
                Err(e) => {
                    error!(
                        "egs::load_existing_user() settings_lock.read() failed! | Err: {}",
                        e
                    );
                }
            },
            Err(e) => {
                error!(
                    "egs::load_existing_user() get_settings() failed! | Err: {}",
                    e
                );
            }
        }

        Ok(())
    }

    /// Checks which platforms are supported for a game by its namespace.
    pub async fn check_platform_support(&self, namespace: &str) -> Result<SupportedPlatforms> {
        check_platform_support(&self.user, namespace)
            .await
            .with_context(|| {
                "egs_client::check_platform_support() Failed to check platform support"
            })
    }

    pub fn credentials_exist(&self) -> bool {
        Self::get_epic_games_token_path().exists()
    }

    pub fn open_epic_login(&self) {
        info!("User logging into Epic Games...");
        self.user.start_auth();
    }

    pub async fn save_epic_auth_code(&mut self, code: &str) -> Result<()> {
        info!("Logging in using Epic Games auth code...");
        self.user.finish_auth(code).await.with_context(|| {
            "egs::save_epic_auth_code() Failed to authenticate using auth code! | Err: "
        })?;

        self.store_session_to_file()
            .with_context(|| "egs::save_epic_auth_code() -> ")
    }

    pub fn display_name(&self) -> String {
        self.user.display_name()
    }

    /// Read access to the underlying Epic Games user, e.g. for update checks.
    pub fn user(&self) -> &User {
        &self.user
    }

    /// Validates the game's Epic metadata, fetches the current install
    /// manifest and wraps everything in a [`DownloadJob`] ready to be handed
    /// to the global downloader.
    pub async fn prepare_download_job(
        &self,
        game: &mut MonarchGame,
        opts: &DownloadOptions,
    ) -> Result<DownloadJob> {
        let mut namespace: String = String::new();
        for store in game.stores.iter() {
            if store.name == "epicgames" {
                namespace = store.store_id.clone();
            }
        }

        if namespace.is_empty() {
            error!("egs_client::prepare_download_job() Missing Epic Games namespace!");
            bail!("Missing Epic Games namespace!")
        }

        let catalog_id = game
            .properties
            .other
            .get("catalog_id")
            .cloned()
            .unwrap_or_default();
        let app_name = game
            .properties
            .other
            .get("app_name")
            .cloned()
            .unwrap_or_default();

        if catalog_id.is_empty() || app_name.is_empty() {
            error!(
                "egs_client::prepare_download_job() Missing asset catalog_id/app_name on game '{}'",
                game.name
            );
            bail!("Missing Epic Games asset catalog_id or app_name — refresh the library")
        }

        // Map the OS target to the Epic Games platform identifier
        let platform = match opts.os.as_str() {
            "linux" => "Linux",
            "windows" => "Windows",
            "macos" => "Mac",
            _ => "Windows",
        };

        let token = self.user.session().get_access_token().await;
        let manifest: Manifest =
            get_game_manifest(&token, platform, &namespace, &catalog_id, &app_name)
                .await
                .with_context(|| {
                    "egs_client::prepare_download_job() Failed to fetch game manifest from Epic Games! | Err: "
                })?;

        // Add some missing properties to the game from Manifest
        game.properties.other.insert(
            "egs_manifest_launch_command".to_string(),
            manifest.launch_command().to_string(),
        );

        Ok(DownloadJob::new(game, opts.clone(), manifest))
    }

    pub async fn get_library(&self) -> Vec<MonarchGame> {
        let mut games = self.get_user_games().await;

        for game in games.iter_mut() {
            game.thumbnail_path = generate_library_image_path(&game.name, GameImageType::Cover)
                .to_string_lossy()
                .to_string();
            game.artwork_path = generate_library_image_path(&game.name, GameImageType::Artwork)
                .to_string_lossy()
                .to_string();
        }

        games
    }

    pub async fn get_user_games(&self) -> Vec<MonarchGame> {
        // Launcher assets (not entitlements) carry the catalogItemId + appName
        // pair required by the CDN manifest endpoint.
        let assets = match monarch_egs::owned_assets(&self.user, "Windows").await {
            Ok(assets) => assets,
            Err(e) => {
                error!(
                    "egs::get_user_games() Failed to fetch owned assets from Epic Games! | Err: {e}"
                );
                return Vec::new();
            }
        };
        let mut results: Vec<MonarchGame> = Vec::new();
        let monarch_url: &'static str = std::env!("MONARCH_URL");

        let mut seen_namespaces = std::collections::HashSet::new();
        let mut chosen_assets = Vec::new();
        for asset in &assets {
            if asset.namespace == "ue" || !seen_namespaces.insert(asset.namespace.clone()) {
                continue;
            }
            if let Some(chosen) = monarch_egs::pick_asset_for_namespace(&assets, &asset.namespace) {
                chosen_assets.push(chosen.clone());
            }
        }

        let mut tasks = Vec::new();

        for asset in chosen_assets {
            let namespace = asset.namespace.clone();
            let catalog_id = asset.catalog_item_id.clone();
            let app_name = asset.app_name.clone();
            let egs_user: User = self.user.clone();

            let task: JoinHandle<Result<MonarchGame>> = task::spawn(async move {
                // Parse general game metadata via monarch-launcher.com
                info!("Parsing {} via {}.", namespace, monarch_url);
                let url = format!(
                    "{}/api/games?store_id={}&store=epicgames",
                    monarch_url, namespace
                );
                let response = match monarch_http::client().get(&url).send().await {
                    Ok(resp) => resp,
                    Err(e) => {
                        error!("egs::get_user_games() Failed to request Monarch API for app_id: {} | Err: {}", namespace, e);
                        bail!("egs::get_user_games() Failed to request Monarch API")
                    }
                };

                let response_text: String = response.text().await.unwrap();

                let mut game: MonarchGame;
                match serde_json::from_str::<Vec<MonarchWebApiGame>>(&response_text) {
                    Ok(games) => {
                        if !games.is_empty() {
                            game = games[0].clone().into_monarchgame();
                            game.properties
                                .other
                                .insert("catalog_id".to_string(), catalog_id.clone());
                            game.properties
                                .other
                                .insert("app_name".to_string(), app_name);
                        } else {
                            bail!("egs_client::get_user_games() Failed to parse response as MonarchWebApiGame | Err: Vec is empty")
                        }
                    }
                    Err(e) => {
                        bail!("egs_client::get_user_games() Failed to parse response as MonarchWebApiGame | Err: {e}")
                    }
                };

                // Parse EGS specific metadata
                let meta = match monarch_egs::get_game_metadata(
                    &egs_user,
                    &namespace,
                    &catalog_id,
                    "US",
                    "en",
                )
                .await
                {
                    Ok(m) => m,
                    Err(e) => {
                        warn!(
                            "Failed to get metadata for {} from EGS! | Err: {}",
                            game.name, e
                        );
                        bail!("egs_client::get_user_games() Failed to get metadata for {} from EGS! | Err: {}", game.name, e)
                    }
                };

                game.properties.other.insert(
                    "requires_ot".to_string(),
                    meta.custom_attributes
                        .get("OwnershipToken")
                        .cloned()
                        .unwrap_or_default()
                        .value,
                );
                game.properties.other.insert(
                    "offline_enabled".to_string(),
                    meta.custom_attributes
                        .get("CanRunOffline")
                        .cloned()
                        .unwrap_or_default()
                        .value,
                );

                Ok(game)
            });

            tasks.push(task);
        }

        for task in tasks {
            if let Ok(finished_task) = task.await {
                if let Ok(game) = finished_task {
                    results.push(game);
                }
            }
        }

        results
    }

    fn get_epic_games_token_path() -> PathBuf {
        get_monarch_home().join("monarch_egs.json")
    }

    fn load_session_from_file(&self) -> Result<Session> {
        let path: PathBuf = Self::get_epic_games_token_path();
        let json_content_str: String = std::fs::read_to_string(&path).with_context(|| {
            format!(
                "egs::load_session_from_file() Failed to read {} to String! | Err: ",
                path.display()
            )
        })?;
        serde_json::from_str(&json_content_str).with_context(|| {
            "egs::load_session_from_file() Failed to parse content to Session! | Err: "
        })
    }

    fn store_session_to_file(&self) -> Result<()> {
        let json_content = serde_json::to_value(self.user.session()).unwrap();
        let path: PathBuf = Self::get_epic_games_token_path();
        std::fs::write(&path, json_content.to_string()).with_context(|| {
            "egs::store_session_to_file() Failed to write EGS credentials to file! | Err: "
        })
    }
}

#[async_trait]
impl StoreType for EgsClient {
    async fn search_games(&self, name: &str, _filter: &SearchFilter) -> Vec<Box<dyn SearchResult>> {
        let monarch_url: &'static str = std::env!("MONARCH_URL");
        let search_term: String =
            format!("{}/api/games?search={}?store=epicgames", monarch_url, name,);

        let response = match monarch_http::client().get(search_term).send().await {
            Ok(resp) => resp,
            Err(e) => {
                error!("egs::search_games() reqwest::get() failed! | Err: {}", e);
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

    async fn install_game(&self, game: &mut MonarchGame, opts: &DownloadOptions) -> Result<()> {
        let job = self.prepare_download_job(game, opts).await?;

        // Submit the job to the global downloader, which routes it to the EGS
        // download handler (and on to monarch_egs) once registered.
        match MONARCH_STATE.read() {
            Ok(state) => match state.get_downloader_ptr().write() {
                Ok(mut downloader) => {
                    if let Err(e) = downloader.register_egs_handler() {
                        error!(
                            "egs_client::install_game() Failed to register EGS download handler | Err: {e}"
                        );
                        bail!("Failed to register EGS download handler!")
                    }
                    downloader.start_download(job);
                }
                Err(e) => {
                    error!("egs_client::install_game() Failed to lock on downloader | Err: {e}");
                    bail!("Failed to lock on downloader!")
                }
            },
            Err(e) => {
                error!("egs_client::install_game() Failed to lock on MONARCH_STATE | Err: {e}");
                bail!("Failed to lock on MONARCH_STATE!")
            }
        }

        Ok(())
    }

    async fn uninstall_game(&self, _game: &MonarchGame) -> Result<()> {
        unimplemented!()
    }

    async fn update_game(&self, _game: &MonarchGame) -> Result<()> {
        unimplemented!()
    }

    fn game_is_installed(&self, _store_id: &str) -> bool {
        false
    }

    fn store_enabled(&self) -> bool {
        let settings_lock = match get_settings() {
            Ok(lock) => lock,
            Err(e) => {
                error!("egs::store_enabled() get_settings() failed! | Err: {}", e);
                return false;
            }
        };
        let settings = match settings_lock.read() {
            Ok(settings) => settings,
            Err(e) => {
                error!(
                    "egs::store_enabled() settings_lock.read() failed! | Err: {}",
                    e
                );
                return false;
            }
        };

        settings.epic.manage
    }

    async fn launch_game(&self, game: &MonarchGame) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            // Epic Games games installed and managed by Monarch launch through
            // the EGS path so they get online-auth arguments and the manifest's
            // launch executable + arguments.
            let is_managed_egs = game.managed_by_monarch
                && game.stores.iter().any(|store| store.name == "epicgames");

            if is_managed_egs {
                return egs::egs_run(&game)
                    .await
                    .with_context(|| "monarchgame::launch() -> ");
            }

            if game.executable_path.is_some() {
                return umu_run(&game)
                    .await
                    .with_context(|| "monarchgame::launch() -> ");
            }

            if let Some(comp) = &game.compatibility {
                if comp.contains("UMU") {
                    return umu_run(&game)
                        .await
                        .with_context(|| "monarchgame::launch() -> ");
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Epic Games games installed and managed by Monarch launch through
            // the EGS path so they get online-auth arguments and the manifest's
            // launch executable + arguments.
            let is_managed_egs = game.managed_by_monarch
                && game.stores.iter().any(|store| store.name == "epicgames");

            if is_managed_egs {
                return egs::egs_run(&game)
                    .await
                    .with_context(|| "monarchgame::launch() -> ");
            }

            /*
             * TODO: Handle launching exes
            if game.executable_path.is_some() {
                return (&game)
                    .await
                    .with_context(|| "monarchgame::launch() -> ");
            }
            */
        }

        Ok(())
    }
}
