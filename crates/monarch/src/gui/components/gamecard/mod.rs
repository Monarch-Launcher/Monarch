use crate::monarch_games::monarchgame::MonarchGame;

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
    UpdateGames(Vec<MonarchGame>),

    // Properties related
    Properties(properties::Message),
}
