use iced::widget::{button, column, container, text, text_input};
use iced::{alignment, Element, Length};

#[derive(Clone, Debug)]
pub enum Message {
    SearchChanged(String),
    FiltersPressed,
}

#[derive(Default)]
pub struct SearchPage {
    search_value: String,
}

impl SearchPage {
    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::SearchChanged(value) => {
                self.search_value = value;
            }
            Message::FiltersPressed => {
                // TODO: Show filters
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let search_input = text_input("Search for games ...", &self.search_value)
            .on_input(Message::SearchChanged)
            .style(crate::gui::styles::text_input::search)
            .padding(15)
            .width(Length::Fixed(600.0))
            .size(16);

        // Customize filter button to look like the image (dark with orange icon/text probably)
        // For now using standard text button
        let filters_button = button(text("Filters").align_x(alignment::Horizontal::Center))
            .on_press(Message::FiltersPressed)
            .padding(10)
            .style(crate::gui::styles::header::button);

        container(
            column![search_input, filters_button]
                .spacing(15)
                .align_x(alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .into()
    }
}
