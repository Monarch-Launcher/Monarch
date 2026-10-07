use std::sync::{Arc, RwLock};

use iced::widget::{column, container, row, text};
use iced::Length::{self, Fill};
use iced::{alignment, Element, Task};
use monarch_core::monarch_utils::monarch_state::MonarchState;
use tracing::{error, info};

use crate::gui::components::common::icon_button;
use crate::gui::components::gamecard;
use crate::gui::components::gamecard::container::LibraryFilter;
use crate::gui::components::gamecard::game_browser::GameBrowser;
use crate::gui::resources::{ADD_FOLDER, FILTER, REFRESH};
use crate::gui::show_error;
use monarch_core::monarch_games::monarchgame::MonarchGame;
use monarch_core::{monarch_games, monarch_utils};

mod add_game;
use add_game::AddGameModal;

mod filter;
use filter::FilterModal;

#[derive(Clone, Debug)]
pub enum Message {
    RefreshLibrary,
    /// Cheap post-install update: read the game from MONARCH_STATE and upsert
    /// it into the browser without rescanning Steam/Epic.
    GameInstalled(String),
    /// Cheap post-uninstall update: drop the card without a full refresh.
    GameRemoved(String),
    UpdateGames,
    UpdateGameProperties,
    GameUpdated(Arc<RwLock<MonarchGame>>),
    GameCard(gamecard::GameCardMessage),
    OpenGameDetails(Arc<RwLock<MonarchGame>>),
    Tick,
    FilterPressed,
    FilterModal(filter::Message),
    OpenAddModal,
    AddModal(add_game::Message),
    AddGame(MonarchGame),
}

#[derive(Debug, Clone)]
pub struct LibraryPage {
    browser: GameBrowser,
    is_refreshing: bool,
    dot_count: u8,
    tick_counter: u8,
    add_game_modal: Option<AddGameModal>,
    filter_modal: Option<FilterModal>,

    app_state: Arc<RwLock<MonarchState>>,
}

impl LibraryPage {
    pub fn new(state_handle: Arc<RwLock<MonarchState>>) -> Self {
        let mut browser: GameBrowser = GameBrowser::default();
        if let Ok(state) = state_handle.read() {
            let _ = browser.update(gamecard::GameCardMessage::UpdateGames(
                state.get_library_games().to_vec(),
            ));
        }

        let mut page = Self {
            browser,
            is_refreshing: false,
            dot_count: 3,
            tick_counter: 0,
            add_game_modal: None,
            filter_modal: None,

            app_state: state_handle,
        };
        page.load_persisted_filter();
        page
    }

    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::RefreshLibrary => {
                self.is_refreshing = true;
                self.dot_count = 3;
                self.tick_counter = 0;
                let state_handle_clone = self.app_state.clone();
                iced::Task::perform(
                    async move {
                        if let Err(e) =
                            monarch_games::commands::refresh_library(state_handle_clone).await
                        {
                            show_error(e);
                        }
                    },
                    |_| Message::UpdateGames,
                )
            }
            Message::GameInstalled(game_id) => {
                let game_handle: Arc<RwLock<MonarchGame>> = match self.app_state.read() {
                    Ok(state) => {
                        match state
                            .get_library_games()
                            .iter()
                            .cloned()
                            .find(|g| g.read().unwrap().id == game_id)
                        {
                            Some(game) => game,
                            None => return Task::none(),
                        }
                    }
                    Err(e) => {
                        error!("LibraryPage::update() Failed to acquire read lock on state_handle! | Err: {e}");
                        show_error("Failed to finish game installation!");
                        return Task::none();
                    }
                };

                self.browser.games.upsert_game(game_handle.clone());

                let state_handle_clone = self.app_state.clone();
                let settings_handle_clone = match self.app_state.read() {
                    Ok(state) => state.get_settings_ptr(),
                    Err(e) => {
                        error!("LibraryPage::update() Failed to acquire read lock on state_handle! | Err: {e}");
                        show_error("Failed to get installed games!");
                        return Task::none();
                    }
                };

                iced::Task::perform(
                    async move {
                        let game_clone = game_handle.read().unwrap().clone();
                        info!("Downloading artwork for: {}", game_clone.name);
                        let _ = monarch_games::commands::download_artwork(
                            settings_handle_clone.clone(),
                            game_handle.clone(),
                        )
                        .await;

                        info!("Downloading cover for: {}", game_clone.name);
                        if let Err(e) = monarch_games::commands::download_cover(
                            settings_handle_clone.clone(),
                            game_handle.clone(),
                        )
                        .await
                        {
                            error!(
                                "Failed to download cover for game {} ({}): {}",
                                game_clone.id, game_clone.cover_url, e
                            );
                        }

                        if !game_clone.is_installed {
                            info!("Downloading greyscale for: {}", game_clone.name);
                            let _ = monarch_games::commands::download_greyscale(
                                settings_handle_clone.clone(),
                                game_handle.clone(),
                            )
                            .await;
                        }

                        info!("Updating game properties for : {}", game_clone.name);
                        monarch_games::commands::get_game_properties(
                            state_handle_clone,
                            game_handle.clone(),
                        )
                        .await;
                        game_handle
                    },
                    Message::GameUpdated,
                )
            }
            Message::GameRemoved(game_id) => {
                self.browser.games.remove_game(&game_id);
                iced::Task::none()
            }
            Message::UpdateGames => {
                self.is_refreshing = false;

                let new_games = match self.app_state.read() {
                    Ok(state) => state.get_library_games().to_vec(),
                    Err(e) => {
                        error!("LibraryPage::update() Failed to acquire read lock on state_handle! | Err: {e}");
                        show_error("Failed to refresh library!");
                        return Task::none();
                    }
                };
                let state_handle = self.app_state.clone();

                let _ = self
                    .browser
                    .update(gamecard::GameCardMessage::UpdateGames(new_games.clone()));

                // Trigger download tasks. Each game emits GameUpdated as soon as
                // its images are ready (artwork/cover/greyscale early-return
                // when already cached), then chains a second task that performs
                // the slower properties enrichment — removing the network call
                // from the image-critical path.
                let download_tasks = iced::Task::batch(new_games.into_iter().map(move |game| {
                    // `map`'s closure is FnMut (called once per game), so it can't
                    // move `state_handle` itself; hand a fresh Arc to each task.
                    let state_handle = state_handle.clone();
                    let settings_handle_clone = match self.app_state.read() {
                        Ok(state) => state.get_settings_ptr(),
                        Err(e) => {
                            error!("LibraryPage::update() Failed to acquire read lock on state_handle! | Err: {e}");
                            show_error("Failed to refresh library!");
                            return Task::none();
                        }
                    };

                    let image_task: iced::Task<Arc<RwLock<MonarchGame>>> = iced::Task::perform(
                        async move {
                            let game_clone = match game.read() {
                                Ok(g) => g.clone(),
                                Err(e) => {
                                    error!("LibraryPage::update() Failed to acquire read lock on game! | Err: {e}");
                                    return game.clone();
                                }
                            };

                            info!("Downloading artwork for: {}", game_clone.name);
                            let _ = monarch_games::commands::download_artwork(settings_handle_clone.clone(), game.clone()).await;

                            info!("Downloading cover for: {}", game_clone.name);
                            if let Err(e) =
                                monarch_games::commands::download_cover(settings_handle_clone.clone(), game.clone()).await
                            {
                                error!(
                                    "Failed to download cover for game {} ({}): {}",
                                    game_clone.id, game_clone.cover_url, e
                                );
                            }

                            if !game_clone.is_installed {
                                info!("Downloading greyscale for: {}", game_clone.name);
                                let _ =
                                    monarch_games::commands::download_greyscale(settings_handle_clone, game.clone()).await;
                            }

                            game
                        },
                        |game| game,
                    );

                    image_task.then(move |game| {
                        // `then`'s closure is also FnMut, so clone the handles
                        // into the async block instead of moving them.
                        let state = state_handle.clone();
                        let props_handle = game.clone();

                        iced::Task::batch([
                            iced::Task::done(Message::GameUpdated(game)),
                            iced::Task::perform(
                                async move {
                                    if let Ok(g) = props_handle.read() {
                                        info!("Updating game properties for : {}", g.name);
                                    }

                                    monarch_games::commands::get_game_properties(
                                        state,
                                        props_handle.clone(),
                                    )
                                    .await;

                                    props_handle
                                },
                                Message::GameUpdated,
                            ),
                        ])
                    })
                }));

                download_tasks
            }
            Message::UpdateGameProperties => {
                // Trigger download tasks
                let update_tasks =
                    iced::Task::batch(self.browser.games.games.iter().cloned().map(|gamecard| {
                        let state_handle_clone = self.app_state.clone();
                        iced::Task::perform(
                            async move {
                                if !gamecard.game.read().unwrap().has_properties() {
                                    monarch_games::commands::get_game_properties(
                                        state_handle_clone,
                                        gamecard.game.clone(),
                                    )
                                    .await;
                                }
                                gamecard.game
                            },
                            Message::GameUpdated,
                        )
                    }));

                update_tasks
            }
            Message::GameUpdated(game) => {
                if let Some(card) = self
                    .browser
                    .games
                    .games
                    .iter_mut()
                    .find(|c| c.game.read().unwrap().id == game.read().unwrap().id)
                {
                    card.update_game(game);
                }
                iced::Task::none()
            }
            Message::GameCard(game_card_message) => {
                // Check if it's a game press event
                if let gamecard::GameCardMessage::GamePressed(id) = &game_card_message {
                    // Find the game and emit OpenGameDetails
                    if let Some(game_card) = self
                        .browser
                        .games
                        .games
                        .iter()
                        .find(|g| g.game.read().unwrap().id == *id)
                    {
                        return iced::Task::done(Message::OpenGameDetails(game_card.game.clone()));
                    }
                }

                self.browser
                    .update(game_card_message)
                    .map(Message::GameCard)
            }
            Message::OpenGameDetails(_) => {
                // This will be handled by the parent App
                iced::Task::none()
            }
            Message::Tick => {
                if self.is_refreshing {
                    self.tick_counter = (self.tick_counter + 1) % 60;
                    if self.tick_counter == 0 {
                        self.dot_count = (self.dot_count % 3) + 1;
                    }
                }

                let mut tasks = vec![self
                    .browser
                    .update(gamecard::GameCardMessage::Tick)
                    .map(Message::GameCard)];

                if let Some(modal) = &mut self.add_game_modal {
                    tasks.push(modal.update(add_game::Message::Tick).map(Message::AddModal));
                }

                iced::Task::batch(tasks)
            }
            Message::FilterPressed => {
                let filter = self.browser.games.filter.clone();
                self.filter_modal = Some(FilterModal::new(filter));
                iced::Task::none()
            }
            Message::FilterModal(modal_msg) => {
                if let Some(modal) = &mut self.filter_modal {
                    modal.update(modal_msg.clone());
                    match modal_msg {
                        filter::Message::Apply => {
                            self.browser.games.filter = modal.filter.clone();
                            self.persist_filter();
                            self.filter_modal = None;
                        }
                        filter::Message::Cancel => {
                            self.filter_modal = None;
                        }
                        _ => {}
                    }
                }
                iced::Task::none()
            }
            Message::OpenAddModal => {
                self.add_game_modal = Some(AddGameModal::new(self.app_state.clone()));
                iced::Task::none()
            }
            Message::AddModal(modal_msg) => {
                if let Some(modal) = &mut self.add_game_modal {
                    match modal_msg {
                        add_game::Message::Cancel => {
                            self.add_game_modal = None;
                            iced::Task::none()
                        }
                        add_game::Message::AddGame => {
                            let game = MonarchGame::new(
                                &modal.name,
                                0,
                                "monarch",
                                "",
                                "",
                                &modal.exec_path,
                                &modal.thumb_path,
                            );
                            self.add_game_modal = None;
                            iced::Task::done(Message::AddGame(game))
                        }
                        _ => modal.update(modal_msg).map(Message::AddModal),
                    }
                } else {
                    iced::Task::none()
                }
            }
            Message::AddGame(game) => {
                let state_handle_clone = self.app_state.clone();
                iced::Task::perform(
                    async move {
                        monarch_games::commands::manual_add_game(state_handle_clone, game).await
                    },
                    |res| match res {
                        Ok(_) => Message::RefreshLibrary,
                        Err(e) => {
                            show_error(&format!("Failed to add game: {}", e));
                            Message::Tick // Dummy message
                        }
                    },
                )
            }
        }
    }

    /// Persist the current library filter to settings if the corresponding
    /// setting is enabled. Best-effort: failures only log.
    fn persist_filter(&self) {
        let Ok(state) = self.app_state.read() else {
            return;
        };
        let settings_ptr = state.get_settings_ptr();
        let Ok(mut settings) = settings_ptr.write() else {
            return;
        };
        if !settings.monarch.persist_library_filters {
            return;
        }
        settings.monarch.library_filter_steam = self.browser.games.filter.steam;
        settings.monarch.library_filter_epic = self.browser.games.filter.epic;
        settings.monarch.library_filter_installed = self.browser.games.filter.installed;
        settings.monarch.library_filter_uninstalled = self.browser.games.filter.uninstalled;
        if let Err(e) = monarch_utils::commands::write_settings(&settings) {
            error!("Failed to persist library filter | Err: {e}");
        }
    }

    /// Load the persisted library filter at startup when the setting is
    /// enabled.
    fn load_persisted_filter(&mut self) {
        let Ok(state) = self.app_state.read() else {
            return;
        };
        let settings_ptr = state.get_settings_ptr();
        let Ok(settings) = settings_ptr.read() else {
            return;
        };
        if !settings.monarch.persist_library_filters {
            return;
        }
        self.browser.games.filter = LibraryFilter {
            steam: settings.monarch.library_filter_steam,
            epic: settings.monarch.library_filter_epic,
            installed: settings.monarch.library_filter_installed,
            uninstalled: settings.monarch.library_filter_uninstalled,
        };
    }

    pub fn view(&self) -> Element<'_, Message> {
        let modal_active = self.add_game_modal.is_some() || self.filter_modal.is_some();

        let refresh_rotation = if self.is_refreshing {
            -(self.tick_counter as f32 / 60.0) * std::f32::consts::TAU
        } else {
            0.0
        };

        let refresh_btn = if modal_active {
            icon_button(None, REFRESH.clone(), 0.0)
        } else {
            icon_button(
                Some(Message::RefreshLibrary),
                REFRESH.clone(),
                refresh_rotation,
            )
        };

        let add_btn = if modal_active {
            icon_button(None, ADD_FOLDER.clone(), 0.0)
        } else {
            icon_button(Some(Message::OpenAddModal), ADD_FOLDER.clone(), 0.0)
        };

        let filter_btn = if modal_active {
            icon_button(None, FILTER.clone(), 0.0)
        } else {
            icon_button(Some(Message::FilterPressed), FILTER.clone(), 0.0)
        };

        let games_content: Element<'_, Message> =
            if self.is_refreshing && self.browser.games.is_empty() {
                let dots = ".".repeat(self.dot_count as usize);
                container(
                    column![text(format!("Looking for games{dots}"))
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
                self.browser.view(!modal_active).map(Message::GameCard)
            };

        let base_content = container(
            column![
                container(row![refresh_btn, add_btn, filter_btn].spacing(10))
                    .width(Fill)
                    .padding(30)
                    .align_x(alignment::Horizontal::Left)
                    .align_y(alignment::Vertical::Top),
                games_content
            ]
            .align_x(alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill);

        if let Some(modal) = &self.add_game_modal {
            iced::widget::stack![base_content, modal.view().map(Message::AddModal)].into()
        } else if let Some(modal) = &self.filter_modal {
            iced::widget::stack![base_content, modal.view().map(Message::FilterModal)].into()
        } else {
            base_content.into()
        }
    }
}
