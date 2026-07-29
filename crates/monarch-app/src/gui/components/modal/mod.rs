use iced::widget::{button, column, container, row, text, Space};
use iced::{alignment, Element, Length, Theme};

use crate::gui::styles;

pub mod download_modal;

pub struct Modal<'a, Message> {
    title: String,
    content: Element<'a, Message, Theme>,
    on_close: Option<Message>,
    width: Option<Length>,
}

impl<'a, Message> Modal<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(title: impl Into<String>, content: impl Into<Element<'a, Message, Theme>>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            on_close: None,
            width: None,
        }
    }

    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = Some(width);
        self
    }

    pub fn view(self) -> Element<'a, Message, Theme> {
        let title = text(self.title)
            .size(24)
            .font(crate::gui::styles::fonts::BOLD);

        let mut header = row![title].spacing(20).align_y(alignment::Vertical::Center);

        if let Some(on_close) = self.on_close.clone() {
            header = header.push(Space::new().width(Length::Fill)).push(
                button(text("✕").size(20))
                    .on_press(on_close)
                    .style(styles::button::text)
                    .padding(5),
            );
        }

        let modal_content = container(
            column![header, container(self.content).width(Length::Fill),]
                .spacing(20)
                .padding(20),
        )
        .width(self.width.unwrap_or(Length::Fixed(500.0)))
        .style(styles::modal::content);

        container(modal_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(styles::modal::overlay)
            .into()
    }
}
