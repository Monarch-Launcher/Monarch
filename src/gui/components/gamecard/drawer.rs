use iced::widget::{column, container, image, row, scrollable, stack, text};
use iced::{alignment, Color, Element, Length, Theme};

use crate::monarch_games::monarchgame::MonarchGame;

#[derive(Clone, Debug)]
pub enum DrawerMessage {
    Close,
    Launch,
}

pub struct GameDrawer<'a> {
    game: &'a MonarchGame,
}

impl<'a> GameDrawer<'a> {
    pub fn new(game: &'a MonarchGame) -> Self {
        Self { game }
    }

    pub fn view(&self) -> Element<'a, DrawerMessage> {
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

        stack![background_image, overlay, container(content).padding(20),]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
