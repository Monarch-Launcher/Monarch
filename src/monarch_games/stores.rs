use crate::monarch_games::{games::SearchResult, monarchgame::MonarchGame};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait StoreType: Send + Sync {
    async fn search_games(&self, name: &str, filter: &SearchFilter) -> Vec<Box<dyn SearchResult>>;
    async fn install_game(&self, game: &MonarchGame, opts: &DownloadOptions) -> Result<()>;
    async fn uninstall_game(&self, game: &MonarchGame) -> Result<()>;
    async fn update_game(&self, game: &MonarchGame) -> Result<()>;
    fn game_is_installed(&self, platform_id: &str) -> bool;
    fn platform_enabled(&self) -> bool;
    async fn launch_game(&self, game: &MonarchGame) -> Result<()>;
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchFilter {
    // Stores
    pub steam: bool,
    pub epic: bool,
    pub gog: bool,
    pub itch: bool,

    // Search sources
    pub monarch: bool,
    pub steam_powered: bool,
    pub egs: bool,
}

impl Default for SearchFilter {
    fn default() -> Self {
        Self { 
            steam: true,
            epic: true,
            gog: true,
            itch: true,

            monarch: true,
            steam_powered: false,
            egs: false
         }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DownloadOptions {
    pub folder: String,
    pub platform: String,
    pub game_name: String,
    pub game_platform: String,
    pub game_platform_id: String,
    pub os: String,
}
