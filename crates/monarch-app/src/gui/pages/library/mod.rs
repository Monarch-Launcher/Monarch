use iced::widget::{column, container, mouse_area, row, text};
use iced::Length::{self, Fill};
use iced::{alignment, Element};
use tracing::{error, info};

use crate::gui::components::common::icon_button;
use crate::gui::components::gamecard;
use crate::gui::components::gamecard::container::LibraryFilter;
use crate::gui::components::gamecard::game_browser::GameBrowser;
use crate::gui::resources::{ADD_FOLDER, FILTER, REFRESH};
use crate::gui::show_error;
use monarch_core::monarch_games::monarchgame::MonarchGame;
use monarch_core::{monarch_games, monarch_library, monarch_utils};

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
    UpdateGames(Vec<MonarchGame>),
    UpdateGameProperties,
    GameUpdated(MonarchGame),
    GameCard(gamecard::GameCardMessage),
    OpenGameDetails(MonarchGame),
    Tick,
    ScannerHovered(bool),
    AddGameHovered(bool),
    FilterHovered(bool),
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
    is_scanner_hovered: bool,
    is_add_hovered: bool,
    is_filter_hovered: bool,
    add_game_modal: Option<AddGameModal>,
    filter_modal: Option<FilterModal>,
}

impl LibraryPage {
    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::RefreshLibrary => {
                self.is_refreshing = true;
                self.dot_count = 3;
                self.tick_counter = 0;
                iced::Task::perform(
                    async move {
                        match monarch_games::commands::refresh_library().await {
                            Ok(games) => games,
                            Err(e) => {
                                show_error(e);
                                Vec::new()
                            }
                        }
                    },
                    Message::UpdateGames,
                )
            }
            Message::GameInstalled(game_id) => {
                // Backend already persisted the install; sync just that card
                // from in-memory state instead of a full library refresh.
                let game = match monarch_library::commands::get_library() {
                    Ok(games) => games.into_iter().find(|g| g.id == game_id),
                    Err(e) => {
                        show_error(e);
                        None
                    }
                };

                let Some(game) = game else {
                    return iced::Task::none();
                };

                self.browser.games.upsert_game(game.clone());

                iced::Task::perform(
                    async move {
                        info!("Downloading artwork for: {}", game.name);
                        let _ = monarch_games::commands::download_artwork(&game).await;

                        info!("Downloading cover for: {}", game.name);
                        if let Err(e) = monarch_games::commands::download_thumbnail(&game).await {
                            error!(
                                "Failed to download thumbnail for game {} ({}): {}",
                                game.id, game.thumbnail_url, e
                            );
                        }

                        info!("Updating game properties for : {}", game.name);
                        let mut game = game;
                        monarch_games::commands::get_game_properties(&mut game).await;
                        game
                    },
                    Message::GameUpdated,
                )
            }
            Message::GameRemoved(game_id) => {
                self.browser.games.remove_game(&game_id);
                iced::Task::none()
            }
            Message::UpdateGames(games) => {
                self.is_refreshing = false;

                let processed_games: Vec<MonarchGame> = games
                    .iter()
                    .cloned()
                    .map(|mut game| {
                        game.thumbnail_path = "".to_string();
                        game
                    })
                    .collect();

                // Trigger download tasks
                let download_tasks = iced::Task::batch(games.into_iter().map(|mut game| {
                    iced::Task::perform(
                        async move {
                            info!("Downloading artwork for: {}", game.name);
                            let _ = monarch_games::commands::download_artwork(&game).await;

                            info!("Downloading cover for: {}", game.name);
                            if let Err(e) = monarch_games::commands::download_thumbnail(&game).await
                            {
                                error!(
                                    "Failed to download thumbnail for game {} ({}): {}",
                                    game.id, game.thumbnail_url, e
                                );
                            }

                            info!("Updating game properties for : {}", game.name);
                            monarch_games::commands::get_game_properties(&mut game).await;
                            game
                        },
                        Message::GameUpdated,
                    )
                }));

                // Update browser games
                let _ = self
                    .browser
                    .update(gamecard::GameCardMessage::UpdateGames(processed_games));

                download_tasks
            }
            Message::UpdateGameProperties => {
                // Trigger download tasks
                let update_tasks = iced::Task::batch(self.browser.games.games.iter().cloned().map(
                    |mut gamecard| {
                        iced::Task::perform(
                            async move {
                                if !gamecard.game.has_properties() {
                                    monarch_games::commands::get_game_properties(
                                        &mut gamecard.game,
                                    )
                                    .await;
                                }
                                gamecard.game
                            },
                            Message::GameUpdated,
                        )
                    },
                ));

                update_tasks
            }
            Message::GameUpdated(game) => {
                if let Some(card) = self
                    .browser
                    .games
                    .games
                    .iter_mut()
                    .find(|c| c.game.id == game.id)
                {
                    card.update_game(game);
                }
                iced::Task::none()
            }
            Message::GameCard(game_card_message) => {
                // Check if it's a game press event
                if let gamecard::GameCardMessage::GamePressed(id) = &game_card_message {
                    // Find the game and emit OpenGameDetails
                    if let Some(game_card) =
                        self.browser.games.games.iter().find(|g| g.game.id == *id)
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
            Message::ScannerHovered(hovered) => {
                self.is_scanner_hovered = hovered;
                iced::Task::none()
            }
            Message::AddGameHovered(hovered) => {
                self.is_add_hovered = hovered;
                iced::Task::none()
            }
            Message::FilterHovered(hovered) => {
                self.is_filter_hovered = hovered;
                iced::Task::none()
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
                self.add_game_modal = Some(AddGameModal::default());
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
                iced::Task::perform(
                    async move { monarch_games::commands::manual_add_game(game).await },
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
        let Ok(settings_ptr) = monarch_utils::commands::get_settings() else {
            return;
        };
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
        let Ok(settings_ptr) = monarch_utils::commands::get_settings() else {
            return;
        };
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
            icon_button(None, false, REFRESH.clone(), 0.0)
        } else {
            mouse_area(icon_button(
                Some(Message::RefreshLibrary),
                self.is_scanner_hovered,
                REFRESH.clone(),
                refresh_rotation,
            ))
            .on_enter(Message::ScannerHovered(true))
            .on_exit(Message::ScannerHovered(false))
            .into()
        };

        let add_btn = if modal_active {
            icon_button(None, false, ADD_FOLDER.clone(), 0.0)
        } else {
            mouse_area(icon_button(
                Some(Message::OpenAddModal),
                self.is_add_hovered,
                ADD_FOLDER.clone(),
                0.0,
            ))
            .on_enter(Message::AddGameHovered(true))
            .on_exit(Message::AddGameHovered(false))
            .into()
        };

        let filter_btn = if modal_active {
            icon_button(None, false, FILTER.clone(), 0.0)
        } else {
            mouse_area(icon_button(
                Some(Message::FilterPressed),
                self.is_filter_hovered,
                FILTER.clone(),
                0.0,
            ))
            .on_enter(Message::FilterHovered(true))
            .on_exit(Message::FilterHovered(false))
            .into()
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

impl Default for LibraryPage {
    fn default() -> Self {
        let mut browser: GameBrowser = GameBrowser::default();

        match monarch_library::commands::get_library() {
            Ok(games) => {
                let _ = browser.update(gamecard::GameCardMessage::UpdateGames(games));
            }
            Err(e) => {
                show_error(e);
            }
        }

        let mut page = Self {
            browser,
            is_refreshing: false,
            dot_count: 3,
            tick_counter: 0,
            is_scanner_hovered: false,
            is_add_hovered: false,
            is_filter_hovered: false,
            add_game_modal: None,
            filter_modal: None,
        };

        page.load_persisted_filter();

        page
    }
}
