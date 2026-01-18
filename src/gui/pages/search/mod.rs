use std::collections::HashMap;

use iced::widget::{
    button, column, container, image, mouse_area, responsive, row, scrollable, text, text_input,
};
use iced::{alignment, Color, Element, Length};
use tracing::debug;

use crate::monarch_games;
use crate::monarch_games::monarchgame::MonarchGame;

#[derive(Clone, Debug)]
pub enum Message {
    SearchChanged(String),
    FiltersPressed,
    PerformSearch,
    UpdateGames(Vec<MonarchGame>),
    GameImgLoaded(MonarchGame),
    GameHovered(String),
    GameUnhovered,
    GamePressed(String),
    Tick,
}

#[derive(Default)]
pub struct SearchPage {
    search_value: String,
    games: HashMap<String, MonarchGame>,
    is_searching: bool,
    hovered_id: Option<String>,
    hover_factors: HashMap<String, f32>, // 0.0 to 1.0 for each game animation
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
                debug!("{} Games found!", games.len());

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
            Message::GameHovered(id) => {
                self.hovered_id = Some(id);
                iced::Task::none()
            }
            Message::GameUnhovered => {
                self.hovered_id = None;
                iced::Task::none()
            }
            Message::GamePressed(_id) => {
                // TODO: Show game details
                iced::Task::none()
            }
            Message::Tick => {
                let speed = 0.25; // Animation speed (lerp factor)
                let mut _changed = false;

                // Update factors for all games to reach target (1.0 if hovered, 0.0 if not)
                for game_id in self.games.keys() {
                    let target = if self.hovered_id.as_ref() == Some(game_id) {
                        1.0
                    } else {
                        0.0
                    };
                    let current = self.hover_factors.get(game_id).copied().unwrap_or(0.0);

                    if (current - target).abs() > 0.001 {
                        let new_val = current + (target - current) * speed;
                        self.hover_factors.insert(game_id.clone(), new_val);
                        _changed = true;
                    } else if current != target {
                        self.hover_factors.insert(game_id.clone(), target);
                        _changed = true;
                    }
                }

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

        // Convert HashMap to a sorted Vec for consistent grid rendering
        let mut sorted_games_vec: Vec<&MonarchGame> = self.games.values().collect();
        sorted_games_vec.sort_by(|a, b| a.name.cmp(&b.name));

        let games_grid = responsive(move |size| {
            // Calculate how many games can fit in the available width
            // base_width (240) + spacing (30) + some margin
            let card_width = 240.0 + 30.0;
            let games_per_row = (size.width / card_width).floor().max(1.0) as usize;

            let mut games_column = column![].spacing(30).align_x(alignment::Horizontal::Center);

            for chunk in sorted_games_vec.chunks(games_per_row) {
                let mut row = row![].spacing(30);
                for game in chunk {
                    row = row.push(self.view_game_card(game));
                }
                games_column = games_column.push(row);
            }

            games_column.into()
        });

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
        } else if self.games.is_empty() {
            container(
                column![
                    text(if self.search_value.is_empty() {
                        "Explore Games"
                    } else {
                        "No games found"
                    })
                    .size(32)
                    .style(|_theme: &iced::Theme| text::Style {
                        color: Some(Color::from_rgb8(100, 100, 100)),
                    }),
                    text(if self.search_value.is_empty() {
                        "Type something to start searching..."
                    } else {
                        "Try a different search term"
                    })
                    .size(16)
                    .style(|_theme: &iced::Theme| text::Style {
                        color: Some(Color::from_rgb8(80, 80, 80)),
                    }),
                ]
                .spacing(10)
                .align_x(alignment::Horizontal::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
        } else {
            container(scrollable(
                container(games_grid)
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
        let is_hovered = self.hovered_id.as_ref() == Some(&game.id);
        let factor = self.hover_factors.get(&game.id).copied().unwrap_or(0.0);

        let (base_width, base_height) = (240.0, 360.0);
        let scale = 1.0 + (factor * 0.05);
        let (width, height) = (base_width * scale, base_height * scale);

        let image_widget: Element<'_, Message> = if game.thumbnail_path.is_empty() {
            container(text(game.name.clone()).align_x(alignment::Horizontal::Center))
                .width(Length::Fixed(width))
                .height(Length::Fixed(height))
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
                .style(|_theme: &iced::Theme| container::Style {
                    background: Some(Color::from_rgb8(30, 30, 35).into()),
                    border: iced::Border {
                        color: Color::from_rgb8(50, 50, 60),
                        width: 1.0,
                        radius: 12.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        } else {
            container(
                image(game.thumbnail_path.clone())
                    .width(Length::Fixed(width))
                    .height(Length::Fixed(height))
                    .content_fit(iced::ContentFit::Cover),
            )
            .clip(true)
            .style(move |_theme: &iced::Theme| container::Style {
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: if is_hovered { 2.0 } else { 0.0 },
                    radius: 12.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba8(0, 0, 0, 0.6 * factor),
                    offset: iced::Vector::new(0.0, 10.0 * factor),
                    blur_radius: 20.0 * factor,
                },
                ..Default::default()
            })
            .into()
        };

        let card_button = button(image_widget)
            .on_press(Message::GamePressed(game.id.clone()))
            .padding(0)
            .style(|_theme: &iced::Theme, _status| button::Style {
                background: None,
                border: iced::Border {
                    width: 0.0,
                    radius: 12.0.into(),
                    color: Color::TRANSPARENT,
                },
                ..Default::default()
            });

        // Use mouse_area to detect hover and apply the "moving uplift" effect via padding
        mouse_area(
            container(card_button)
                .padding(10.0 * (1.0 - factor)) // Smooth padding transition
                .width(base_width + 20.0)
                .height(base_height + 20.0)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center),
        )
        .on_enter(Message::GameHovered(game.id.clone()))
        .on_exit(Message::GameUnhovered)
        .into()
    }
}
