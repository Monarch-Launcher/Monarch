use super::stores::StoreType;
use anyhow::Result;

pub trait GameType {
    fn get_name(&self) -> String;
    fn get_platform(&self) -> Box<dyn StoreType>;
    fn get_platform_id(&self) -> String;
    fn get_description(&self) -> String;
    fn get_price(&self) -> f64;
    fn launch(&self) -> Result<()>;
}