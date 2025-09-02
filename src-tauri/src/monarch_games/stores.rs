use anyhow::Result;
use super::games::GameType;

pub trait StoreType {
    fn search_games(&self, name: &str) -> Vec<Box<dyn GameType>>;
    fn install_game(&self, name: &str, platform_id: &str) -> Result<()>;
    fn uninstall_game(&self, platform_id: &str) -> Result<()>;
    fn update_game(&self, platform_id: &str) -> Result<()>;
}