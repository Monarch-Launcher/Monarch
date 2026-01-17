use iced::{widget::container, Element, Length::Fill};

use crate::gui::{
    components::header::{self, Header},
    pages::PageTab,
};

pub mod components;
pub mod pages;
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
        .run()
        .unwrap();
    }

    fn update_wrapper(&mut self, message: AppMessage) -> iced::Task<AppMessage> {
        self.update(message);
        iced::Task::none()
    }
}

impl App {
    fn update(&mut self, message: AppMessage) {
        match message {
            AppMessage::HeaderMessage(msg) => match msg {
                header::Message::HomePage => self.active_tab = PageTab::Home,
                header::Message::LibraryPage => self.active_tab = PageTab::Library,
                header::Message::SearchPage => self.active_tab = PageTab::Search,
                header::Message::SettingsPage => self.active_tab = PageTab::Settings,
            },
            AppMessage::Page(page_msg) => match page_msg {
                pages::Message::Home(msg) => self.home_page.update(msg),
                pages::Message::Library(msg) => self.library_page.update(msg),
                pages::Message::Search(msg) => self.search_page.update(msg),
                pages::Message::Settings(msg) => self.settings_page.update(msg),
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
            .height(Fill)
            .width(Fill);

        container(
            iced::widget::Column::new()
                .push(
                    Element::from(self.header.view(self.active_tab)).map(AppMessage::HeaderMessage),
                )
                .push(content),
        )
        .into()
    }
}
