use iced::{
    alignment,
    widget::{button, column, container, image, row, scrollable, text, Space},
    Color, Element, Length, Theme,
};

use super::{HomePage, Message};
use crate::gui::components::gamecard::GameCardMessage;
use crate::gui::{
    components::common::{launch_button, secondary_button},
    styles,
};

impl HomePage {
    pub fn view(&self) -> Element<'_, Message> {
        let content: Element<'_, Message> = if self.is_loading {
            container(
                text("Loading recommendations…")
                    .size(22)
                    .font(styles::fonts::REGULAR),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .into()
        } else {
            scrollable(
                column![
                    self.view_recommended(),
                    Space::new().height(40),
                    self.view_deals(),
                    Space::new().height(40),
                ]
                .width(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    // ── Recommended ──────────────────────────────────────────────────────────

    fn view_recommended(&self) -> Element<'_, Message> {
        let section_label = text("Jump Back In")
            .size(24)
            .font(crate::gui::styles::fonts::REGULAR);

        let cards: Element<'_, Message> = if self.recommended_games.is_empty() {
            container(
                text("Your library is empty — head to Search to find some games!")
                    .size(16)
                    .font(styles::fonts::REGULAR),
            )
            .width(Length::Fill)
            .padding(20)
            .into()
        } else {
            row(self
                .recommended_games
                .iter()
                .map(|card| {
                    let game = card.game.clone();
                    let launch_btn: Element<'_, Message> =
                        launch_button("Launch", Some(Message::LaunchGame(game)));

                    container(
                        column![
                            card.view_scaled(true, 1.25).map(Message::GameCard),
                            launch_btn,
                        ]
                        .spacing(8)
                        .align_x(alignment::Horizontal::Center),
                    )
                    .width(Length::Fill)
                    .align_x(alignment::Horizontal::Center)
                    .into()
                })
                .collect::<Vec<_>>())
            .width(Length::Fill)
            .into()
        };

        container(column![section_label, cards].spacing(20))
            .width(Length::Fill)
            .padding(
                iced::Padding::new(0.0)
                    .top(30.0)
                    .right(40.0)
                    .bottom(0.0)
                    .left(40.0),
            )
            .into()
    }

    // ── Deals of the Day ─────────────────────────────────────────────────────

    fn view_deals(&self) -> Element<'_, Message> {
        let section_label = text("Deals of the Day")
            .size(24)
            .font(crate::gui::styles::fonts::REGULAR);

        let deal_card: Element<'_, Message> = if self.deals.is_empty() {
            container(
                text("No deals at the moment.")
                    .size(16)
                    .font(styles::fonts::REGULAR),
            )
            .width(Length::Fill)
            .padding(20)
            .into()
        } else {
            let idx = self.current_deal_index;
            let game = &self.deals[idx];

            // Indicator dots
            let dots: Element<'_, Message> = row(self
                .deals
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    let active = i == idx;
                    container(Space::new())
                        .width(if active { 20 } else { 8 })
                        .height(8)
                        .style(move |_: &Theme| container::Style {
                            background: Some(
                                if active {
                                    Color::from_rgb8(255, 127, 0)
                                } else {
                                    Color::from_rgba8(255, 255, 255, 0.3)
                                }
                                .into(),
                            ),
                            border: iced::Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .into()
                })
                .collect::<Vec<_>>())
            .spacing(6)
            .align_y(alignment::Vertical::Center)
            .into();

            // Cover image
            let cover: Element<'_, Message> = if !game.thumbnail_path.is_empty() {
                image(game.thumbnail_path.clone())
                    .width(Length::Fixed(200.0))
                    .height(Length::Fixed(280.0))
                    .content_fit(iced::ContentFit::Cover)
                    .into()
            } else {
                image(crate::gui::resources::ICON.clone())
                    .width(Length::Fixed(200.0))
                    .height(Length::Fixed(280.0))
                    .content_fit(iced::ContentFit::Cover)
                    .into()
            };

            let cover_container = container(cover).style(|_: &Theme| container::Style {
                border: iced::Border {
                    color: Color::from_rgb8(255, 127, 0),
                    width: 2.0,
                    radius: 8.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba8(255, 127, 0, 0.25),
                    offset: iced::Vector::new(0.0, 6.0),
                    blur_radius: 20.0,
                },
                ..Default::default()
            });

            // Spoofed discount badge
            let discount_pct = 10 + (idx * 15) % 70;
            let badge_text = format!("-{}%", discount_pct);
            let deal_badge = container(
                text(badge_text)
                    .size(18)
                    .font(crate::gui::styles::fonts::BOLD),
            )
            .padding(iced::Padding::from([6, 14]))
            .style(|_: &Theme| container::Style {
                background: Some(Color::from_rgb8(255, 127, 0).into()),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

            // Store badge
            let store_name = if game.stores.is_empty() {
                "—".to_string()
            } else {
                game.stores[0].name.to_uppercase()
            };
            let store_badge = container(text(store_name).size(12).font(styles::fonts::REGULAR))
                .padding(iced::Padding::from([4, 10]))
                .style(|_: &Theme| container::Style {
                    background: Some(Color::from_rgba8(255, 255, 255, 0.1).into()),
                    border: iced::Border {
                        radius: 4.0.into(),
                        color: Color::from_rgba8(255, 255, 255, 0.15),
                        width: 1.0,
                    },
                    ..Default::default()
                });

            let game_title = text(game.name.clone())
                .size(32)
                .font(crate::gui::styles::fonts::BOLD);

            let description = text(if game.summary.is_empty() {
                "A great deal — don't miss out!".to_string()
            } else if game.summary.len() > 200 {
                format!("{}…", &game.summary[..200])
            } else {
                game.summary.clone()
            })
            .size(14)
            .font(styles::fonts::REGULAR)
            .line_height(iced::widget::text::LineHeight::Relative(1.6));

            // Nav arrows — built as closures to share style
            let arrow_style = |status: button::Status| button::Style {
                background: Some(
                    Color::from_rgba8(
                        255,
                        255,
                        255,
                        match status {
                            button::Status::Hovered => 0.12,
                            _ => 0.06,
                        },
                    )
                    .into(),
                ),
                border: iced::Border {
                    radius: 6.0.into(),
                    color: Color::from_rgba8(255, 255, 255, 0.15),
                    width: 1.0,
                },
                text_color: Color::WHITE,
                ..Default::default()
            };

            let prev_btn: Element<'_, Message> =
                button(text("‹").size(28).font(styles::fonts::BOLD))
                    .on_press(Message::PrevDeal)
                    .padding(iced::Padding::from([8, 14]))
                    .style(move |_: &Theme, status| arrow_style(status))
                    .into();

            let next_btn: Element<'_, Message> =
                button(text("›").size(28).font(styles::fonts::BOLD))
                    .on_press(Message::NextDeal)
                    .padding(iced::Padding::from([8, 14]))
                    .style(move |_: &Theme, status| arrow_style(status))
                    .into();

            let game_id = game.id.clone();
            let info_col = column![
                row![store_badge, deal_badge]
                    .spacing(8)
                    .align_y(alignment::Vertical::Center),
                game_title,
                Space::new().height(8),
                description,
                Space::new().height(16),
                secondary_button(
                    "View Details",
                    Some(Message::GameCard(GameCardMessage::GamePressed(game_id)))
                ),
            ]
            .spacing(12)
            .width(Length::Fill);

            let card_row = row![prev_btn, cover_container, info_col, next_btn]
                .spacing(30)
                .align_y(alignment::Vertical::Center)
                .width(Length::Fill);

            let bottom_row = row![
                Space::new().width(Length::Fill),
                dots,
                Space::new().width(Length::Fill),
            ]
            .align_y(alignment::Vertical::Center);

            container(column![card_row, Space::new().height(16), bottom_row].width(Length::Fill))
                .width(Length::Fill)
                .padding(30)
                .style(|_: &Theme| container::Style {
                    background: Some(Color::from_rgba8(255, 255, 255, 0.04).into()),
                    border: iced::Border {
                        radius: 12.0.into(),
                        color: Color::from_rgba8(255, 255, 255, 0.08),
                        width: 1.0,
                    },
                    ..Default::default()
                })
                .into()
        };

        container(column![section_label, deal_card].spacing(20))
            .width(Length::Fill)
            .padding(
                iced::Padding::new(0.0)
                    .top(0.0)
                    .right(40.0)
                    .bottom(0.0)
                    .left(40.0),
            )
            .into()
    }
}
