use crate::monarch_games::monarchgame::MonarchGame;

pub mod container;
pub mod drawer;
pub mod gamecard;

#[derive(Debug, Clone)]
pub enum GameCardMessage {
    GameHovered(String),
    GameUnhovered(String),
    GamePressed(String),
    Tick,
    UpdateGames(Vec<MonarchGame>),
    OpenDrawer,
    CloseDrawer,
}
