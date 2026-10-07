use std::sync::{Arc, RwLock};

use iced::{Element, Task};

use crate::gui::components::gamecard;
use monarch_core::{
    monarch_games::monarchgame::{MonarchGame, MonarchWebApiGame},
    monarch_utils::monarch_state::MonarchState,
};

mod update;
mod view;

#[derive(Clone, Debug)]
pub enum Message {
    SearchChanged(String),
    FiltersPressed,
    PerformSearch,
    UpdateGames(Vec<MonarchWebApiGame>),
    GameImgLoaded,
    GameCard(gamecard::GameCardMessage),
    OpenStoreDetails(Arc<RwLock<MonarchGame>>),
    Tick,
}

use crate::gui::components::gamecard::game_browser::GameBrowser;
use monarch_core::monarch_games::stores::SearchFilter;

pub struct SearchPage {
    search_value: String,
    browser: GameBrowser,
    is_searching: bool,
    dot_count: u8,
    tick_counter: u8,

    app_state: Arc<RwLock<MonarchState>>,
}

impl SearchPage {
    pub fn new(state: Arc<RwLock<MonarchState>>) -> Self {
        Self {
            app_state: state,
            ..Default::default()
        }
    }

    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::SearchChanged(value) => {
                self.search_value = value;
                iced::Task::none()
            }
            Message::FiltersPressed => {
                // TODO: Show filters
                iced::Task::none()
            }
            Message::PerformSearch => self.perform_search(SearchFilter::default()),
            Message::UpdateGames(games) => self.update_games(games),
            Message::GameImgLoaded => Task::none(),
            Message::GameCard(game_card_message) => {
                if let gamecard::GameCardMessage::GamePressed(id) = &game_card_message {
                    if let Some(game_card) = self
                        .browser
                        .games
                        .games
                        .iter()
                        .find(|g| g.game.read().unwrap().id == *id)
                    {
                        return iced::Task::done(Message::OpenStoreDetails(game_card.game.clone()));
                    }
                }
                self.browser
                    .update(game_card_message)
                    .map(Message::GameCard)
            }
            Message::OpenStoreDetails(_) => iced::Task::none(),
            Message::Tick => self.tick(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        self.content_view()
    }
}

impl Default for SearchPage {
    fn default() -> Self {
        Self {
            search_value: String::new(),
            browser: GameBrowser::default(),
            is_searching: false,
            dot_count: 0,
            tick_counter: 0,
            app_state: Arc::new(RwLock::new(MonarchState::new())),
        }
    }
}
