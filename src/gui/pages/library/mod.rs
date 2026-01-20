use iced::widget::{column, container, text};
use iced::Length::{self, Fill};
use iced::{alignment, Color, Element};

use crate::gui::components::common::primary_button;
use crate::gui::components::gamecard;
use crate::gui::components::gamecard::game_browser::GameBrowser;

#[derive(Clone, Debug)]
pub enum Message {
    RefreshLibrary,
    GameCard(gamecard::GameCardMessage),
}

#[derive(Default)]
pub struct LibraryPage {
    browser: GameBrowser,
    is_refreshing: bool,
}

impl LibraryPage {
    pub fn update(&mut self, _msg: Message) -> iced::Task<Message> {
        iced::Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let refresh_btn = primary_button("Refresh Library", Some(Message::RefreshLibrary));

        let games_content: Element<'_, Message> = if self.is_refreshing {
            container(
                column![text("Looking for games...")
                    .size(32)
                    .style(|_theme: &iced::Theme| text::Style {
                        color: Some(Color::from_rgb8(255, 127, 0)),
                    })]
                .spacing(20)
                .align_x(alignment::Horizontal::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(100)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .into()
        } else {
            self.browser.view().map(Message::GameCard)
        };

        container(
            column![
                container(refresh_btn)
                    .width(Fill)
                    .height(Fill)
                    .align_x(alignment::Horizontal::Left)
                    .align_y(alignment::Vertical::Top),
                games_content
            ]
            .align_x(alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
