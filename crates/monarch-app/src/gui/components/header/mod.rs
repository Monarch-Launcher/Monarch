use iced::widget::{button, container, image, mouse_area, row, rule, svg, text, Space};
use iced::{alignment, Element, Length};

#[derive(Clone, Debug)]
pub enum Message {
    HomePage,
    LibraryPage,
    SearchPage,
    SettingsPage,
    DownloadPage,
    MinimizeWindow,
    MaximizeWindow,
    CloseWindow,
    DragWindow,
}

#[derive(Default)]
pub struct Header {}

impl Header {
    pub fn _update(&mut self, _msg: Message) {}

    pub fn view(
        &self,
        active_tab: crate::gui::pages::PageTab,
        download_speed: Option<f64>,
        speed_in_bits: bool,
        pending_downloads: usize,
    ) -> Element<'_, Message> {
        let logo = image(crate::gui::resources::LOGO.clone()).height(Length::Fixed(32.0));

        let button_content = |label| {
            text(label)
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .size(20)
        };

        let header_button = |label, msg, is_active| {
            let style = if is_active {
                crate::gui::styles::header::active_button
            } else {
                crate::gui::styles::header::button
            };

            button(button_content(label))
                .on_press(msg)
                .width(Length::Shrink)
                .padding(6)
                .style(style)
        };

        use crate::gui::pages::PageTab;

        let home_button = header_button("Home", Message::HomePage, active_tab == PageTab::Home);
        let library_button = header_button(
            "Library",
            Message::LibraryPage,
            active_tab == PageTab::Library || active_tab == PageTab::GameDetails,
        );
        let search_button =
            header_button("Search", Message::SearchPage, active_tab == PageTab::Search);
        let settings_button = header_button(
            "Settings",
            Message::SettingsPage,
            active_tab == PageTab::Settings,
        );

        let count_badge = |count: usize| -> Element<'_, Message> {
            container(
                text(count.to_string())
                    .size(11)
                    .color(iced::Color::WHITE)
                    .font(crate::gui::styles::fonts::MEDIUM),
            )
            .padding(iced::Padding::from([2.0, 5.0]))
            .style(|_theme: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Color::from_rgb8(255, 127, 0).into()),
                border: iced::border::Border {
                    radius: crate::gui::styles::radius::SUBTLE.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
        };

        let speed_widget = match download_speed {
            Some(speed) => {
                let mut content = row![
                    svg(crate::gui::resources::DOWNLOAD.clone())
                        .width(Length::Fixed(16.0))
                        .height(Length::Fixed(16.0))
                        .style(|_theme: &iced::Theme, _status| iced::widget::svg::Style {
                            color: Some(iced::Color::from_rgb8(255, 127, 0)),
                        }),
                    text(crate::gui::components::common::format_speed(
                        speed,
                        speed_in_bits,
                    ))
                    .size(14)
                    .color(iced::Color::from_rgb8(255, 127, 0))
                    .font(crate::gui::styles::fonts::MEDIUM),
                ]
                .spacing(8)
                .align_y(alignment::Vertical::Center);

                if pending_downloads > 0 {
                    content = content.push(count_badge(pending_downloads));
                }

                button(content)
                    .on_press(Message::DownloadPage)
                    .padding(iced::Padding::from([8, 12]))
                    .style(crate::gui::styles::download::speed_widget)
            }
            None => {
                let content: Element<'_, Message> = if pending_downloads > 0 {
                    row![
                        svg(crate::gui::resources::DOWNLOAD.clone())
                            .width(Length::Fixed(16.0))
                            .height(Length::Fixed(16.0))
                            .style(|_theme: &iced::Theme, _status| iced::widget::svg::Style {
                                color: Some(iced::Color::from_rgb8(150, 150, 150)),
                            }),
                        count_badge(pending_downloads)
                    ]
                    .spacing(6)
                    .align_y(alignment::Vertical::Center)
                    .into()
                } else {
                    svg(crate::gui::resources::DOWNLOAD.clone())
                        .width(Length::Fixed(16.0))
                        .height(Length::Fixed(16.0))
                        .style(|_theme: &iced::Theme, _status| iced::widget::svg::Style {
                            color: Some(iced::Color::from_rgb8(150, 150, 150)),
                        })
                        .into()
                };

                button(content)
                    .on_press(Message::DownloadPage)
                    .padding(iced::Padding::from(8))
                    .style(crate::gui::styles::download::speed_widget_idle)
            }
        };

        let nav = row![home_button, library_button, search_button, settings_button]
            .spacing(10)
            .align_y(alignment::Vertical::Center);

        fn window_button<F>(
            icon: &svg::Handle,
            msg: Message,
            style: F,
            svg_color: iced::Color,
            svg_hover_color: iced::Color,
        ) -> Element<'_, Message>
        where
            F: Fn(&iced::Theme, button::Status) -> button::Style + 'static,
        {
            button(
                svg(icon.clone())
                    .width(Length::Fixed(12.0))
                    .height(Length::Fixed(12.0))
                    .style(move |_theme, status| iced::widget::svg::Style {
                        color: Some(match status {
                            iced::widget::svg::Status::Idle => svg_color,
                            iced::widget::svg::Status::Hovered => svg_hover_color,
                        }),
                    }),
            )
            .on_press(msg)
            .width(Length::Fixed(36.0))
            .height(Length::Fixed(28.0))
            .padding(iced::Padding::from([8.0, 12.0]))
            .style(style)
            .into()
        }

        let window_controls = row![
            speed_widget,
            window_button(
                &crate::gui::resources::WINDOW_MINIMIZE,
                Message::MinimizeWindow,
                crate::gui::styles::header::window_control,
                iced::Color::from_rgba8(255, 127, 0, 0.7),
                iced::Color::from_rgb8(255, 127, 0),
            ),
            window_button(
                &crate::gui::resources::WINDOW_MAXIMIZE,
                Message::MaximizeWindow,
                crate::gui::styles::header::window_control,
                iced::Color::from_rgba8(255, 127, 0, 0.7),
                iced::Color::from_rgb8(255, 127, 0),
            ),
            window_button(
                &crate::gui::resources::WINDOW_CLOSE,
                Message::CloseWindow,
                crate::gui::styles::header::window_control_close,
                iced::Color::from_rgba8(255, 127, 0, 0.7),
                iced::Color::from_rgb8(255, 127, 0),
            ),
        ]
        .spacing(4)
        .align_y(alignment::Vertical::Center);

        // The whole header is a drag region: on press anywhere that isn't an
        // interactive button (buttons capture the event first), the window is
        // dragged. This fires on mouse-down, which winit requires for
        // `window::drag` to start a native drag loop.
        let content = mouse_area(
            container(
                row![
                    logo,
                    nav,
                    Space::new().width(Length::Fill).height(Length::Shrink),
                    window_controls,
                ]
                .align_y(alignment::Vertical::Top)
                .width(Length::Fill),
            )
            .padding(iced::Padding {
                top: 4.0,
                right: 0.0,
                bottom: 4.0,
                left: 10.0,
            })
            .style(crate::gui::styles::header::container),
        )
        .on_press(Message::DragWindow)
        .interaction(iced::mouse::Interaction::Pointer);

        let accent_line =
            rule::horizontal(1).style(|theme: &iced::Theme| iced::widget::rule::Style {
                color: theme.palette().primary,
                radius: 0.0.into(),
                fill_mode: iced::widget::rule::FillMode::Padded(24),
                snap: true,
            });

        iced::widget::column![content, accent_line].into()
    }
}

impl From<Message> for crate::gui::AppMessage {
    fn from(value: Message) -> Self {
        crate::gui::AppMessage::HeaderMessage(value)
    }
}
