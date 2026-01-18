use iced::widget::{button, container, image, row, text, Container};
use iced::{alignment, Length};

#[derive(Clone, Debug)]
pub enum Message {
    HomePage,
    LibraryPage,
    SearchPage,
    SettingsPage,
}

#[derive(Default)]
pub struct Header {}

impl Header {
    pub fn _update(&mut self, _msg: Message) {}

    pub fn view(&self, active_tab: crate::gui::pages::PageTab) -> Container<'_, Message> {
        let logo = image("icons/Square71x71Logo.png");

        let button_content = |label| {
            text(label)
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .size(24)
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
                .padding(15)
                .style(style)
        };

        use crate::gui::pages::PageTab;

        let home_button = header_button("Home", Message::HomePage, active_tab == PageTab::Home);
        let library_button = header_button(
            "Library",
            Message::LibraryPage,
            active_tab == PageTab::Library,
        );
        let search_button =
            header_button("Search", Message::SearchPage, active_tab == PageTab::Search);
        let settings_button = header_button(
            "Settings",
            Message::SettingsPage,
            active_tab == PageTab::Settings,
        );

        container(
            row![
                logo,
                row![home_button, library_button, search_button, settings_button,]
                    .width(Length::Fill)
                    .spacing(10),
            ]
            .align_y(alignment::Vertical::Center),
        )
        .padding(10)
        .style(crate::gui::styles::header::container)
    }
}

impl From<Message> for crate::gui::AppMessage {
    fn from(value: Message) -> Self {
        crate::gui::AppMessage::HeaderMessage(value)
    }
}
