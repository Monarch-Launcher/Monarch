use std::sync::{Arc, RwLock};

use iced::Task;
use tracing::error;

use crate::gui::{
    components::gamecard,
    pages::search::{Message, SearchPage},
    show_error,
};
use monarch_core::monarch_games::{
    self,
    games::SearchResult,
    monarchgame::{MonarchGame, MonarchWebApiGame},
    stores::SearchFilter,
};

impl SearchPage {
    pub fn perform_search(&mut self, search_filter: SearchFilter) -> Task<Message> {
        self.is_searching = true;
        self.dot_count = 3;
        self.tick_counter = 0;
        let search_term = self.search_value.clone();
        let settins_handle = match self.app_state.read() {
            Ok(state) => state.get_settings_ptr(),
            Err(e) => {
                error!("SearchPage::perform_search() Failed to acquire lock on state_handle! | Err: {e}");
                show_error(format!("Failed to search for: {search_term}"));
                return Task::none();
            }
        };
        iced::Task::perform(
            async move {
                monarch_games::commands::search_games(settins_handle, search_term, search_filter)
                    .await
            },
            Message::UpdateGames,
        )
    }

    pub fn update_games(&mut self, games: Vec<MonarchWebApiGame>) -> iced::Task<Message> {
        self.is_searching = false;

        let processed_games: Vec<MonarchWebApiGame> = games
            .iter()
            .cloned()
            .map(|mut game| {
                game.thumbnail_path = "".to_string();
                game
            })
            .collect();

        let processed_game_handles: Vec<Arc<RwLock<MonarchGame>>> = processed_games
            .iter()
            .map(|g| Arc::new(RwLock::new(g.into_monarchgame())))
            .collect();

        // Trigger download tasks
        let download_tasks =
            iced::Task::batch(processed_game_handles.iter().cloned().map(|game| {
                iced::Task::perform(
                    async move {
                        if let Err(e) =
                            monarch_games::commands::download_thumbnail(game.clone()).await
                        {
                            error!(
                                "Failed to download thumbnail for game {}: {}",
                                game.read().unwrap().id,
                                e
                            );
                        }
                    },
                    |_| Message::GameImgLoaded,
                )
            }));

        // Update browser games
        let _ = self.browser.update(gamecard::GameCardMessage::UpdateGames(
            processed_game_handles,
        ));

        download_tasks
    }

    pub fn tick(&mut self) -> iced::Task<Message> {
        if self.is_searching {
            self.tick_counter = self.tick_counter.wrapping_add(1);
            if self.tick_counter % 60 == 0 {
                self.dot_count = (self.dot_count % 3) + 1;
            }
        }
        self.browser
            .update(gamecard::GameCardMessage::Tick)
            .map(Message::GameCard)
    }
}
