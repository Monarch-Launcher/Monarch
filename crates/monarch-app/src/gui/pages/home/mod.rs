use std::sync::{Arc, RwLock};

use crate::gui::components::gamecard::gamecard::GameCard;
use crate::gui::components::gamecard::GameCardMessage;
use monarch_core::monarch_games::monarchgame::{MonarchGame, MonarchWebApiGame};
use monarch_core::monarch_utils::monarch_state::MonarchState;

mod update;
mod view;

#[derive(Clone, Debug)]
pub enum Message {
    UpdateRecommendations(Vec<Arc<RwLock<MonarchGame>>>),
    GameUpdated(Arc<RwLock<MonarchGame>>),
    GameCard(GameCardMessage),
    OpenGameDetails(Arc<RwLock<MonarchGame>>),
    LaunchGame(Arc<RwLock<MonarchGame>>),
    NextDeal,
    PrevDeal,
    Tick,
}

pub struct HomePage {
    pub recommended_games: Vec<GameCard>,
    pub deals: Vec<MonarchWebApiGame>,
    pub current_deal_index: usize,
    pub is_loading: bool,

    app_state: Arc<RwLock<MonarchState>>,
}

impl HomePage {
    pub fn new(state: Arc<RwLock<MonarchState>>) -> Self {
        Self {
            app_state: state,
            ..Default::default()
        }
    }
}

impl Default for HomePage {
    fn default() -> Self {
        Self {
            recommended_games: Vec::new(),
            deals: Vec::new(),
            current_deal_index: 0,
            is_loading: true,
            app_state: Arc::new(RwLock::new(MonarchState::new())),
        }
    }
}
