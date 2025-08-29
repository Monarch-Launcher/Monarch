use anyhow::Result;
use super::monarchgame::MonarchGame;
use super::games::GameType;

pub trait StoreType {
    fn search_game(&self, name: &str) -> Result<Vec<Box<dyn GameType>>>;
    fn download_game(&self, name: &str, platform_id: &str) -> Result<Vec<Box<dyn GameType>>>;
    fn uninstall_game(&self, platform_id: &str) -> Result<()>;
    fn update_game(&self, game: &MonarchGame) -> Result<()>;
}