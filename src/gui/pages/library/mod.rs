use iced::widget::{column, container, mouse_area, text};
use iced::Length::{self, Fill};
use iced::{alignment, Color, Element};
use tracing::{error, info};

use crate::gui::components::common::scanner_button;
use crate::gui::components::gamecard;
use crate::gui::components::gamecard::game_browser::GameBrowser;
use crate::gui::show_error;
use crate::monarch_games;
use crate::monarch_games::monarchgame::MonarchGame;

#[derive(Clone, Debug)]
pub enum Message {
    RefreshLibrary,
    UpdateGames(Vec<MonarchGame>),
    UpdateGameProperties,
    GameUpdated(MonarchGame),
    GameCard(gamecard::GameCardMessage),
    OpenGameDetails(MonarchGame),
    Tick,
    ScannerHovered(bool),
}

#[derive(Debug, Clone)]
pub struct LibraryPage {
    browser: GameBrowser,
    is_refreshing: bool,
    dot_count: u8,
    tick_counter: u8,
    is_scanner_hovered: bool,
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

                let processed_games: Vec<MonarchGame> = games
                    .iter()
                    .cloned()
                    .map(|mut game| {
                        game.thumbnail_path = "".to_string();
                        game
                    })
                    .collect();

                // Trigger download tasks
                let download_tasks = iced::Task::batch(games.into_iter().map(|mut game| {
                    iced::Task::perform(
                        async move {
                            info!("Downloading artwork for: {}", game.name);
                            let _ = monarch_games::commands::download_artwork(&game).await;

                            info!("Downloading cover for: {}", game.name);
                            if let Err(e) = monarch_games::commands::download_thumbnail(&game).await
                            {
                                error!("Failed to download thumbnail for game {}: {}", game.id, e);
                            }

                            info!("Updating game properties for : {}", game.name);
                            monarch_games::commands::get_game_properties(&mut game).await;
                            game
                        },
                        Message::GameUpdated,
                    )
                }));

                // Update browser games
                let _ = self
                    .browser
                    .update(gamecard::GameCardMessage::UpdateGames(processed_games));

                download_tasks
            }
            Message::UpdateGameProperties => {
                // Trigger download tasks
                let update_tasks = iced::Task::batch(self.browser.games.games.iter().cloned().map(
                    |mut gamecard| {
                        iced::Task::perform(
                            async move {
                                monarch_games::commands::get_game_properties(&mut gamecard.game)
                                    .await;
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
                    .browser
                    .games
                    .games
                    .iter_mut()
                    .find(|c| c.game.id == game.id)
                {
                    card.game = game;
                }
                iced::Task::none()
            }
            Message::GameCard(game_card_message) => {
                // Check if it's a game press event
                if let gamecard::GameCardMessage::GamePressed(id) = &game_card_message {
                    // Find the game and emit OpenGameDetails
                    if let Some(game_card) =
                        self.browser.games.games.iter().find(|g| g.game.id == *id)
                    {
                        return iced::Task::done(Message::OpenGameDetails(game_card.game.clone()));
                    }
                }

                self.browser
                    .update(game_card_message)
                    .map(Message::GameCard)
            }
            Message::OpenGameDetails(_) => {
                // This will be handled by the parent App
                iced::Task::none()
            }
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
            Message::ScannerHovered(hovered) => {
                self.is_scanner_hovered = hovered;
                iced::Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let refresh_btn = mouse_area(scanner_button(
            "Scan for games",
            Some(Message::RefreshLibrary),
            self.is_scanner_hovered,
        ))
        .on_enter(Message::ScannerHovered(true))
        .on_exit(Message::ScannerHovered(false));

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
                    .padding(30)
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
                show_error("Failed to get library!");
            }
        }

        Self {
            browser: browser,
            is_refreshing: false,
            dot_count: 3,
            tick_counter: 0,
            is_scanner_hovered: false,
        }
    }
}
