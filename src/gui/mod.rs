use iced::{widget::container, Element, Length::Fill};

use crate::gui::{
    components::header::{self, Header},
    pages::PageTab,
};

pub mod components;
pub mod pages;
pub mod resources;
pub mod styles;

#[derive(Clone, Debug)]
enum AppMessage {
    HeaderMessage(header::Message),
    Page(pages::Message),
}

#[derive(Default)]
pub struct App {
    header: Header,
    active_tab: PageTab,
    home_page: pages::home::HomePage,
    library_page: pages::library::LibraryPage,
    search_page: pages::search::SearchPage,
    settings_page: pages::settings::SettingsPage,
}

impl App {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn run(&self) {
        iced::application(
            || (App::new(), iced::Task::none()),
            App::update_wrapper,
            App::view,
        )
        .title(|_: &App| "Monarch".to_string())
        .theme(|_: &App| styles::theme::monarch())
        .subscription(App::subscription)
        .run()
        .unwrap();
    }

    fn subscription(&self) -> iced::Subscription<AppMessage> {
        match self.active_tab {
            pages::PageTab::Search => iced::time::every(std::time::Duration::from_millis(16))
                .map(|_| AppMessage::Page(pages::Message::Search(pages::search::Message::Tick))),
            pages::PageTab::Library => iced::time::every(std::time::Duration::from_millis(16))
                .map(|_| AppMessage::Page(pages::Message::Library(pages::library::Message::Tick))),
            _ => iced::Subscription::none(),
        }
    }

    fn update_wrapper(&mut self, message: AppMessage) -> iced::Task<AppMessage> {
        self.update(message)
    }
}

impl App {
    fn update(&mut self, message: AppMessage) -> iced::Task<AppMessage> {
        match message {
            AppMessage::HeaderMessage(msg) => {
                match msg {
                    header::Message::HomePage => self.active_tab = PageTab::Home,
                    header::Message::LibraryPage => self.active_tab = PageTab::Library,
                    header::Message::SearchPage => self.active_tab = PageTab::Search,
                    header::Message::SettingsPage => self.active_tab = PageTab::Settings,
                }
                iced::Task::none()
            }
            AppMessage::Page(page_msg) => match page_msg {
                pages::Message::Home(msg) => self
                    .home_page
                    .update(msg)
                    .map(|m| AppMessage::Page(pages::Message::Home(m))),
                pages::Message::Library(msg) => self
                    .library_page
                    .update(msg)
                    .map(|m| AppMessage::Page(pages::Message::Library(m))),
                pages::Message::Search(msg) => self
                    .search_page
                    .update(msg)
                    .map(|m| AppMessage::Page(pages::Message::Search(m))),
                pages::Message::Settings(msg) => self
                    .settings_page
                    .update(msg)
                    .map(|m| AppMessage::Page(pages::Message::Settings(m))),
            },
        }
    }

    fn view(&self) -> Element<'_, AppMessage> {
        let page_content = match self.active_tab {
            PageTab::Home => self.home_page.view().map(pages::Message::Home),
            PageTab::Library => self.library_page.view().map(pages::Message::Library),
            PageTab::Search => self.search_page.view().map(pages::Message::Search),
            PageTab::Settings => self.settings_page.view().map(pages::Message::Settings),
        };

        let content = container(page_content.map(AppMessage::Page))
            .width(Fill)
            .height(Fill);

        container(
            iced::widget::Column::new()
                .push(
                    Element::from(self.header.view(self.active_tab)).map(AppMessage::HeaderMessage),
                )
                .push(content)
                .width(Fill)
                .height(Fill),
        )
        .width(Fill)
        .height(Fill)
        .into()
    }
}
