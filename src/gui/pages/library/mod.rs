use iced::widget::{container, text};
use iced::Element;
use iced::Length::Fill;

#[derive(Clone, Debug)]
pub enum Message {}

#[derive(Default)]
pub struct LibraryPage {}

impl LibraryPage {
    pub fn update(&mut self, _msg: Message) {}

    pub fn view(&self) -> Element<'_, Message> {
        container(
            text("Library is WIP")
                .size(50)
                .width(Fill)
                .height(Fill)
                .center(),
        )
        .into()
    }
}
