use iced::widget::{button, container, row, text, Container};
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
    pub fn update(&mut self, _msg: Message) {}

    pub fn view(&self) -> Container<'_, Message> {
        let button_content = |label| {
            text(label)
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .size(18)
        };

        let header_button = |label, msg| {
            button(button_content(label))
                .on_press(msg)
                .width(Length::Fill)
                .padding(15)
                .style(crate::gui::styles::header::button)
        };

        let home_button = header_button("Home", Message::HomePage);
        let library_button = header_button("Library", Message::LibraryPage);
        let search_button = header_button("Search", Message::SearchPage);
        let settings_button = header_button("Settings", Message::SettingsPage);

        container(
            row![home_button, library_button, search_button, settings_button,]
                .width(Length::Fill)
                .spacing(10),
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
