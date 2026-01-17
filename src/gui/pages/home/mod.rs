use iced::widget::{container, text};
use iced::Element;
use iced::Length::Fill;

#[derive(Clone, Debug)]
pub enum Message {}

#[derive(Default)]
pub struct HomePage {}

impl HomePage {
    pub fn update(&mut self, _msg: Message) {}

    pub fn view(&self) -> Element<'_, Message> {
        container(
            text("Welcome to Monarch")
                .size(50)
                .width(Fill)
                .height(Fill)
                .center(),
        )
        .into()
    }
}
