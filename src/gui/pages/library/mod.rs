use iced::widget::{column, container, text};
use iced::Length::{self, Fill};
use iced::{alignment, Color, Element};
use tracing::{error, info};

use crate::gui::components::common::primary_button;
use crate::gui::components::gamecard;
use crate::gui::components::gamecard::game_browser::GameBrowser;
use crate::monarch_games;
use crate::monarch_games::monarchgame::MonarchGame;

#[derive(Clone, Debug)]
pub enum Message {
    RefreshLibrary,
    UpdateGames(Vec<MonarchGame>),
    GameImgLoaded(MonarchGame),
    GameCard(gamecard::GameCardMessage),
    Tick,
}

pub struct LibraryPage {
    browser: GameBrowser,
    is_refreshing: bool,
    dot_count: u8,
    tick_counter: u8,
}

impl LibraryPage {
    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::RefreshLibrary => {
                self.is_refreshing = true;
                self.dot_count = 3;
                self.tick_counter = 0;
                iced::Task::perform(
                    async move { monarch_games::commands::refresh_library().await },
                    Message::UpdateGames,
                )
            }
            Message::UpdateGames(games) => {
                self.is_refreshing = false;

                for game in games.iter() {
                    info!("Found game: {:?}", game);
                }

                let processed_games: Vec<MonarchGame> = games
                    .iter()
                    .cloned()
                    .map(|mut game| {
                        game.thumbnail_path = "".to_string();
                        game
                    })
                    .collect();

                // Trigger download tasks
                let download_tasks = iced::Task::batch(games.iter().cloned().map(|game| {
                    iced::Task::perform(
                        async move {
                            if let Err(e) =
                                monarch_games::commands::download_thumbnail(game.clone()).await
                            {
                                error!("Failed to download thumbnail for game {}: {}", game.id, e);
                            }
                            game
                        },
                        Message::GameImgLoaded,
                    )
                }));

                // Update browser games
                let _ = self
                    .browser
                    .update(gamecard::GameCardMessage::UpdateGames(processed_games));

                download_tasks
            }
            Message::GameImgLoaded(game) => {
                if let Some(card) = self
                    .browser
                    .games
                    .games
                    .iter_mut()
                    .find(|c| c.game.id == game.id)
                {
                    card.game.thumbnail_path = game.thumbnail_path.clone();
                }
                iced::Task::none()
            }
            Message::GameCard(game_card_message) => self
                .browser
                .update(game_card_message)
                .map(Message::GameCard),
            Message::Tick => {
                if self.is_refreshing {
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
    }

    pub fn view(&self) -> Element<'_, Message> {
        let refresh_btn = primary_button("Scan for games", Some(Message::RefreshLibrary));

        let games_content: Element<'_, Message> = if self.is_refreshing {
            let dots = ".".repeat(self.dot_count as usize);
            container(
                column![text(format!("Looking for games{dots}")).size(32).style(
                    |_theme: &iced::Theme| text::Style {
                        color: Some(Color::from_rgb8(255, 127, 0)),
                    }
                )]
                .spacing(20)
                .align_x(alignment::Horizontal::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(100)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .into()
        } else {
            self.browser.view().map(Message::GameCard)
        };

        container(
            column![
                container(refresh_btn)
                    .width(Fill)
                    .padding(20)
                    .align_x(alignment::Horizontal::Left)
                    .align_y(alignment::Vertical::Top),
                games_content
            ]
            .align_x(alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

impl Default for LibraryPage {
    fn default() -> Self {
        let mut browser: GameBrowser = GameBrowser::default();

        match monarch_games::commands::get_library() {
            Ok(games) => {
                let _ = browser.update(gamecard::GameCardMessage::UpdateGames(games));
            }
            Err(e) => {
                error!("Failed to get library: {}", e);
            }
        }

        Self {
            browser: browser,
            is_refreshing: false,
            dot_count: 3,
            tick_counter: 0,
        }
    }
}
