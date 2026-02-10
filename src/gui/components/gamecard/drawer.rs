use iced::widget::{column, container, image, row, scrollable, stack, text};
use iced::{alignment, Color, Element, Length, Theme};

use crate::gui::components::gamecard::GameCardMessage;
use crate::monarch_games::monarchgame::{MonarchGame, MonarchWebApiPlatform};

#[derive(Default, Debug, Clone)]
pub struct GameDrawer {}

impl GameDrawer {
    pub fn view<'a>(
        &self,
        game: &'a MonarchGame,
        stores: Vec<MonarchWebApiPlatform>,
    ) -> Element<'a, GameCardMessage> {
        let background_image = if game.thumbnail_path.is_empty() {
            container(text(""))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme: &Theme| container::Style {
                    background: Some(Color::BLACK.into()),
                    ..Default::default()
                })
        } else {
            container(
                image(game.thumbnail_path.clone())
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

        let store_names: String = stores.iter().map(|s| format!(" {} ", s.name)).collect();
        let content = scrollable(
            column![
                // Header Image
                container(
                    if game.thumbnail_path.is_empty() {
                        image(crate::gui::resources::ICON.clone())
                    } else {
                        image(game.thumbnail_path.clone())
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
                text(&game.name)
                    .size(32)
                    .color(Color::WHITE)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                // Stores / Platform
                row![
                    text("Available at:").color(Color::from_rgb8(180, 180, 180)),
                    text(store_names).color(Color::from_rgb8(255, 127, 0))
                ]
                .spacing(10),
                // Description
                text("Description").size(20).color(Color::WHITE),
                text(if game.summary.is_empty() {
                    "No description available."
                } else {
                    &game.summary
                })
                .color(Color::from_rgb8(200, 200, 200)),
                row(stores
                    .iter()
                    .filter(|s| !s.store_page.is_empty())
                    .map(|s| {
                        crate::gui::components::common::primary_button(
                            &format!("{} Store", s.name),
                            Some(GameCardMessage::OpenStorePage(s.store_page.clone())),
                        )
                    })
                    .collect::<Vec<Element<'a, GameCardMessage>>>())
                .spacing(10),
            ]
            .spacing(20)
            .padding(40)
            .align_x(alignment::Horizontal::Center),
        )
        .width(Length::Fill);

        let content_stack = stack![background_image, overlay, container(content).padding(20),]
            .width(Length::Fill)
            .height(Length::Fill);

        iced::widget::mouse_area(content_stack)
            .on_press(GameCardMessage::NoOp)
            .into()
    }
}
