use chrono::{TimeZone, Utc};
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
    pub fn view_game_details(&self) -> Element<'_, Message> {
        let game: MonarchGame = self.game.as_ref().unwrap().lock().unwrap().clone();

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
            image(crate::gui::resources::LOGO_LARGE.clone())
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
        let title = text(game.name.clone())
            .size(42)
            .color(Color::WHITE)
            .font(crate::gui::styles::fonts::REGULAR);

        // Store info
        let store_badge = container(
            text(game.get_store_name().to_uppercase())
                .size(14)
                .color(Color::WHITE)
                .font(crate::gui::styles::fonts::REGULAR),
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
            .font(crate::gui::styles::fonts::REGULAR);

        let description_text = text(if game.summary.is_empty() {
            "No description available.".to_string()
        } else {
            game.summary.clone()
        })
        .size(15)
        .color(Color::from_rgb8(180, 180, 180))
        .line_height(iced::widget::text::LineHeight::Relative(1.7));

        // Technical details
        let install_size_display = format_size(game.properties.size_on_disk as f64);

        let version_display = if game.properties.version.is_empty() {
            "Unknown".to_string()
        } else {
            game.properties.version.clone()
        };

        let play_time_display = if game.properties.time_played.is_empty() {
            "0 hours".to_string()
        } else {
            game.properties.time_played.clone()
        };

        let last_played_display = if game.properties.last_played.is_empty() {
            "Unknown".to_string()
        } else {
            Utc.timestamp_opt(game.properties.last_played.parse::<i64>().unwrap(), 0)
                .unwrap()
                .to_string()
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
                        .font(crate::gui::styles::fonts::MEDIUM),
                ]
                .spacing(4),
                column![
                    text("Version")
                        .size(12)
                        .color(Color::from_rgb8(140, 140, 140)),
                    text(version_display)
                        .size(14)
                        .color(Color::from_rgb8(220, 220, 220))
                        .font(crate::gui::styles::fonts::MEDIUM),
                ]
                .spacing(4),
                column![
                    text("Playtime")
                        .size(12)
                        .color(Color::from_rgb8(140, 140, 140)),
                    text(play_time_display)
                        .size(14)
                        .color(Color::from_rgb8(220, 220, 220))
                        .font(crate::gui::styles::fonts::MEDIUM),
                ]
                .spacing(4),
                column![
                    text("Last Played")
                        .size(12)
                        .color(Color::from_rgb8(140, 140, 140)),
                    text(last_played_display)
                        .size(14)
                        .color(Color::from_rgb8(220, 220, 220))
                        .font(crate::gui::styles::fonts::MEDIUM),
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
            .font(crate::gui::styles::fonts::SEMIBOLD);

        let property_item = |label: String, value: String| -> Element<'_, Message> {
            column![
                text(label).size(12).color(Color::from_rgb8(140, 140, 140)),
                text(if value.is_empty() {
                    "Not set".to_string()
                } else {
                    value
                })
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
                        property_item(
                            "Executable".to_string(),
                            game.executable_path.unwrap_or_default()
                        ),
                        property_item(
                            "Compatibility".to_string(),
                            game.compatibility.unwrap_or_default()
                        ),
                        property_item(
                            "Launch Arguments".to_string(),
                            game.launch_args.unwrap_or_default()
                        ),
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

        let header_panel = column![title, store_badge].spacing(12);

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

fn format_size(bytes: f64) -> String {
    let prefixes: [&str; 4] = ["B", "KiB", "MiB", "GiB"];
    let k = 1024 as f64;
    let i = (bytes.log2() / k.log2()).floor();
    let size = bytes / (k.powi(i as i32));

    if size.is_nan() {
        return "0 B".to_string();
    }
    format!("{:.2} {}", size, prefixes[i as usize])
}
