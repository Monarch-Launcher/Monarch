use iced::widget::{button, container, row, Container};

#[derive(Clone)]
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
        let home_button = button("Home").on_press(Message::HomePage);
        let library_button = button("Library").on_press(Message::HomePage);
        let search_button = button("Search").on_press(Message::HomePage);
        let settings_button = button("Settings").on_press(Message::HomePage);

        container(row![
            home_button,
            library_button,
            search_button,
            settings_button,
        ])
    }
}

impl From<Message> for crate::gui::AppMessage {
    fn from(value: Message) -> Self {
        crate::gui::AppMessage::HeaderMessage(value)
    }
}
