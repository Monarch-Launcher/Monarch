use crate::monarch_games::monarchgame::MonarchGame;
use anyhow::Result;
use tauri::{utils::platform, AppHandle};

use super::games::GameType;

pub trait StoreType {
    fn search_games(&self, name: &str) -> Vec<Box<dyn GameType>>;
    fn install_game(&self, handle: &AppHandle, name: &str, platform_id: &str) -> Result<()>;
    fn uninstall_game(&self, handle: &AppHandle, platform_id: &str) -> Result<()>;
    fn update_game(&self, handle: &AppHandle, platform_id: &str) -> Result<()>;
    fn game_is_installed(&self, handle: &AppHandle, platform_id: &str) -> bool;
    fn platform_enabled(&self) -> bool;
    fn launch_game(&self, handle: &AppHandle, game: &MonarchGame) -> Result<()>;
}
