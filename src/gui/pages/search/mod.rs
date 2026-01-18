use iced::widget::{button, column, container, row, scrollable, stack, text, text_input};
use iced::{alignment, Color, Element, Length};
use tracing::debug;

use crate::gui::components::gamecard;
use crate::gui::components::gamecard::container::GameCardContainer;
use crate::gui::components::gamecard::drawer::{DrawerMessage, GameDrawer};
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
    Drawer(DrawerMessage),
    Tick,
}

#[derive(Default)]
pub struct SearchPage {
    search_value: String,
    games: GameCardContainer,
    is_searching: bool,
    selected_game: Option<MonarchGame>,
    // Animation state: 0.0 (closed) to 1.0 (open)
    drawer_animation: f32,
    is_closing: bool,
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
                    .cloned()
                    .map(|mut game| {
                        game.thumbnail_path = "icons/icon.png".to_string();
                        game
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
            Message::GameCard(game_card_message) => {
                match &game_card_message {
                    gamecard::GameCardMessage::GamePressed(id) => {
                        if let Some(game_card) = self.games.games.iter().find(|g| g.game.id == *id)
                        {
                            self.selected_game = Some(game_card.game.clone());
                            self.is_closing = false;
                            // Start animation from current state (usually 0.0 if closed)
                        }
                    }
                    _ => {}
                }
                self.games.update(game_card_message).map(Message::GameCard)
            }
            Message::Drawer(drawer_msg) => {
                match drawer_msg {
                    DrawerMessage::Close => {
                        self.is_closing = true;
                    }
                    _ => {}
                }
                iced::Task::none()
            }
            Message::Tick => {
                // Animation Logic
                let target = if self.is_closing || self.selected_game.is_none() {
                    0.0
                } else {
                    1.0
                };
                let speed = 0.2; // Animation speed

                if (self.drawer_animation - target).abs() > 0.001 {
                    self.drawer_animation += (target - self.drawer_animation) * speed;
                } else {
                    self.drawer_animation = target;
                    if self.is_closing && target == 0.0 {
                        self.selected_game = None;
                        self.is_closing = false;
                    }
                }

                self.games
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
            .into()
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
            .into()
        } else {
            container(scrollable(
                container(self.games.view().map(Message::GameCard))
                    .width(Length::Fill)
                    .padding(20)
                    .align_x(alignment::Horizontal::Center),
            ))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
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
        if let Some(game) = &self.selected_game {
            iced::widget::responsive(move |size| {
                let drawer_width = size.width * 0.5;
                let padding_left = size.width - (drawer_width * self.drawer_animation);

                let drawer_layer = container(GameDrawer::new(game).view().map(Message::Drawer))
                    .width(Length::Fixed(drawer_width))
                    .height(Length::Fill);

                let drawer_wrapper = container(drawer_layer)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(iced::Padding::default().left(padding_left))
                    .align_x(alignment::Horizontal::Left);

                stack![
                    self.content_view(),
                    container(
                        iced::widget::mouse_area(
                            container(text(""))
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .style(move |_theme: &iced::Theme| container::Style {
                                    background: Some(
                                        Color::from_rgba8(0, 0, 0, 0.5 * self.drawer_animation)
                                            .into()
                                    ),
                                    ..Default::default()
                                })
                        )
                        .on_press(Message::Drawer(DrawerMessage::Close))
                    )
                    .width(Length::Fill)
                    .height(Length::Fill),
                    drawer_wrapper
                ]
                .into()
            })
            .into()
        } else {
            self.content_view()
        }
    }
}
