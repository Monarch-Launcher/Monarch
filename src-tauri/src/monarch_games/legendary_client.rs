use anyhow::Result;
use tauri::AppHandle;

use crate::monarch_games::monarchgame::MonarchGame;
use crate::monarch_utils::monarch_terminal::run_in_terminal;

use super::games::GameType;
use super::stores::StoreType;

pub struct LegendaryClient {
    cli_path: String,
}

impl LegendaryClient {
    pub fn new() -> Self {
        Self {
            cli_path: String::new()
        }
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

    fn game_is_installed(&self, platform_id: &str) -> bool {
        unimplemented!()
    }

    fn platform_enabled(&self) -> bool {
        unimplemented!()
    }
    
    fn launch_game(&self, handle: &AppHandle, game: &MonarchGame) -> Result<()> {
        let command: String = format!("{} launch {}", self.cli_path, game.platform_id);
        let handle_clone: AppHandle = handle.clone();
        
        // Start a new async thread launching the game
        tokio::spawn(async move { 
            run_in_terminal(&handle_clone, &command, None, None);
        });

        Ok(())
    }
}