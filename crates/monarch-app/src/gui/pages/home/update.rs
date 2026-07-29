use super::{HomePage, Message};
use crate::gui::components::gamecard::gamecard::GameCard;
use crate::gui::components::gamecard::GameCardMessage;
use monarch_core::{monarch_games, monarch_library};

impl HomePage {
    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::UpdateRecommendations(games) => {
                self.is_loading = false;

                // Build gamecards for the recommended section
                self.recommended_games = games.iter().cloned().map(GameCard::new).collect();

                // Spoof deals from the same library (shift one slot so the cards feel different)
                let mut deals = games.clone();
                if deals.len() > 1 {
                    deals.rotate_left(1);
                }
                self.deals = deals;

                iced::Task::none()
            }

            Message::GameCard(gc_msg) => {
                if let GameCardMessage::GamePressed(id) = &gc_msg {
                    if let Some(card) = self.recommended_games.iter().find(|c| c.game.id == *id) {
                        return iced::Task::done(Message::OpenGameDetails(card.game.clone()));
                    }
                }

                // Forward hover/tick messages to all cards
                for card in &mut self.recommended_games {
                    let _ = card.update(gc_msg.clone());
                }

                // Trigger download tasks
                let update_tasks = iced::Task::batch(self.recommended_games.iter().cloned().map(
                    |mut gamecard| {
                        iced::Task::perform(
                            async move {
                                if !gamecard.game.has_properties() {
                                    monarch_games::commands::get_game_properties(
                                        &mut gamecard.game,
                                    )
                                    .await;
                                }
                                gamecard.game
                            },
                            Message::GameUpdated,
                        )
                    },
                ));

                update_tasks
            }

            Message::GameUpdated(game) => {
                if let Some(card) = self
                    .recommended_games
                    .iter_mut()
                    .find(|c| c.game.id == game.id)
                {
                    card.game = game;
                }
                iced::Task::none()
            }

            Message::OpenGameDetails(_) => iced::Task::none(),

            Message::LaunchGame(game) => iced::Task::perform(
                async move {
                    let _ = monarch_core::monarch_games::commands::launch_game(&game).await;
                },
                |_| Message::Tick, // dummy message; we just want the side-effect
            ),

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
        iced::Task::perform(
            async {
                match monarch_library::commands::get_home_recomendations().await {
                    Ok(games) => games,
                    Err(_) => Vec::new(),
                }
            },
            Message::UpdateRecommendations,
        )
    }
}
