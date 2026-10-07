use std::sync::{Arc, RwLock};

use super::stores::StoreType;
use crate::{
    monarch_games::monarchgame::{MonarchGame, MonarchWebApiGame},
    monarch_utils::monarch_settings::Settings,
};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait GameType: Send + Sync {
    fn get_name(&self) -> String;
    fn get_store(&self) -> Box<dyn StoreType>;
    fn get_store_name(&self) -> String;
    fn get_store_id(&self) -> String;
    fn get_description(&self) -> String;
    fn get_price(&self) -> f64;
    async fn launch(&self, settings_handle: Arc<RwLock<Settings>>) -> Result<()>;
    fn into_monarchgame(&self) -> MonarchGame;
}

#[async_trait]
pub trait SearchResult: Send + Sync {
    fn to_search_result(&self) -> MonarchWebApiGame;
    fn into_monarchgame(&self) -> MonarchGame;
}
