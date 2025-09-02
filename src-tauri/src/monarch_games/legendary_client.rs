use anyhow::Result;

use super::games::GameType;
use super::stores::StoreType;

pub struct LegendaryClient {}

impl LegendaryClient {
    pub fn new() -> Self {
        Self {}
    }
}

impl StoreType for LegendaryClient {
    fn search_games(&self, name: &str) -> Vec<Box<dyn GameType>> {
        unimplemented!()
    }

    fn install_game(&self, name: &str, platform_id: &str) -> Result<()> {
        unimplemented!()
    }

    fn uninstall_game(&self, platform_id: &str) -> Result<()> {
        unimplemented!()
    }

    fn update_game(&self, platform_id: &str) -> Result<()> {
        unimplemented!()
    }
}