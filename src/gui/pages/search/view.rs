use iced::widget::text_input;
use iced::widget::{button, column, container, row, text};
use iced::Element;
use iced::{alignment, Length};

use crate::gui::pages::search::{Message, SearchPage};

impl SearchPage {
    pub fn content_view(&self) -> Element<'_, Message> {
        let search_input = text_input("Search for games ...", &self.search_value)
            .on_input(Message::SearchChanged)
            .on_submit(Message::PerformSearch)
            .style(crate::gui::styles::text_input::search)
            .padding(15)
            .width(Length::Fixed(600.0))
            .size(20);

        let filters_button = button(text("Filters").align_x(alignment::Horizontal::Center))
            .on_press(Message::FiltersPressed)
            .padding(10)
            .style(crate::gui::styles::header::button);

        let search_bar = row![search_input, filters_button]
            .spacing(15)
            .align_y(alignment::Vertical::Center);

        let games_content: Element<'_, Message> = if self.is_searching {
            let dots = ".".repeat(self.dot_count as usize);
            container(
                column![text(format!("Searching for games{dots}"))
                    .size(32)
                    .font(crate::gui::styles::fonts::REGULAR)]
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

        let content = container(
            column![
                container(search_bar)
                    .padding(30)
                    .width(Length::Fill)
                    .align_x(alignment::Horizontal::Center),
                games_content,
            ]
            .align_x(alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill);

        content.into()
    }
}
