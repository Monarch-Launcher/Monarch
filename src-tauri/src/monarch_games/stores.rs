use crate::monarch_games::monarchgame::MonarchGame;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use super::games::GameType;

#[async_trait]
pub trait StoreType: Send + Sync {
    async fn search_games(&self, name: &str, filter: &SearchFilter) -> Vec<Box<dyn GameType>>;
    async fn install_game(&self, handle: &AppHandle, game: &MonarchGame) -> Result<()>;
    async fn uninstall_game(&self, handle: &AppHandle, game: &MonarchGame) -> Result<()>;
    async fn update_game(&self, handle: &AppHandle, game: &MonarchGame) -> Result<()>;
    fn game_is_installed(&self, handle: &AppHandle, platform_id: &str) -> bool;
    fn platform_enabled(&self) -> bool;
    async fn launch_game(&self, handle: &AppHandle, game: &MonarchGame) -> Result<()>;
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
