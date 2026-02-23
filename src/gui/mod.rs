use std::collections::HashMap;

use futures::channel::mpsc::Sender;
use iced::{
    widget::{container, text},
    window::{self, Id},
    Element,
    Length::Fill,
    Subscription,
};
use iced_term;
use std::sync::{Arc, LazyLock, Mutex};
use tracing::info;

use crate::gui::{
    components::{
        header::{self, Header},
        terminal::TermInstance,
    },
    pages::{library, PageTab},
};

pub mod components;
pub mod pages;
pub mod resources;
pub mod styles;

#[derive(Clone, Debug)]
pub enum ModalState {
    Error(String),
    Confirm(String, Box<AppMessage>),
}

#[derive(Clone, Debug)]
pub enum AppMessage {
    HeaderMessage(header::Message),
    Page(pages::Message),
    OpenGameDetails(crate::monarch_games::monarchgame::MonarchGame),
    OpenStoreDetails(crate::monarch_games::monarchgame::MonarchGame),
    OpenTerminal(Id),
    CloseTerminal(Id),
    CloseWindow(Id),
    Terminal(iced_term::Event),
    OpenTerminalRaw(String, HashMap<String, String>),
    ShowModal(ModalState),
    CloseModal,
    OpenLogs,
}

pub static GUI_SENDER: LazyLock<
    Mutex<Option<futures::channel::mpsc::UnboundedSender<AppMessage>>>,
> = LazyLock::new(|| Mutex::new(None));

/// Display an error modal with the given message.
pub fn show_error(message: impl Into<String>) {
    if let Some(sender) = GUI_SENDER.lock().unwrap().as_mut() {
        let _ = sender.unbounded_send(AppMessage::ShowModal(ModalState::Error(message.into())));
    }
}

/// Display a confirmation modal with the given message and action.
pub fn show_confirm(message: impl Into<String>, on_confirm: AppMessage) {
    if let Some(sender) = GUI_SENDER.lock().unwrap().as_mut() {
        let _ = sender.unbounded_send(AppMessage::ShowModal(ModalState::Confirm(
            message.into(),
            Box::new(on_confirm),
        )));
    }
}

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
    store_details_page: pages::store_details::StoreDetailsPage,

    active_terminals: HashMap<Id, TermInstance>,
    active_modal: Option<ModalState>,
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
                    header::Message::LibraryPage => {
                        self.active_tab = PageTab::Library;
                        return self
                            .library_page
                            .update(library::Message::UpdateGameProperties)
                            .map(|m| AppMessage::Page(pages::Message::Library(m)));
                    }
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
                pages::Message::Search(msg) => {
                    if let pages::search::Message::OpenStoreDetails(game) = &msg {
                        return iced::Task::done(AppMessage::OpenStoreDetails(game.clone()));
                    }
                    self.search_page
                        .update(msg)
                        .map(|m| AppMessage::Page(pages::Message::Search(m)))
                }
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
                        pages::game_details::Message::OpenProperties => self
                            .game_details_page
                            .update(pages::game_details::Message::OpenProperties)
                            .map(|m| AppMessage::Page(pages::Message::GameDetails(m))),
                        pages::game_details::Message::Properties(p_msg) => self
                            .game_details_page
                            .update(pages::game_details::Message::Properties(p_msg))
                            .map(|m| AppMessage::Page(pages::Message::GameDetails(m))),
                        pages::game_details::Message::Nop(_) => iced::Task::none(),
                    }
                }
                pages::Message::StoreDetails(msg) => match msg {
                    pages::store_details::Message::BackPressed => {
                        self.active_tab = self.previous_tab;
                        iced::Task::none()
                    }
                    _ => self
                        .store_details_page
                        .update(msg)
                        .map(|m| AppMessage::Page(pages::Message::StoreDetails(m))),
                },
            },
            AppMessage::OpenGameDetails(game) => {
                self.previous_tab = self.active_tab;
                self.game_details_page.set_game(Arc::new(Mutex::new(game)));
                self.active_tab = PageTab::GameDetails;
                iced::Task::none()
            }
            AppMessage::OpenStoreDetails(game) => {
                self.previous_tab = self.active_tab;
                self.store_details_page
                    .set_game(Arc::new(Mutex::new(game.clone())));
                self.active_tab = PageTab::StoreDetails;

                iced::Task::perform(
                    async move {
                        let artwork_path = game.artwork_path.clone();
                        if !artwork_path.is_empty() && std::path::Path::new(&artwork_path).exists()
                        {
                            return ();
                        }
                        let _ = crate::monarch_games::commands::download_artwork(&game).await;
                    },
                    |_| {
                        AppMessage::Page(pages::Message::StoreDetails(
                            pages::store_details::Message::ArtworkDownloaded,
                        ))
                    },
                )
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
            AppMessage::Terminal(event) => iced::Task::batch(
                self.active_terminals
                    .values_mut()
                    .map(|term| term.update(event.clone())),
            ),
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
            AppMessage::ShowModal(state) => {
                self.active_modal = Some(state);
                iced::Task::none()
            }
            AppMessage::CloseModal => {
                self.active_modal = None;
                iced::Task::none()
            }
            AppMessage::OpenLogs => {
                let _ = crate::monarch_utils::commands::open_logs();
                iced::Task::none()
            }
        }
    }

    fn view(&self, window_id: Id) -> Element<'_, AppMessage> {
        if let Some(term) = self.active_terminals.get(&window_id) {
            return term.view().map(AppMessage::Terminal);
        }

        // If it's not the main window and not a terminal, it's a webview or other secondary window.
        // Render empty content to avoid duplicating the main app.
        if window_id != self.app_id {
            return iced::widget::Space::new().into();
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
            PageTab::StoreDetails => self
                .store_details_page
                .view()
                .map(pages::Message::StoreDetails),
        };

        let main_content = container(
            iced::widget::Column::new()
                .push(
                    Element::from(self.header.view(self.active_tab)).map(AppMessage::HeaderMessage),
                )
                .push(page_content.map(AppMessage::Page))
                .width(Fill)
                .height(Fill),
        )
        .width(Fill)
        .height(Fill);

        let app_content: Element<'_, AppMessage> = main_content.into();

        if let Some(modal_state) = &self.active_modal {
            let modal_element = match modal_state {
                ModalState::Error(error) => components::modal::Modal::new(
                    "Error",
                    iced::widget::column![
                        text(error.clone()),
                        iced::widget::row![
                            components::common::secondary_button(
                                "Open Logs",
                                Some(AppMessage::OpenLogs)
                            ),
                            iced::widget::Space::new().width(Fill),
                            components::common::primary_button(
                                "Close",
                                Some(AppMessage::CloseModal)
                            ),
                        ]
                        .width(Fill)
                        .spacing(10)
                    ]
                    .spacing(20),
                )
                .on_close(AppMessage::CloseModal)
                .view(),
                ModalState::Confirm(msg, on_confirm) => components::modal::Modal::new(
                    "Confirm",
                    iced::widget::column![
                        text(msg.clone()),
                        iced::widget::row![
                            components::common::primary_button(
                                "Confirm",
                                Some((**on_confirm).clone())
                            ),
                            iced::widget::Space::new().width(Fill),
                            components::common::secondary_button(
                                "Cancel",
                                Some(AppMessage::CloseModal)
                            ),
                        ]
                        .width(Fill)
                        .spacing(10)
                    ]
                    .spacing(20),
                )
                .on_close(AppMessage::CloseModal)
                .view(),
            };

            iced::widget::stack![app_content, modal_element].into()
        } else {
            app_content
        }
    }
}

fn external_subscription_stream(_: &()) -> iced::futures::stream::BoxStream<'static, AppMessage> {
    use iced::futures::{SinkExt, StreamExt};
    iced::stream::channel(100, |mut output: Sender<AppMessage>| async move {
        let rx = EXTERNAL_RECEIVER.lock().unwrap().take();
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
            store_details_page: pages::store_details::StoreDetailsPage::default(),
            active_terminals: HashMap::new(),
            active_modal: None,
        }
    }
}
