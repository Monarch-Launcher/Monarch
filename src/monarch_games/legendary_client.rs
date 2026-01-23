use anyhow::{Context, Result};
use async_trait::async_trait;
use tracing::{error, info};

use crate::monarch_games::games::SearchResult;
use crate::monarch_games::monarchgame::{GameImageType, MonarchGame, MonarchWebApiGame};
use crate::monarch_games::stores::SearchFilter;
use crate::monarch_utils::monarch_fs::generate_cache_image_path;
use crate::monarch_utils::monarch_settings::get_settings_state;
use crate::monarch_games::stores::DownloadOptions;

use super::games::GameType;
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
        /*
        tauri::async_runtime::spawn(async move {
            if let Err(e) = run_in_terminal(&handle, &command, None, None).await {
                error!("legendary_client::launch_game() -> {e}");
                // TODO: Trigger an error dialog in frontend.
            }
        });
        */
        Ok(())
    }

    pub fn login(&self) -> Result<()> {
        let login_cmd: String = format!("{} auth", self.cli_path);

        info!("Loging in to Epic Games with Legendary...");
        self.run_legendary_cmd(login_cmd).with_context(|| "legendary_client::login() -> ")?;
        info!("Logic complete!");

        Ok(())
    } 
}

#[async_trait]
impl StoreType for LegendaryClient {
    async fn search_games(&self, name: &str, _filter: &SearchFilter) -> Vec<Box<dyn SearchResult>> {
        let monarch_url: &'static str = std::env!("MONARCH_URL");
        let search_term: String = format!(
            "{}/api/games?search={}?platform=epicgames",
            monarch_url, name,
        );

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
            "{} install {} --game-folder {} --platform {}",
            self.cli_path,
            &game.name,
            opts.folder,
            opts.os
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

    fn game_is_installed(&self, platform_id: &str) -> bool {
        unimplemented!()
    }

    fn platform_enabled(&self) -> bool {
        get_settings_state().epic.manage
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
