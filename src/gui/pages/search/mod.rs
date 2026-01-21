use iced::widget::{button, column, container, row, text, text_input};
use iced::{alignment, Color, Element, Length};
use tracing::error;

use crate::gui::components::gamecard;
use crate::monarch_games;
use crate::monarch_games::monarchgame::MonarchGame;

#[derive(Clone, Debug)]
pub enum Message {
    SearchChanged(String),
    FiltersPressed,
    PerformSearch,
    UpdateGames(Vec<MonarchGame>),
    GameImgLoaded(MonarchGame),
    GameCard(gamecard::GameCardMessage),
    Tick,
}

use crate::gui::components::gamecard::game_browser::GameBrowser;

#[derive(Default)]
pub struct SearchPage {
    search_value: String,
    browser: GameBrowser,
    is_searching: bool,
    dot_count: u8,
    tick_counter: u8,
}

impl SearchPage {
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
            Message::PerformSearch => {
                self.is_searching = true;
                self.dot_count = 3;
                self.tick_counter = 0;
                let search_term = self.search_value.clone();
                iced::Task::perform(
                    async move { monarch_games::commands::search_games(search_term, true).await },
                    Message::UpdateGames,
                )
            }
            Message::UpdateGames(games) => {
                self.is_searching = false;

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
    }

    fn content_view(&self) -> Element<'_, Message> {
        let search_input = text_input("Search for games ...", &self.search_value)
            .on_input(Message::SearchChanged)
            .on_submit(Message::PerformSearch)
            .style(crate::gui::styles::text_input::search)
            .padding(15)
            .width(Length::Fixed(600.0))
            .size(20);

        let filters_button = button(text("Filters").align_x(alignment::Horizontal::Center))
            .on_press(Message::FiltersPressed)
            .padding(10)
            .style(crate::gui::styles::header::button);

        let search_bar = row![search_input, filters_button]
            .spacing(15)
            .align_y(alignment::Vertical::Center);

        let games_content: Element<'_, Message> = if self.is_searching {
            let dots = ".".repeat(self.dot_count as usize);
            container(
                column![text(format!("Searching for games{dots}")).size(32).style(
                    |_theme: &iced::Theme| text::Style {
                        color: Some(Color::from_rgb8(255, 127, 0)),
                    }
                ),]
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

        let content = container(
            column![
                container(search_bar)
                    .padding(30)
                    .width(Length::Fill)
                    .align_x(alignment::Horizontal::Center),
                games_content,
            ]
            .align_x(alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill);

        content.into()
    }

    pub fn view(&self) -> Element<'_, Message> {
        self.content_view()
    }
}
