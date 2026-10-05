use std::sync::{Arc, RwLock};

use monarch_core::monarch_games::monarchgame::MonarchGame;

pub mod actions;
pub mod container;
pub mod game_browser;
pub mod gamecard;
pub mod properties;

#[derive(Debug, Clone)]
pub enum GameCardMessage {
    GameHovered(String),
    GameUnhovered(String),
    GamePressed(String),
    Tick,
    UpdateGames(Vec<Arc<RwLock<MonarchGame>>>),

    // Properties related
    Properties(properties::Message),
}
