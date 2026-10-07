use std::sync::{Arc, RwLock};

use iced::Task;
use tracing::error;

use super::{HomePage, Message};
use crate::gui::components::gamecard::gamecard::GameCard;
use crate::gui::components::gamecard::GameCardMessage;
use crate::gui::show_error;

use monarch_core::monarch_games;
use monarch_core::monarch_games::monarchgame::MonarchGame;
use monarch_core::monarch_library;

impl HomePage {
    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::UpdateRecommendations(games) => {
                self.is_loading = false;

                // Build gamecards for the recommended section
                self.recommended_games = games.iter().cloned().map(GameCard::new).collect();

                // Spoof deals from the same library (shift one slot so the cards feel different)
                /*
                               let mut deals = games.clone();
                               if deals.len() > 1 {
                                   deals.rotate_left(1);
                               }
                               self.deals = deals;
                */
                self.deals = vec![];

                // One background pass to enrich cards that are still missing
                // properties. Runs after paint; bounded concurrency.
                self.enrich_missing_properties()
            }

            Message::GameCard(gc_msg) => {
                if let GameCardMessage::GamePressed(id) = &gc_msg {
                    if let Some(card) = self
                        .recommended_games
                        .iter()
                        .find(|c| c.game.read().unwrap().id == *id)
                    {
                        return iced::Task::done(Message::OpenGameDetails(card.game.clone()));
                    }
                }

                // Forward hover/tick messages to all cards. Property fetching
                // intentionally does NOT happen here: it used to refetch every
                // card on each hover/tick message.
                for card in &mut self.recommended_games {
                    let _ = card.update(gc_msg.clone());
                }

                iced::Task::none()
            }

            Message::GameUpdated(game) => {
                if let Some(card) = self
                    .recommended_games
                    .iter_mut()
                    .find(|c| c.game.read().unwrap().id == game.read().unwrap().id)
                {
                    card.game = game;
                }
                iced::Task::none()
            }

            Message::OpenGameDetails(_) => iced::Task::none(),

            Message::LaunchGame(game) => {
                let settings_handle_clone = match self.app_state.read() {
                    Ok(state) => state.get_settings_ptr(),
                    Err(e) => {
                        error!("HomePage::update() Failed to acquire read lock on state_handle! | Err: {e}");
                        show_error("Failed to launch game!");
                        return Task::none();
                    }
                };

                iced::Task::perform(
                    async move {
                        let _ = monarch_core::monarch_games::commands::launch_game(
                            settings_handle_clone,
                            game,
                        )
                        .await;
                    },
                    |_| Message::Tick, // dummy message; we just want the side-effect
                )
            }

            Message::NextDeal => {
                if !self.deals.is_empty() {
                    self.current_deal_index = (self.current_deal_index + 1) % self.deals.len();
                }
                iced::Task::none()
            }

            Message::PrevDeal => {
                if !self.deals.is_empty() {
                    self.current_deal_index = self
                        .current_deal_index
                        .checked_sub(1)
                        .unwrap_or(self.deals.len() - 1);
                }
                iced::Task::none()
            }

            Message::Tick => {
                for card in &mut self.recommended_games {
                    let _ = card.update(GameCardMessage::Tick);
                }
                iced::Task::none()
            }
        }
    }

    /// Kick off a background load of recommendations. Call this once after Default::default().
    pub fn init(&self) -> iced::Task<Message> {
        // Clone outside the future so no borrow of `self` can escape the method.
        let state_handle = self.app_state.clone();
        iced::Task::perform(
            async move {
                match monarch_library::commands::get_home_recomendations(state_handle).await {
                    Ok(games) => games,
                    Err(_) => Vec::new(),
                }
            },
            Message::UpdateRecommendations,
        )
    }

    /// Enriches recommended games that don't have properties yet, keeping at
    /// most MAX_CONCURRENT_FETCHES requests in flight. Games whose properties
    /// are already known are skipped. Emits one GameUpdated per finished fetch.
    fn enrich_missing_properties(&self) -> iced::Task<Message> {
        use futures::stream::StreamExt;

        const MAX_CONCURRENT_FETCHES: usize = 8;

        let missing: Vec<Arc<RwLock<MonarchGame>>> = self
            .recommended_games
            .iter()
            .filter(|card| !card.game.read().unwrap().has_properties())
            .map(|card| card.game.clone())
            .collect();

        if missing.is_empty() {
            return iced::Task::none();
        }

        let state_handle = self.app_state.clone();
        iced::Task::stream(
            futures::stream::iter(missing.into_iter().map(move |game| {
                // The mapping closure is FnMut (called once per game), so it can't
                // move `state_handle` itself; clone a fresh Arc per future instead.
                let state_handle = state_handle.clone();
                async move {
                    monarch_games::commands::get_game_properties(
                        state_handle,
                        game.clone(),
                    )
                    .await;
                    Message::GameUpdated(game)
                }
            }))
            .buffer_unordered(MAX_CONCURRENT_FETCHES),
        )
    }
}
