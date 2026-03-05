use iced::{
    alignment,
    widget::{column, container, image, row, scrollable, stack, text},
    Color, Element, Length, Theme,
};

use crate::{
    gui::{
        components::common::{secondary_button, store_button},
        pages::store_details::{Message, StoreDetailsPage},
    },
    monarch_games::monarchgame::MonarchGame,
};

impl StoreDetailsPage {
    pub fn view_store_details(&self) -> Element<'_, Message> {
        let game: MonarchGame = self.game.as_ref().unwrap().lock().unwrap().clone();

        // Determine background image with fallback logic
        let background_image = if self.artwork_loaded && !game.artwork_path.is_empty() {
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

        // Description section
        let description_title = text("About")
            .size(20)
            .color(Color::from_rgb8(200, 200, 200))
            .font(crate::gui::styles::fonts::SEMIBOLD);

        let description_text = text(if game.summary.is_empty() {
            "No description available.".to_string()
        } else {
            game.summary.clone()
        })
        .size(15)
        .color(Color::from_rgb8(180, 180, 180))
        .line_height(iced::widget::text::LineHeight::Relative(1.7));

        // Available Stores UI
        let stores_title = text("Available Stores")
            .size(24)
            .color(Color::WHITE)
            .font(crate::gui::styles::fonts::BOLD);

        let stores_list = column(
            game.stores
                .iter()
                .map(|store| {
                    let dl_btn = store_button(
                        &format!("Download from {}", store.name),
                        Some(Message::DownloadGame(
                            game.name.clone(),
                            store.name.clone(),
                            store.store_id.clone(),
                        )),
                    );

                    // Add external store page button if url exists
                    let ext_btn = if store.store_url.is_empty() {
                        container(iced::widget::Space::new())
                    } else {
                        container(store_button(
                            "External Page",
                            Some(Message::OpenStorePage(store.store_url.clone())),
                        ))
                    };

                    container(
                        row![
                            text(store.name.clone())
                                .size(18)
                                .color(Color::from_rgb8(220, 220, 220))
                                .width(Length::Fixed(150.0)),
                            dl_btn,
                            ext_btn,
                        ]
                        .spacing(15)
                        .align_y(alignment::Vertical::Center),
                    )
                    .width(Length::Fill)
                    .padding(15)
                    .style(|_theme: &Theme| container::Style {
                        background: Some(Color::from_rgba8(255, 255, 255, 0.05).into()),
                        border: iced::Border {
                            radius: 8.0.into(),
                            color: Color::from_rgba8(255, 255, 255, 0.1),
                            width: 1.0,
                        },
                        ..Default::default()
                    })
                    .into()
                })
                .collect::<Vec<Element<'_, Message>>>(),
        )
        .spacing(10);

        let stores_panel = column![stores_title, stores_list].spacing(15);

        let header_panel = column![title].spacing(12);

        // Right side content (info panel)
        let info_panel = column![
            description_title,
            description_text,
            container(text("")).height(Length::Fixed(20.0)), // Spacer
            stores_panel,
        ]
        .spacing(12)
        .width(Length::FillPortion(2));

        // Main content area with horizontal layout
        let main_content =
            column![cover_container, header_panel, row![info_panel].spacing(40),].spacing(40);

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

        if let Some(modal) = &self.download_modal {
            layers = layers.push(modal.view().map(Message::DownloadModalMessage));
        }

        layers.into()
    }
}
