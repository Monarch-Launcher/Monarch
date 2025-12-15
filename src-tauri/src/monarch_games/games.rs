use super::stores::StoreType;
use crate::monarch_games::monarchgame::MonarchGame;
use anyhow::Result;
use async_trait::async_trait;
use tauri::AppHandle;

#[async_trait]
pub trait GameType: Send + Sync {
    fn get_name(&self) -> String;
    fn get_platform(&self) -> Box<dyn StoreType>;
    fn get_platform_id(&self) -> String;
    fn get_description(&self) -> String;
    fn get_price(&self) -> f64;
    async fn launch(&self, handle: &AppHandle) -> Result<()>;
    fn into_monarchgame(&self) -> MonarchGame;
}
