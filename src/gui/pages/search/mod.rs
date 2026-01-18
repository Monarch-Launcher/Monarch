use std::collections::HashMap;

use iced::widget::{button, column, container, image, row, scrollable, text, text_input};
use iced::{alignment, Color, Element, Length};

use crate::monarch_games;
use crate::monarch_games::monarchgame::MonarchGame;

#[derive(Clone, Debug)]
pub enum Message {
    SearchChanged(String),
    FiltersPressed,
    PerformSearch,
    UpdateGames(Vec<MonarchGame>),
    GameImgLoaded(MonarchGame),
    GamePressed(String),
}

#[derive(Default)]
pub struct SearchPage {
    search_value: String,
    games: HashMap<String, MonarchGame>,
    is_searching: bool,
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
                let search_term = self.search_value.clone();
                iced::Task::perform(
                    async move { monarch_games::commands::search_games(search_term, true).await },
                    Message::UpdateGames,
                )
            }
            Message::UpdateGames(games) => {
                self.is_searching = false;

                self.games = games
                    .iter()
                    .map(|game| {
                        let mut game = game.clone();
                        game.thumbnail_path = "icons/icon.png".to_string();
                        (game.id.clone(), game)
                    })
                    .collect();

                iced::Task::batch(games.into_iter().map(|game| {
                    iced::Task::perform(
                        async move {
                            let _ = monarch_games::commands::download_thumbnail(game.clone()).await;
                            game
                        },
                        Message::GameImgLoaded,
                    )
                }))
            }
            Message::GameImgLoaded(game) => {
                if let Some(g) = self.games.get_mut(&game.id) {
                    g.thumbnail_path = game.thumbnail_path.clone();
                }
                iced::Task::none()
            }
            Message::GamePressed(_id) => {
                // TODO: Show game details
                iced::Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let search_input = text_input("Search for games ...", &self.search_value)
            .on_input(Message::SearchChanged)
            .on_submit(Message::PerformSearch)
            .style(crate::gui::styles::text_input::search)
            .padding(15)
            .width(Length::Fixed(600.0))
            .size(16);

        let filters_button = button(text("Filters").align_x(alignment::Horizontal::Center))
            .on_press(Message::FiltersPressed)
            .padding(10)
            .style(crate::gui::styles::header::button);

        let search_bar = row![search_input, filters_button]
            .spacing(15)
            .align_y(alignment::Vertical::Center);

        let mut games_column = column![].spacing(30).align_x(alignment::Horizontal::Center);

        // Convert HashMap to a sorted Vec for consistent grid rendering
        let mut sorted_games: Vec<&MonarchGame> = self.games.values().collect();
        sorted_games.sort_by(|a, b| a.name.cmp(&b.name));

        // Grid layout for games
        let games_per_row = 5;
        for chunk in sorted_games.chunks(games_per_row) {
            let mut row = row![].spacing(20);
            for game in chunk {
                row = row.push(self.view_game_card(game));
            }
            games_column = games_column.push(row);
        }

        let games_content = if self.is_searching {
            container(
                column![
                    text("Searching for games...")
                        .size(32)
                        .style(|_theme: &iced::Theme| text::Style {
                            color: Some(Color::from_rgb8(255, 127, 0)),
                        }),
                    text("Sifting through the library...").size(16).style(
                        |_theme: &iced::Theme| text::Style {
                            color: Some(Color::from_rgb8(150, 150, 150)),
                        }
                    ),
                ]
                .spacing(20)
                .align_x(alignment::Horizontal::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(100)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
        } else if self.games.is_empty() && !self.search_value.is_empty() {
            container(
                text("No games found.")
                    .size(24)
                    .style(|_theme: &iced::Theme| text::Style {
                        color: Some(Color::from_rgb8(100, 100, 100)),
                    }),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
        } else {
            container(scrollable(
                container(games_column)
                    .width(Length::Fill)
                    .padding(20)
                    .align_x(alignment::Horizontal::Center),
            ))
            .width(Length::Fill)
            .height(Length::Fill)
        };

        container(
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
        .height(Length::Fill)
        .into()
    }

    fn view_game_card(&self, game: &MonarchGame) -> Element<'_, Message> {
        let image_widget: Element<'_, Message> = if game.thumbnail_path.is_empty() {
            container(text(game.name.clone()).align_x(alignment::Horizontal::Center))
                .width(Length::Fixed(240.0))
                .height(Length::Fixed(360.0))
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
                .style(|_theme: &iced::Theme| container::Style {
                    background: Some(Color::from_rgb8(30, 30, 35).into()),
                    border: iced::Border {
                        color: Color::from_rgb8(50, 50, 60),
                        width: 1.0,
                        radius: 10.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        } else {
            container(
                image(game.thumbnail_path.clone())
                    .width(Length::Fixed(240.0))
                    .height(Length::Fixed(360.0))
                    .content_fit(iced::ContentFit::Cover),
            )
            .clip(true)
            .style(|_theme: &iced::Theme| container::Style {
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 10.0.into(),
                },
                ..Default::default()
            })
            .into()
        };

        button(image_widget)
            .on_press(Message::GamePressed(game.id.clone()))
            .padding(0)
            .style(|_theme: &iced::Theme, _status| button::Style {
                background: None,
                border: iced::Border {
                    width: 0.0,
                    radius: 10.0.into(),
                    color: Color::TRANSPARENT,
                },
                ..Default::default()
            })
            .into()
    }
}
