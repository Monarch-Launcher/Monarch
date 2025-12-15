use anyhow::Result;
use async_trait::async_trait;
use tauri::AppHandle;
use tracing::error;

use crate::monarch_games::monarchgame::{MonarchGame, MonarchWebGame};
use crate::monarch_games::stores::SearchFilter;
use crate::monarch_utils::monarch_fs::generate_cache_image_path;
use crate::monarch_utils::monarch_settings::get_settings_state;
use crate::monarch_utils::monarch_terminal::run_in_terminal;

use super::games::GameType;
use super::stores::StoreType;

pub struct LegendaryClient {
    cli_path: String,
}

impl LegendaryClient {
    pub fn new() -> Self {
        Self {
            cli_path: String::new(),
        }
    }

    /// Abstraction for all legendary functions to run terminal commands.
    fn run_legendary_cmd(&self, handle: AppHandle, command: String) -> Result<()> {
        // Start a new async thread launching the game
        tokio::spawn(async move {
            if let Err(e) = run_in_terminal(&handle, &command, None, None).await {
                error!("legendary_client::launch_game() -> {e}");
                // TODO: Trigger an error dialog in frontend.
            }
        });
        Ok(())
    }
}

#[async_trait]
impl StoreType for LegendaryClient {
    async fn search_games(&self, name: &str, _filter: &SearchFilter) -> Vec<Box<dyn GameType>> {
        let search_term: String = format!(
            "https://monarch-launcher.com/api/games?search={}?platform=legendary",
            name,
        );
        let response = reqwest::blocking::get(search_term).unwrap();
        let resp_content = response.text().unwrap();
        let web_games: Vec<MonarchWebGame> = serde_json::from_str(&resp_content).unwrap();

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

    async fn install_game(&self, handle: &AppHandle, game: &MonarchGame) -> Result<()> {
        let command: String = format!("{} install {}", self.cli_path, &game.platform_id);
        self.run_legendary_cmd(handle.clone(), command)
    }

    async fn uninstall_game(&self, handle: &AppHandle, game: &MonarchGame) -> Result<()> {
        let command: String = format!("{} uninstall {}", self.cli_path, &game.platform_id);
        self.run_legendary_cmd(handle.clone(), command)
    }

    async fn update_game(&self, handle: &AppHandle, game: &MonarchGame) -> Result<()> {
        let command: String = format!("{} update {}", self.cli_path, &game.platform_id);
        self.run_legendary_cmd(handle.clone(), command)
    }

    fn game_is_installed(&self, handle: &AppHandle, platform_id: &str) -> bool {
        unimplemented!()
    }

    fn platform_enabled(&self) -> bool {
        get_settings_state().epic.manage
    }

    async fn launch_game(&self, handle: &AppHandle, game: &MonarchGame) -> Result<()> {
        let command: String = format!("{} launch {}", self.cli_path, game.platform_id);
        self.run_legendary_cmd(handle.clone(), command)
    }
}
