use crate::gui::components::common::{primary_button, secondary_button};
use crate::gui::components::gamecard::container::LibraryFilter;
use crate::gui::components::modal::Modal;
use crate::gui::styles;
use crate::gui::styles::button::text as text_button;
use iced::widget::{button, checkbox, column, container, row, text, Space};
use iced::{Element, Length};

#[derive(Clone, Debug)]
pub enum Message {
    SteamToggled(bool),
    EpicToggled(bool),
    InstalledToggled(bool),
    UninstalledToggled(bool),
    Reset,
    Apply,
    Cancel,
}

#[derive(Clone, Debug)]
pub struct FilterModal {
    pub filter: LibraryFilter,
}

impl FilterModal {
    pub fn new(filter: LibraryFilter) -> Self {
        Self { filter }
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::SteamToggled(value) => self.filter.steam = value,
            Message::EpicToggled(value) => self.filter.epic = value,
            Message::InstalledToggled(value) => self.filter.installed = value,
            Message::UninstalledToggled(value) => self.filter.uninstalled = value,
            Message::Reset => self.filter = LibraryFilter::default(),
            Message::Apply | Message::Cancel => {}
        }
    }

    fn checkbox<'a>(
        label: &'a str,
        checked: bool,
        on_toggle: impl Fn(bool) -> Message + 'a,
    ) -> container::Container<'a, Message> {
        container(
            checkbox(checked)
                .label(label)
                .on_toggle(on_toggle)
                .text_size(16)
                .size(20),
        )
        .width(Length::Fill)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let content = column![
            text("Store").size(18).font(styles::fonts::BOLD),
            Self::checkbox("Steam", self.filter.steam, Message::SteamToggled),
            Self::checkbox(
                "Epic Games",
                self.filter.epic,
                Message::EpicToggled,
            ),
            Space::new().height(Length::Fixed(10.0)),
            text("Status").size(18).font(styles::fonts::BOLD),
            Self::checkbox(
                "Installed",
                self.filter.installed,
                Message::InstalledToggled,
            ),
            Self::checkbox(
                "Ready to Install",
                self.filter.uninstalled,
                Message::UninstalledToggled,
            ),
            Space::new().height(Length::Fixed(10.0)),
            button(text("Reset")).on_press(Message::Reset).style(text_button),
            Space::new().height(Length::Fixed(10.0)),
            row![
                Space::new().width(Length::Fill),
                secondary_button("Cancel", Some(Message::Cancel)),
                primary_button("Apply", Some(Message::Apply)),
            ]
            .spacing(10)
        ]
        .spacing(10);

        Modal::new("Filter Library", content)
            .on_close(Message::Cancel)
            .width(Length::Fixed(400.0))
            .view()
    }
}
