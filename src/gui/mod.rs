use iced::{widget::container, window::Id, Element, Length::Fill, Subscription};
use iced_term;
use tracing::info;

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
    OpenGameDetails(crate::monarch_games::monarchgame::MonarchGame),
    CloseGameDetails,
    OpenTerminal(Id),
    CloseTerminal(Id),
    Terminal(iced_term::Event),
}

#[derive(Default)]
pub struct App {
    header: Header,
    active_tab: PageTab,
    previous_tab: PageTab, // Track previous tab for back navigation
    home_page: pages::home::HomePage,
    library_page: pages::library::LibraryPage,
    search_page: pages::search::SearchPage,
    settings_page: pages::settings::SettingsPage,
    game_details_page: pages::game_details::GameDetailsPage,

    active_terminals: Vec<Id>,
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
        // Terminal subscriptions
        let terminal_subscription: Subscription<AppMessage> = if !self.active_terminals.is_empty() {
            return iced::event::listen_with(|event, status, id| match status {
                iced::event::Status::Ignored => match event {
                    iced::Event::Window(iced::window::Event::CloseRequested) => {
                        Some(AppMessage::CloseTerminal(id))
                    }
                    _ => Some(AppMessage::CloseTerminal(id)),
                },
                iced::event::Status::Captured => None,
            });
        } else {
            iced::Subscription::none()
        };

        // Page/animation subscriptions
        let page_subscription: Subscription<AppMessage> = match self.active_tab {
            pages::PageTab::Search => iced::time::every(std::time::Duration::from_millis(16))
                .map(|_| AppMessage::Page(pages::Message::Search(pages::search::Message::Tick))),
            pages::PageTab::Library => iced::time::every(std::time::Duration::from_millis(16))
                .map(|_| AppMessage::Page(pages::Message::Library(pages::library::Message::Tick))),
            _ => iced::Subscription::none(),
        };

        let subscriptions: [Subscription<AppMessage>; 2] =
            [terminal_subscription, page_subscription];

        iced::Subscription::batch(subscriptions)
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
                    header::Message::SettingsPage => {
                        self.active_tab = PageTab::Settings;
                        self.settings_page.refresh();
                    }
                }
                iced::Task::none()
            }
            AppMessage::Page(page_msg) => match page_msg {
                pages::Message::Home(msg) => self
                    .home_page
                    .update(msg)
                    .map(|m| AppMessage::Page(pages::Message::Home(m))),
                pages::Message::Library(msg) => {
                    // Check if it's OpenGameDetails
                    if let pages::library::Message::OpenGameDetails(game) = &msg {
                        return iced::Task::done(AppMessage::OpenGameDetails(game.clone()));
                    }
                    self.library_page
                        .update(msg)
                        .map(|m| AppMessage::Page(pages::Message::Library(m)))
                }
                pages::Message::Search(msg) => self
                    .search_page
                    .update(msg)
                    .map(|m| AppMessage::Page(pages::Message::Search(m))),
                pages::Message::Settings(msg) => self
                    .settings_page
                    .update(msg)
                    .map(|m| AppMessage::Page(pages::Message::Settings(m))),
                pages::Message::GameDetails(msg) => {
                    match msg {
                        pages::game_details::Message::BackPressed => {
                            // Navigate back to previous page
                            self.active_tab = self.previous_tab;
                            iced::Task::none()
                        }
                        pages::game_details::Message::LaunchGame => self
                            .game_details_page
                            .update(msg)
                            .map(|m| AppMessage::Page(pages::Message::GameDetails(m))),
                        pages::game_details::Message::OpenTerminal(id) => {
                            self.update(AppMessage::OpenTerminal(id))
                        }
                        pages::game_details::Message::CloseTerminal(id) => {
                            self.update(AppMessage::CloseTerminal(id))
                        }
                    }
                }
            },
            AppMessage::OpenGameDetails(game) => {
                self.previous_tab = self.active_tab;
                self.game_details_page.set_game(game);
                self.active_tab = PageTab::GameDetails;
                iced::Task::none()
            }
            AppMessage::CloseGameDetails => {
                self.active_tab = self.previous_tab;
                iced::Task::none()
            }
            AppMessage::OpenTerminal(id) => {
                self.active_terminals.push(id);
                info!("Open | Terms: {:?}", self.active_terminals);
                iced::Task::none()
            }
            AppMessage::CloseTerminal(id) => {
                self.active_terminals = self
                    .active_terminals
                    .iter()
                    .cloned()
                    .filter(|&t| t != id)
                    .collect();
                info!("Close | Terms: {:?}", self.active_terminals);
                iced::Task::none()
            }
            AppMessage::Terminal(event) => iced::Task::none(),
        }
    }

    fn view(&self) -> Element<'_, AppMessage> {
        let page_content = match self.active_tab {
            PageTab::Home => self.home_page.view().map(pages::Message::Home),
            PageTab::Library => self.library_page.view().map(pages::Message::Library),
            PageTab::Search => self.search_page.view().map(pages::Message::Search),
            PageTab::Settings => self.settings_page.view().map(pages::Message::Settings),
            PageTab::GameDetails => self
                .game_details_page
                .view()
                .map(pages::Message::GameDetails),
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
