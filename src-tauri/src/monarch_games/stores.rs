use crate::monarch_games::monarchgame::MonarchGame;
use anyhow::Result;
use async_trait::async_trait;
use tauri::AppHandle;

use super::games::GameType;

#[async_trait]
pub trait StoreType: Send + Sync {
    async fn search_games(&self, name: &str) -> Vec<Box<dyn GameType>>;
    async fn install_game(&self, handle: &AppHandle, game: &MonarchGame) -> Result<()>;
    async fn uninstall_game(&self, handle: &AppHandle, game: &MonarchGame) -> Result<()>;
    async fn update_game(&self, handle: &AppHandle, game: &MonarchGame) -> Result<()>;
    fn game_is_installed(&self, handle: &AppHandle, platform_id: &str) -> bool;
    fn platform_enabled(&self) -> bool;
    async fn launch_game(&self, handle: &AppHandle, game: &MonarchGame) -> Result<()>;
}
