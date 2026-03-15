use std::collections::HashMap;

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use tracing::{error, info, warn};

use crate::monarch_games::games::SearchResult;
use crate::monarch_games::monarchgame::{GameImageType, MonarchGame, MonarchWebApiGame};
use crate::monarch_games::stores::DownloadOptions;
use crate::monarch_games::stores::SearchFilter;
use crate::monarch_utils::monarch_fs::generate_cache_image_path;
use crate::monarch_utils::monarch_settings::get_settings;
use crate::monarch_utils::monarch_terminal::spawn_terminal;

use super::stores::StoreType;

#[cfg(target_os = "linux")]
use super::linux::legendary;

#[cfg(target_os = "macos")]
use super::macos::legendary;

#[cfg(target_os = "windows")]
use super::windows::legendary;

pub struct LegendaryClient {
    cli_path: String,
}

impl LegendaryClient {
    pub fn new() -> Self {
        Self {
            cli_path: legendary::get_legendary_exe().to_str().unwrap().to_string(),
        }
    }

    /// Abstraction for all legendary functions to run terminal commands.
    fn run_legendary_cmd(&self, command: String) -> Result<()> {
        // Start a new async thread launching the game
        tokio::spawn(async move {
            let rx = spawn_terminal(command, HashMap::new(), None);
            if let Err(e) = rx.await {
                error!("legendary_client::run_legendary_cmd() Terminal command failed! | Err: {e}");
            }
        });
        Ok(())
    }

    pub fn login(&self) -> Result<()> {
        let login_cmd: String = format!("{} auth; sleep 3;", self.cli_path);

        info!("Logging in to Epic Games with Legendary...");
        self.run_legendary_cmd(login_cmd)
            .with_context(|| "legendary_client::login() -> ")?;
        info!("Login complete!");

        Ok(())
    }
}

#[async_trait]
impl StoreType for LegendaryClient {
    async fn search_games(&self, name: &str, _filter: &SearchFilter) -> Vec<Box<dyn SearchResult>> {
        let monarch_url: &'static str = std::env!("MONARCH_URL");
        let search_term: String =
            format!("{}/api/games?search={}?store=epicgames", monarch_url, name,);

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

    async fn install_game(&self, game: &MonarchGame, opts: &DownloadOptions) -> Result<()> {
        let command: String = format!(
            "{} install {} --game-folder {} --store {}; sleep 5",
            self.cli_path, &game.name, opts.folder, opts.os
        );

        self.run_legendary_cmd(command)
    }

    async fn uninstall_game(&self, game: &MonarchGame) -> Result<()> {
        let command: String = format!("{} uninstall {}", self.cli_path, &game.name);
        self.run_legendary_cmd(command)
    }

    async fn update_game(&self, game: &MonarchGame) -> Result<()> {
        let command: String = format!("{} update {}", self.cli_path, &game.name);
        self.run_legendary_cmd(command)
    }

    fn game_is_installed(&self, _store_id: &str) -> bool {
        unimplemented!()
    }

    fn store_enabled(&self) -> bool {
        let settings_lock = match get_settings() {
            Ok(lock) => lock,
            Err(e) => {
                error!(
                    "legendary_client::store_enabled() get_settings() failed! | Err: {}",
                    e
                );
                return false;
            }
        };
        let settings = match settings_lock.read() {
            Ok(settings) => settings,
            Err(e) => {
                error!(
                    "legendary_client::store_enabled() settings_lock.read() failed! | Err: {}",
                    e
                );
                return false;
            }
        };

        settings.epic.manage
    }

    async fn launch_game(&self, game: &MonarchGame) -> Result<()> {
        let command: String = format!("{} launch {}", self.cli_path, game.name);
        self.run_legendary_cmd(command)
    }
}

pub fn legendary_is_installed() -> bool {
    legendary::legendary_is_installed()
}

pub fn install_legendary() -> Result<()> {
    legendary::install_legendary().with_context(|| "legendary_client::install_legendary() -> ")
}

pub fn remove_legendary() -> Result<()> {
    if !legendary_is_installed() {
        warn!("linux::remove_legendary() Umu not found!");
        bail!("Umu not found!")
    }

    std::fs::remove_dir_all(&legendary::get_legendary_dir()).with_context(|| "linux::remove_umu() Failed to remove_dir_all() | Err: ")
}
