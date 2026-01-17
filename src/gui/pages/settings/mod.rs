use iced::widget::{container, text};
use iced::Element;
use iced::Length::Fill;

#[derive(Clone, Debug)]
pub enum Message {}

#[derive(Default)]
pub struct SettingsPage {}

impl SettingsPage {
    pub fn update(&mut self, _msg: Message) {}

    pub fn view(&self) -> Element<'_, Message> {
        container(
            text("Settings is WIP")
                .size(50)
                .width(Fill)
                .height(Fill)
                .center(),
        )
        .into()
    }
}
