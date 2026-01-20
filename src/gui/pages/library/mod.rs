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
}

#[derive(Default)]
pub struct LibraryPage {
    browser: GameBrowser,
    is_refreshing: bool,
}

impl LibraryPage {
    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::RefreshLibrary => {
                self.is_refreshing= true;
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
            _ => {
                iced::Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let refresh_btn = primary_button("Scan for games", Some(Message::RefreshLibrary));

        let games_content: Element<'_, Message> = if self.is_refreshing {
            container(
                column![text("Looking for games...")
                    .size(32)
                    .style(|_theme: &iced::Theme| text::Style {
                        color: Some(Color::from_rgb8(255, 127, 0)),
                    })]
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
                    .height(Fill)
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
