use super::stores::StoreType;
use anyhow::Result;
use tauri::AppHandle;
use crate::monarch_games::monarchgame::MonarchGame;
use async_trait::async_trait;

#[async_trait]
pub trait GameType {
    fn get_name(&self) -> String;
    fn get_platform(&self) -> Box<dyn StoreType>;
    fn get_platform_id(&self) -> String;
    fn get_description(&self) -> String;
    fn get_price(&self) -> f64;
    async fn launch(&self, handle: &AppHandle) -> Result<()>;
    fn into_monarchgame(&self) -> MonarchGame;
}