use crate::gui::components::gamecard::gamecard::GameCard;
use crate::gui::components::gamecard::GameCardMessage;
use crate::monarch_games::monarchgame::MonarchGame;

mod update;
mod view;

#[derive(Clone, Debug)]
pub enum Message {
    UpdateRecommendations(Vec<MonarchGame>),
    GameCard(GameCardMessage),
    OpenGameDetails(MonarchGame),
    LaunchGame(MonarchGame),
    NextDeal,
    PrevDeal,
    Tick,
}

pub struct HomePage {
    pub recommended_games: Vec<GameCard>,
    pub deals: Vec<MonarchGame>,
    pub current_deal_index: usize,
    pub is_loading: bool,
}

impl Default for HomePage {
    fn default() -> Self {
        Self {
            recommended_games: Vec::new(),
            deals: Vec::new(),
            current_deal_index: 0,
            is_loading: true,
        }
    }
}

impl HomePage {
    pub fn new() -> Self {
        Self::default()
    }
}
