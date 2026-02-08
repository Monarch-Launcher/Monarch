use iced::{
    alignment,
    widget::{column, container, image, row, scrollable, stack, text, Space},
    Color, Element, Length, Theme,
};

use crate::{
    gui::{
        components::common::{launch_button, secondary_button},
        pages::game_details::{GameDetailsPage, Message},
    },
    monarch_games::{games::GameType, monarchgame::MonarchGame},
};

impl GameDetailsPage {
    pub fn view_game_details<'a>(&'a self, game: &'a MonarchGame) -> Element<'a, Message> {
        // Determine background image with fallback logic
        let background_image = if !game.artwork_path.is_empty() {
            container(
                image(game.artwork_path.clone())
                    .width(Length::Fill)
                    .height(800)
                    .content_fit(iced::ContentFit::Cover),
            )
        } else if !game.thumbnail_path.is_empty() {
            container(
                image(game.thumbnail_path.clone())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .content_fit(iced::ContentFit::Cover),
            )
        } else {
            // Fallback to dark gradient
            container(text(""))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme: &Theme| container::Style {
                    background: Some(
                        iced::gradient::Linear::new(iced::Radians(0.0))
                            .add_stop(0.0, Color::from_rgb8(20, 20, 30))
                            .add_stop(1.0, Color::from_rgb8(10, 10, 15))
                            .into(),
                    ),
                    ..Default::default()
                })
        };

        // Dark overlay for readability
        let overlay = container(column![])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(
                    iced::gradient::Linear::new(iced::Radians(std::f32::consts::PI))
                        .add_stop(0.5, Color::from_rgba8(10, 10, 17, 0.0))
                        .add_stop(0.6, Color::from_rgba8(10, 10, 17, 1.0))
                        .into(),
                ),
                ..Default::default()
            });

        // Back button in top left
        let back_btn =
            container(secondary_button("← Back", Some(Message::BackPressed))).padding(40);

        // Game cover/thumbnail
        let game_cover = if !game.thumbnail_path.is_empty() {
            image(game.thumbnail_path.clone())
                .width(Length::Fixed(300.0))
                .height(Length::Fixed(450.0))
                .content_fit(iced::ContentFit::Cover)
        } else {
            image(crate::gui::resources::ICON.clone())
                .width(Length::Fixed(300.0))
                .height(Length::Fixed(450.0))
                .content_fit(iced::ContentFit::Cover)
        };

        let cover_container = container(game_cover).style(|_theme: &Theme| container::Style {
            border: iced::Border {
                color: Color::from_rgb8(255, 127, 0),
                width: 3.0,
                radius: 8.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba8(0, 0, 0, 0.8),
                offset: iced::Vector::new(0.0, 10.0),
                blur_radius: 30.0,
            },
            ..Default::default()
        });

        // Title
        let title = text(&game.name)
            .size(42)
            .color(Color::WHITE)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            });

        // Platform/Store info
        let platform_badge = container(
            text(game.get_store_name())
                .size(14)
                .color(Color::WHITE)
                .font(iced::Font {
                    weight: iced::font::Weight::Semibold,
                    ..Default::default()
                }),
        )
        .padding(iced::Padding::from([6, 12]))
        .style(|_theme: &Theme| container::Style {
            background: Some(Color::from_rgb8(255, 127, 0).into()),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        // Description section
        let description_title = text("About")
            .size(20)
            .color(Color::from_rgb8(200, 200, 200))
            .font(iced::Font {
                weight: iced::font::Weight::Semibold,
                ..Default::default()
            });

        let description_text = text(if game.summary.is_empty() {
            "No description available."
        } else {
            &game.summary
        })
        .size(15)
        .color(Color::from_rgb8(180, 180, 180))
        .line_height(iced::widget::text::LineHeight::Relative(1.7));

        // Technical details
        let install_size_display = if game.install_size.is_empty() {
            "Unknown"
        } else {
            &game.install_size
        };

        let version_display = if game.version.is_empty() {
            "Unknown"
        } else {
            &game.version
        };

        let play_time_display = if game.play_time.is_empty() {
            "0 hours"
        } else {
            &game.play_time
        };

        let last_played_display = if game.last_played.is_empty() {
            "Never"
        } else {
            &game.last_played
        };

        let tech_details = container(
            row![
                column![
                    text("Install Size")
                        .size(12)
                        .color(Color::from_rgb8(140, 140, 140)),
                    text(install_size_display)
                        .size(14)
                        .color(Color::from_rgb8(220, 220, 220))
                        .font(iced::Font {
                            weight: iced::font::Weight::Medium,
                            ..Default::default()
                        }),
                ]
                .spacing(4),
                column![
                    text("Version")
                        .size(12)
                        .color(Color::from_rgb8(140, 140, 140)),
                    text(version_display)
                        .size(14)
                        .color(Color::from_rgb8(220, 220, 220))
                        .font(iced::Font {
                            weight: iced::font::Weight::Medium,
                            ..Default::default()
                        }),
                ]
                .spacing(4),
                column![
                    text("Playtime")
                        .size(12)
                        .color(Color::from_rgb8(140, 140, 140)),
                    text(play_time_display)
                        .size(14)
                        .color(Color::from_rgb8(220, 220, 220))
                        .font(iced::Font {
                            weight: iced::font::Weight::Medium,
                            ..Default::default()
                        }),
                ]
                .spacing(4),
                column![
                    text("Last Played")
                        .size(12)
                        .color(Color::from_rgb8(140, 140, 140)),
                    text(last_played_display)
                        .size(14)
                        .color(Color::from_rgb8(220, 220, 220))
                        .font(iced::Font {
                            weight: iced::font::Weight::Medium,
                            ..Default::default()
                        }),
                ]
                .spacing(4),
            ]
            .spacing(30),
        )
        .padding(iced::Padding::from([15, 20]))
        .style(|_theme: &Theme| container::Style {
            background: Some(Color::from_rgba8(255, 255, 255, 0.05).into()),
            border: iced::Border {
                radius: 8.0.into(),
                color: Color::from_rgba8(255, 255, 255, 0.1),
                width: 1.0,
            },
            ..Default::default()
        });

        let launch_btn = launch_button("Launch", Some(Message::LaunchGame));
        let edit_btn = secondary_button("Edit", Some(Message::OpenProperties));

        // Properties section
        let properties_title = text("Properties")
            .size(20)
            .color(Color::from_rgb8(200, 200, 200))
            .font(iced::Font {
                weight: iced::font::Weight::Semibold,
                ..Default::default()
            });

        let property_item = |label: &'a str, value: &'a str| -> Element<'a, Message> {
            column![
                text(label).size(12).color(Color::from_rgb8(140, 140, 140)),
                text(if value.is_empty() { "Not set" } else { value })
                    .size(14)
                    .color(Color::from_rgb8(200, 200, 200)),
            ]
            .spacing(4)
            .into()
        };

        let properties_panel = container(
            column![
                row![properties_title, Space::new().width(Length::Fill), edit_btn]
                    .align_y(alignment::Vertical::Center),
                container(
                    column![
                        property_item("Executable", &game.executable_path),
                        property_item("Compatibility", &game.compatibility),
                        property_item("Launch Arguments", &game.launch_args),
                    ]
                    .spacing(16)
                )
                .padding(20)
                .style(|_theme: &Theme| container::Style {
                    background: Some(Color::from_rgba8(255, 255, 255, 0.03).into()),
                    border: iced::Border {
                        radius: 8.0.into(),
                        color: Color::from_rgba8(255, 255, 255, 0.05),
                        width: 1.0,
                    },
                    ..Default::default()
                })
                .width(Length::Fill)
            ]
            .spacing(15),
        )
        .width(Length::FillPortion(1));

        let header_panel = column![title, platform_badge].spacing(12);

        // Right side content (info panel)
        let info_panel = column![
            description_title,
            description_text,
            container(text("")).height(Length::Fixed(20.0)), // Spacer
            tech_details,
        ]
        .spacing(12)
        .width(Length::FillPortion(2));

        // Main content area with horizontal layout
        let main_content = column![
            cover_container,
            header_panel,
            row![launch_btn]
                .spacing(10)
                .align_y(alignment::Vertical::Center),
            row![info_panel, properties_panel].spacing(40),
        ]
        .spacing(40);

        // Full content with back button and scrollable area
        let content = scrollable(
            column![
                back_btn,
                container(main_content).width(Length::Fill).padding(
                    iced::Padding::new(0.0)
                        .top(0.0)
                        .right(40.0)
                        .bottom(40.0)
                        .left(40.0)
                ),
            ]
            .align_x(alignment::Horizontal::Left),
        )
        .width(Length::Fill)
        .height(Length::Fill);

        // Stack everything: background -> overlay -> content
        let mut layers = stack![background_image, overlay, content]
            .width(Length::Fill)
            .height(Length::Fill);

        if let Some(modal) = &self.properties_modal {
            layers = layers.push(modal.view().map(Message::Properties));
        }

        layers.into()
    }
}
