use std::collections::HashMap;

use futures::channel::mpsc::Sender;
use iced::{
    widget::container,
    window::{self, Id},
    Element,
    Length::Fill,
    Subscription,
};
use iced_term;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use tracing::info;

use crate::gui::{
    components::header::{self, Header},
    components::terminal::TermInstance,
    pages::PageTab,
};

pub mod components;
pub mod pages;
pub mod resources;
pub mod styles;

#[derive(Clone, Debug)]
pub enum AppMessage {
    HeaderMessage(header::Message),
    Page(pages::Message),
    OpenGameDetails(crate::monarch_games::monarchgame::MonarchGame),
    CloseGameDetails,
    OpenTerminal(Id),
    CloseTerminal(Id),
    CloseWindow(Id),
    Terminal(iced_term::Event),
    OpenTerminalRaw(String, HashMap<String, String>),
}

pub static GUI_SENDER: Lazy<Mutex<Option<futures::channel::mpsc::UnboundedSender<AppMessage>>>> =
    Lazy::new(|| Mutex::new(None));

static EXTERNAL_RECEIVER: Mutex<Option<futures::channel::mpsc::UnboundedReceiver<AppMessage>>> =
    Mutex::new(None);

pub struct App {
    app_id: Id,

    header: Header,
    active_tab: PageTab,
    previous_tab: PageTab, // Track previous tab for back navigation
    home_page: pages::home::HomePage,
    library_page: pages::library::LibraryPage,
    search_page: pages::search::SearchPage,
    settings_page: pages::settings::SettingsPage,
    game_details_page: pages::game_details::GameDetailsPage,

    active_terminals: HashMap<Id, TermInstance>,
}

impl App {
    fn new(id: Id) -> Self {
        Self {
            app_id: id,
            ..Default::default()
        }
    }

    pub fn run() {
        iced::daemon(
            || {
                let (sender, receiver) = futures::channel::mpsc::unbounded();
                *GUI_SENDER.lock().unwrap() = Some(sender);
                *EXTERNAL_RECEIVER.lock().unwrap() = Some(receiver);

                let (id, task) = window::open(window::Settings::default());
                (
                    App::new(id),
                    task.map(|_| AppMessage::HeaderMessage(header::Message::HomePage)),
                )
            },
            App::update,
            App::view,
        )
        .title("Monarch")
        .theme(App::theme)
        .subscription(App::subscription)
        .run()
        .unwrap();
    }

    fn theme(&self, _window_id: Id) -> iced::Theme {
        styles::theme::monarch()
    }

    fn subscription(&self) -> iced::Subscription<AppMessage> {
        // Terminal subscriptions
        let mut subscriptions: Vec<Subscription<AppMessage>> = Vec::new();

        if !self.active_terminals.is_empty() {
            subscriptions.push(Subscription::batch(
                self.active_terminals
                    .values()
                    .map(|term| term.subscription().map(AppMessage::Terminal)),
            ));
        }

        let window_subscription = iced::event::listen_with(|event, status, id| match status {
            iced::event::Status::Ignored => match event {
                iced::Event::Window(iced::window::Event::Closed) => {
                    Some(AppMessage::CloseWindow(id))
                }
                _ => None,
            },
            iced::event::Status::Captured => None,
        });
        subscriptions.push(window_subscription);

        // Page/animation subscriptions
        let page_subscription: Subscription<AppMessage> = match self.active_tab {
            pages::PageTab::Search => iced::time::every(std::time::Duration::from_millis(16))
                .map(|_| AppMessage::Page(pages::Message::Search(pages::search::Message::Tick))),
            pages::PageTab::Library => iced::time::every(std::time::Duration::from_millis(16))
                .map(|_| AppMessage::Page(pages::Message::Library(pages::library::Message::Tick))),
            _ => iced::Subscription::none(),
        };
        subscriptions.push(page_subscription);
        subscriptions.push(Self::external_subscription());

        iced::Subscription::batch(subscriptions)
    }

    fn external_subscription() -> Subscription<AppMessage> {
        Subscription::run_with((), external_subscription_stream)
    }

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
                            .update(pages::game_details::Message::LaunchGame)
                            .map(|m| AppMessage::Page(pages::Message::GameDetails(m))),
                        pages::game_details::Message::OpenTerminal(_id) => {
                            // Already handled in LaunchGame? No, Task returns Id.
                            iced::Task::none()
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
            AppMessage::OpenTerminal(_id) => {
                // ID already inserted in LaunchGame
                iced::Task::none()
            }
            AppMessage::CloseTerminal(id) => {
                self.active_terminals.remove(&id);
                info!("Close | Terms keys: {:?}", self.active_terminals.keys());
                iced::window::close(id)
            }
            AppMessage::CloseWindow(id) => {
                if id == self.app_id {
                    info!("Monarch close requested. Cleaning up...");
                    return iced::exit();
                }
                iced::window::close(id)
            }
            AppMessage::Terminal(event) => {
                // Broadcast event to all terminals or find which one?
                // iced_term 0.7.0 subscription usually binds to a specific terminal if mapped correctly?
                // Wait, our TermInstance::subscription calls term.subscription().
                // We should probably route the event to the correct terminal if possible.
                // But AppMessage::Terminal(event) loses the ID context if we don't wrap it.
                // However, the View is what processes input.
                // The Subscription is for PTY output.
                // If we iterate active_terminals, we should probably update them?

                iced::Task::batch(
                    self.active_terminals
                        .values_mut()
                        .map(|term| term.update(event.clone())),
                )
            }
            AppMessage::OpenTerminalRaw(command, env) => {
                let settings = window::Settings {
                    decorations: false,
                    ..Default::default()
                };
                let (id, task) = window::open(settings);

                let term = TermInstance::new(id, command, env);
                self.active_terminals.insert(id, term);
                task.map(AppMessage::OpenTerminal)
            }
        }
    }

    fn view(&self, window_id: Id) -> Element<'_, AppMessage> {
        if let Some(term) = self.active_terminals.get(&window_id) {
            return term.view().map(AppMessage::Terminal);
        }

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

fn external_subscription_stream(_: &()) -> iced::futures::stream::BoxStream<'static, AppMessage> {
    use iced::futures::{SinkExt, StreamExt};
    iced::stream::channel(100, |mut output: Sender<AppMessage>| async move {
        let mut rx = EXTERNAL_RECEIVER.lock().unwrap().take();
        if let Some(mut rx) = rx {
            while let Some(msg) = rx.next().await {
                let _ = output.send(msg).await;
            }
        }
        // Keep the stream alive
        std::future::pending::<()>().await;
    })
    .boxed()
}

impl Default for App {
    fn default() -> Self {
        Self {
            app_id: Id::unique(),
            header: Header::default(),
            active_tab: PageTab::Home,
            previous_tab: PageTab::Home,
            home_page: pages::home::HomePage::default(),
            library_page: pages::library::LibraryPage::default(),
            search_page: pages::search::SearchPage::default(),
            settings_page: pages::settings::SettingsPage::default(),
            game_details_page: pages::game_details::GameDetailsPage::default(),
            active_terminals: HashMap::new(),
        }
    }
}
