use std::{collections::HashMap, sync::RwLock};

use futures::channel::mpsc::Sender;
use iced::{
    widget::{container, text},
    window::{self, Id},
    Element,
    Length::Fill,
    Subscription,
};
use iced_term;
use monarch_core::monarch_utils::monarch_state::MonarchState;
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
    ResizeWindow(iced::window::Direction),
    OpenGameDetails(monarch_core::monarch_games::monarchgame::MonarchGame),
    OpenStoreDetails(monarch_core::monarch_games::monarchgame::MonarchGame),
    OpenTerminal(Id),
    CloseTerminal(Id),
    CloseWindow(Id),
    Terminal(iced_term::Event),
    OpenTerminalRaw(
        String,
        HashMap<String, String>,
        Option<String>,
        std::sync::Arc<std::sync::Mutex<Option<futures::channel::oneshot::Sender<()>>>>,
    ),
    ShowModal(ModalState),
    CloseModal,
    ConfirmModalAction(Box<AppMessage>),
    OpenLogs,
    ToggleFullscreen(iced::window::Mode),
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
    is_fullscreen: bool,

    header: Header,
    active_tab: PageTab,
    previous_tab: PageTab, // Track previous tab for back navigation
    home_page: pages::home::HomePage,
    library_page: pages::library::LibraryPage,
    search_page: pages::search::SearchPage,
    settings_page: pages::settings::SettingsPage,
    game_details_page: pages::game_details::GameDetailsPage,
    store_details_page: pages::store_details::StoreDetailsPage,
    download_page: pages::download::DownloadPage,

    active_terminals: HashMap<Id, TermInstance>,
    active_modal: Option<ModalState>,

    state: Arc<RwLock<MonarchState>>, // Moving monarch state from a singleton to an app state as single source of truth
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
                *GUI_SENDER.lock().unwrap() = Some(sender.clone());
                *EXTERNAL_RECEIVER.lock().unwrap() = Some(receiver);

                monarch_core::monarch_utils::monarch_terminal::register_terminal_handler(Box::new(
                    move |command, env, workdir, done| {
                        let _ = sender.unbounded_send(AppMessage::OpenTerminalRaw(
                            command, env, workdir, done,
                        ));
                    },
                ));

                let (id, task) = window::open(window::Settings {
                    decorations: false,
                    size: iced::Size::new(1600.0, 900.0),
                    ..Default::default()
                });
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

        // A queue drag can be ended anywhere (even outside the list); end it on
        // any mouse release while a drag is in flight.
        let drag_active = self.download_page.drag.is_some();
        if drag_active && matches!(self.active_tab, pages::PageTab::Download) {
            subscriptions.push(iced::event::listen_with(
                |event, _status, _id| match event {
                    iced::Event::Mouse(iced::mouse::Event::ButtonReleased(_)) => {
                        Some(AppMessage::Page(pages::Message::Download(
                            pages::download::Message::DragEnded,
                        )))
                    }
                    _ => None,
                },
            ));
        }

        // Page/animation subscriptions
        let page_subscription: Subscription<AppMessage> = match self.active_tab {
            pages::PageTab::Search => iced::time::every(std::time::Duration::from_millis(16))
                .map(|_| AppMessage::Page(pages::Message::Search(pages::search::Message::Tick))),
            pages::PageTab::Library => iced::time::every(std::time::Duration::from_millis(16))
                .map(|_| AppMessage::Page(pages::Message::Library(pages::library::Message::Tick))),
            pages::PageTab::Home => iced::time::every(std::time::Duration::from_millis(16))
                .map(|_| AppMessage::Page(pages::Message::Home(pages::home::Message::Tick))),
            // The speed graph redraws continuously between samples so it
            // scrolls smoothly instead of jumping once per poll.
            pages::PageTab::Download => iced::time::every(std::time::Duration::from_millis(16))
                .map(|_| {
                    AppMessage::Page(pages::Message::Download(
                        pages::download::Message::AnimationFrame,
                    ))
                }),
            _ => iced::Subscription::none(),
        };
        subscriptions.push(page_subscription);

        // Poll the downloader so queue changes and completion (which upserts
        // the installed game into the library UI) are picked up even when no
        // download is in flight.
        subscriptions.push(
            iced::time::every(std::time::Duration::from_millis(250)).map(|_| {
                AppMessage::Page(pages::Message::Download(pages::download::Message::Tick))
            }),
        );
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
                    header::Message::HomePage => {
                        self.active_tab = PageTab::Home;
                        if self.home_page.is_loading {
                            return self
                                .home_page
                                .init()
                                .map(|m| AppMessage::Page(pages::Message::Home(m)));
                        }
                    }
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
                        let _ = self
                            .settings_page
                            .update(pages::settings::Message::Refresh(()));
                    }
                    header::Message::DownloadPage => {
                        self.active_tab = PageTab::Download;
                    }
                    header::Message::MinimizeWindow => {
                        return iced::window::minimize(self.app_id, true);
                    }
                    header::Message::ToggleFullscreen => {
                        return iced::window::mode(self.app_id).map(AppMessage::ToggleFullscreen);
                    }
                    header::Message::CloseWindow => {
                        return iced::window::close(self.app_id);
                    }
                    header::Message::DragWindow => {
                        return iced::window::drag(self.app_id);
                    }
                }
                iced::Task::none()
            }
            AppMessage::ToggleFullscreen(current_mode) => {
                let new_mode = match current_mode {
                    iced::window::Mode::Fullscreen => iced::window::Mode::Windowed,
                    _ => iced::window::Mode::Fullscreen,
                };
                let entering_fullscreen = new_mode == iced::window::Mode::Fullscreen;
                #[cfg(target_os = "windows")]
                crate::window::set_rounded_corners(!entering_fullscreen);
                self.is_fullscreen = entering_fullscreen;
                iced::window::set_mode(self.app_id, new_mode)
            }
            AppMessage::ResizeWindow(direction) => {
                return iced::window::drag_resize(self.app_id, direction);
            }
            AppMessage::Page(page_msg) => match page_msg {
                pages::Message::Home(msg) => {
                    if let pages::home::Message::OpenGameDetails(game) = &msg {
                        return iced::Task::done(AppMessage::OpenGameDetails(game.clone()));
                    }
                    self.home_page
                        .update(msg)
                        .map(|m| AppMessage::Page(pages::Message::Home(m)))
                }
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
                        pages::game_details::Message::GameUninstalled(game_id) => {
                            // Drop the card immediately and return to the library.
                            self.active_tab = PageTab::Library;
                            self.library_page
                                .update(pages::library::Message::GameRemoved(game_id))
                                .map(|m| AppMessage::Page(pages::Message::Library(m)))
                        }
                        pages::game_details::Message::LaunchGame => self
                            .game_details_page
                            .update(pages::game_details::Message::LaunchGame)
                            .map(|m| AppMessage::Page(pages::Message::GameDetails(m))),
                        pages::game_details::Message::DownloadGame => self
                            .game_details_page
                            .update(pages::game_details::Message::DownloadGame)
                            .map(|m| AppMessage::Page(pages::Message::GameDetails(m))),
                        pages::game_details::Message::OpenProperties => self
                            .game_details_page
                            .update(pages::game_details::Message::OpenProperties)
                            .map(|m| AppMessage::Page(pages::Message::GameDetails(m))),
                        pages::game_details::Message::OpenActions => self
                            .game_details_page
                            .update(pages::game_details::Message::OpenActions)
                            .map(|m| AppMessage::Page(pages::Message::GameDetails(m))),
                        pages::game_details::Message::Properties(p_msg) => self
                            .game_details_page
                            .update(pages::game_details::Message::Properties(p_msg))
                            .map(|m| AppMessage::Page(pages::Message::GameDetails(m))),
                        pages::game_details::Message::Actions(a_msg) => self
                            .game_details_page
                            .update(pages::game_details::Message::Actions(a_msg))
                            .map(|m| AppMessage::Page(pages::Message::GameDetails(m))),
                        pages::game_details::Message::Nop(_) => iced::Task::none(),
                        pages::game_details::Message::DownloadModalMessage(dm_msg) => self
                            .game_details_page
                            .update(pages::game_details::Message::DownloadModalMessage(dm_msg))
                            .map(|m| AppMessage::Page(pages::Message::GameDetails(m))),
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
                pages::Message::Download(msg) => {
                    if let pages::download::Message::DownloadFinished(game_id) = &msg {
                        // The backend already wrote the installed game into
                        // MONARCH_STATE; upsert that one card instead of a full
                        // store refresh.
                        let game_id = game_id.clone();
                        let _ = self.download_page.update(msg);
                        return self
                            .library_page
                            .update(pages::library::Message::GameInstalled(game_id))
                            .map(|m| AppMessage::Page(pages::Message::Library(m)));
                    }
                    self.download_page
                        .update(msg)
                        .map(|m| AppMessage::Page(pages::Message::Download(m)))
                }
            },
            AppMessage::OpenGameDetails(game) => {
                self.previous_tab = self.active_tab;
                self.game_details_page.set_game(Arc::new(Mutex::new(game)));
                self.active_tab = PageTab::GameDetails;
                iced::Task::none()
            }
            AppMessage::OpenStoreDetails(game) => {
                self.previous_tab = self.active_tab;
                let props_task = self
                    .store_details_page
                    .set_game(Arc::new(Mutex::new(game.clone())))
                    .map(|m| AppMessage::Page(pages::Message::StoreDetails(m)));

                self.active_tab = PageTab::StoreDetails;

                let artwork_task = iced::Task::perform(
                    async move {
                        let artwork_path = game.artwork_path.clone();
                        if !artwork_path.is_empty() && std::path::Path::new(&artwork_path).exists()
                        {
                            return ();
                        }
                        let _ =
                            monarch_core::monarch_games::commands::download_artwork(&game).await;
                    },
                    |_| {
                        AppMessage::Page(pages::Message::StoreDetails(
                            pages::store_details::Message::ArtworkDownloaded,
                        ))
                    },
                );

                iced::Task::batch(vec![props_task, artwork_task])
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
            AppMessage::OpenTerminalRaw(command, env, workdir, tx) => {
                let settings = window::Settings {
                    decorations: true,
                    ..Default::default()
                };
                let (id, task) = window::open(settings);

                let completion_tx = tx.lock().unwrap().take();
                if let Ok(term) = TermInstance::new(id, command, env, workdir, completion_tx) {
                    self.active_terminals.insert(id, term);
                    return task.map(AppMessage::OpenTerminal);
                }
                iced::Task::none()
            }
            AppMessage::ShowModal(state) => {
                self.active_modal = Some(state);
                iced::Task::none()
            }
            AppMessage::CloseModal => {
                self.active_modal = None;
                iced::Task::none()
            }
            AppMessage::ConfirmModalAction(inner) => {
                self.active_modal = None;
                self.update(*inner)
            }
            AppMessage::OpenLogs => {
                let _ = monarch_core::monarch_utils::commands::open_logs();
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

        let show_speed_in_bits = match monarch_core::monarch_utils::commands::get_settings() {
            Ok(settings) => settings
                .read()
                .map(|s| s.monarch.show_download_speed_in_bits)
                .unwrap_or(false),
            Err(_) => false,
        };

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
            PageTab::Download => self
                .download_page
                .view(show_speed_in_bits)
                .map(pages::Message::Download),
        };

        let main_content = container(
            iced::widget::Column::new()
                .push(
                    self.header
                        .view(
                            self.active_tab,
                            self.download_page.current_download_speed(),
                            show_speed_in_bits,
                            monarch_core::monarch_games::commands::get_pending_download_count(),
                            self.is_fullscreen,
                        )
                        .map(AppMessage::HeaderMessage),
                )
                .push(page_content.map(AppMessage::Page))
                .width(Fill)
                .height(Fill),
        )
        .width(Fill)
        .height(Fill);

        // Wrap the app in a rounded, thin-bordered frame with invisible
        // resize handles on every edge/corner.
        let frame = container(main_content)
            .padding(crate::gui::styles::window::FRAME_PADDING)
            .style(crate::gui::styles::window::frame)
            .width(Fill)
            .height(Fill);

        let app_content: Element<'_, AppMessage> = iced::widget::stack![frame, resize_handles()]
            .width(Fill)
            .height(Fill)
            .into();

        let Some(modal_state) = &self.active_modal else {
            return app_content;
        };
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
                        components::common::primary_button("Close", Some(AppMessage::CloseModal)),
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
                            Some(AppMessage::ConfirmModalAction(on_confirm.clone()))
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
            is_fullscreen: false,
            header: Header::default(),
            active_tab: PageTab::Home,
            previous_tab: PageTab::Home,
            home_page: pages::home::HomePage::default(),
            library_page: pages::library::LibraryPage::default(),
            search_page: pages::search::SearchPage::default(),
            settings_page: pages::settings::SettingsPage::default(),
            game_details_page: pages::game_details::GameDetailsPage::default(),
            store_details_page: pages::store_details::StoreDetailsPage::default(),
            download_page: pages::download::DownloadPage::default(),
            active_terminals: HashMap::new(),
            active_modal: None,
            state: Arc::new(RwLock::new(MonarchState::new())),
        }
    }
}

/// Overlay of invisible resize handles along the window edges and corners.
///
/// Each handle is a [`mouse_area`] that fires on mouse press (while the button
/// is held, which is required by the platform drag-resize loop) and requests the
/// corresponding direction from the OS via [`iced::window::drag_resize`].
fn resize_handles() -> iced::Element<'static, AppMessage> {
    use iced::widget::{column, container, mouse_area, row, Space};
    use iced::{mouse, window::Direction, Length};

    // A mouse area wrapped in a fixed-size container, since `MouseArea` itself
    // takes the size of its content.
    let handle =
        |direction: Direction, interaction: mouse::Interaction, width: Length, height: Length| {
            container(
                mouse_area(Space::new().width(Length::Fill).height(Length::Fill))
                    .on_press(AppMessage::ResizeWindow(direction))
                    .interaction(interaction),
            )
            .width(width)
            .height(height)
        };

    let h = crate::gui::styles::window::RESIZE_HANDLE;

    container(
        column![
            row![
                handle(
                    Direction::NorthWest,
                    mouse::Interaction::ResizingDiagonallyDown,
                    Length::Fixed(h),
                    Length::Fixed(h)
                ),
                handle(
                    Direction::North,
                    mouse::Interaction::ResizingVertically,
                    Length::Fill,
                    Length::Fixed(h)
                ),
                handle(
                    Direction::NorthEast,
                    mouse::Interaction::ResizingDiagonallyUp,
                    Length::Fixed(h),
                    Length::Fixed(h)
                ),
            ]
            .width(Length::Fill)
            .height(Length::Fixed(h)),
            row![
                handle(
                    Direction::West,
                    mouse::Interaction::ResizingHorizontally,
                    Length::Fixed(h),
                    Length::Fill
                ),
                Space::new().width(Length::Fill).height(Length::Fill),
                handle(
                    Direction::East,
                    mouse::Interaction::ResizingHorizontally,
                    Length::Fixed(h),
                    Length::Fill
                ),
            ]
            .width(Length::Fill)
            .height(Length::Fill),
            row![
                handle(
                    Direction::SouthWest,
                    mouse::Interaction::ResizingDiagonallyUp,
                    Length::Fixed(h),
                    Length::Fixed(h)
                ),
                handle(
                    Direction::South,
                    mouse::Interaction::ResizingVertically,
                    Length::Fill,
                    Length::Fixed(h)
                ),
                handle(
                    Direction::SouthEast,
                    mouse::Interaction::ResizingDiagonallyDown,
                    Length::Fixed(h),
                    Length::Fixed(h)
                ),
            ]
            .width(Length::Fill)
            .height(Length::Fixed(h)),
        ]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
