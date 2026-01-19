use iced::widget::{column, container, image, row, scrollable, stack, text};
use iced::{alignment, Color, Element, Length, Theme};

use crate::gui::components::gamecard::GameCardMessage;
use crate::monarch_games::monarchgame::MonarchGame;

pub struct GameDrawer<'a> {
    game: &'a MonarchGame,

    // Animation state: 0.0 (closed) to 1.0 (open)
    drawer_animation: f32,
    closed: bool,
}

impl<'a> GameDrawer<'a> {
    pub fn new(game: &'a MonarchGame) -> Self {
        Self { game, drawer_animation: 0.0, closed: true}
    }

    pub fn update(&mut self, msg: GameCardMessage) -> iced::Task<GameCardMessage>{
        match msg {
            GameCardMessage::OpenDrawer => { iced::Task::none() }
            GameCardMessage::CloseDrawer => {iced::Task::none()}
            GameCardMessage::Tick => {
                // Animation Logic
                let target = if self.closed {
                    0.0
                } else {
                    1.0
                };
                let speed = 0.2; // Animation speed

                if (self.drawer_animation - target).abs() > 0.001 {
                    self.drawer_animation += (target - self.drawer_animation) * speed;
                } else {
                    self.drawer_animation = target;
                    if self.closed && target == 0.0 {
                        self.closed = false;
                    }
                }
                iced::Task::none()
            }
            _ => { iced::Task::none() }
        }

    }

    pub fn view(&self) -> Element<'a, GameCardMessage> {
        let background_image = if self.game.thumbnail_path.is_empty() {
            container(text(""))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme: &Theme| container::Style {
                    background: Some(Color::BLACK.into()),
                    ..Default::default()
                })
        } else {
            container(
                image(self.game.thumbnail_path.clone())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .content_fit(iced::ContentFit::Cover),
            )
        };

        
        let overlay = container(column![])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(Color::from_rgba8(0, 0, 0, 0.90).into()),
                ..Default::default()
            });

        if self.drawer_animation == 1.0 {
let content = scrollable(
            column![
                // Header Image
                container(
                    if self.game.thumbnail_path.is_empty() {
                        image(crate::gui::resources::ICON.clone())
                    } else {
                        image(self.game.thumbnail_path.clone())
                    }
                    .width(Length::Fixed(200.0))
                    .height(Length::Fixed(300.0))
                    .content_fit(iced::ContentFit::Cover)
                )
                .style(|_theme: &Theme| container::Style {
                    border: iced::Border {
                        color: Color::WHITE,
                        width: 2.0,
                        radius: 12.0.into(),
                    },
                    shadow: iced::Shadow {
                        color: Color::BLACK,
                        offset: iced::Vector::new(0.0, 5.0),
                        blur_radius: 20.0,
                    },
                    ..Default::default()
                })
                .align_x(alignment::Horizontal::Center),
                // Title
                text(&self.game.name)
                    .size(32)
                    .color(Color::WHITE)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                // Stores / Platform
                row![
                    text("Available at:").color(Color::from_rgb8(180, 180, 180)),
                    text(&self.game.platform).color(Color::from_rgb8(255, 127, 0))
                ]
                .spacing(10),
                // Description
                text("Description").size(20).color(Color::WHITE),
                text(if self.game.summary.is_empty() {
                    "No description available."
                } else {
                    &self.game.summary
                })
                .color(Color::from_rgb8(200, 200, 200))
            ]
            .spacing(20)
            .padding(40)
            .align_x(alignment::Horizontal::Center),
        )
        .width(Length::Fill);

    return stack![background_image, overlay, container(content).padding(20),]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        }
        

        stack![background_image, overlay,]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
