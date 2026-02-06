use crate::monarch_games::monarchgame::MonarchGame;

pub mod container;
pub mod drawer;
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
    OpenDrawer,
    CloseDrawer,
    OpenStorePage(String),
    WebviewOpened(iced::window::Id),
    NoOp,

    // Properties related
    OpenProperties(MonarchGame),
    CloseProperties,
    Properties(properties::Message),
}
