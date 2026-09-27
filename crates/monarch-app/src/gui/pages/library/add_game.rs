use std::sync::{Arc, RwLock};

use crate::gui::components::common::{
    input_field, open_file_dialog, primary_button, secondary_button,
};
use crate::gui::components::gamecard::game_browser::GameBrowser;
use crate::gui::components::gamecard::{self, GameCardMessage};
use crate::gui::components::modal::Modal;
use crate::gui::{show_error, styles};
use iced::widget::{button, column, container, row, text, Space};
use iced::{alignment, Element, Length, Task};
use monarch_core::monarch_games;
use monarch_core::monarch_games::games::SearchResult;
use monarch_core::monarch_games::monarchgame::{MonarchGame, MonarchWebApiGame};
use monarch_core::monarch_games::stores::SearchFilter;
use monarch_core::monarch_utils::monarch_state::MonarchState;
use tracing::error;

#[derive(Clone, Debug)]
pub enum Message {
    NameChanged(String),
    ExecPathChanged(String),
    ExecPathDialog,
    CoverDialog,
    ArtworkDialog,
    ThumbPathChanged(String),
    ArtworkPathChanged(String),
    SearchQueryChanged(String),
    PerformSearch,
    UpdateSearchResults(Vec<MonarchWebApiGame>),
    GameImgLoaded,
    GameCard(GameCardMessage),
    AddGame,
    Cancel,
    Tick,
}

#[derive(Clone, Debug)]
pub struct AddGameModal {
    pub name: String,
    pub exec_path: String,
    pub thumb_path: String,
    pub artwork_path: String,
    pub search_query: String,
    pub browser: GameBrowser,
    pub is_searching: bool,
    pub dot_count: u8,
    pub tick_counter: u8,

    app_state: Arc<RwLock<MonarchState>>,
}

impl AddGameModal {
    pub fn new(state_handle: Arc<RwLock<MonarchState>>) -> Self {
        Self {
            name: String::new(),
            exec_path: String::new(),
            thumb_path: String::new(),
            artwork_path: String::new(),
            search_query: String::new(),
            browser: GameBrowser::default(),
            is_searching: false,
            dot_count: 3,
            tick_counter: 0,
            app_state: state_handle,
        }
    }

    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::NameChanged(name) => {
                self.name = name;
                iced::Task::none()
            }
            Message::ExecPathChanged(path) => {
                self.exec_path = path;
                iced::Task::none()
            }
            Message::ExecPathDialog => iced::Task::future(open_file_dialog(
                "Executables",
                &["exe", "app", "sh", "bin", "run", "x86_64"],
            ))
            .then(|handle| match handle {
                Some(file_handle) => iced::Task::done(Message::ExecPathChanged(
                    file_handle.path().to_string_lossy().to_string(),
                )),
                None => iced::Task::none(),
            }),
            Message::CoverDialog => {
                iced::Task::future(open_file_dialog("Images", &[".png", ".jpg", ".jpeg"])).then(
                    |handle| match handle {
                        Some(file_handle) => iced::Task::done(Message::ThumbPathChanged(
                            file_handle.path().to_string_lossy().to_string(),
                        )),
                        None => iced::Task::none(),
                    },
                )
            }
            Message::ArtworkDialog => {
                iced::Task::future(open_file_dialog("Images", &[".png", ".jpg", ".jpeg"])).then(
                    |handle| match handle {
                        Some(file_handle) => iced::Task::done(Message::ArtworkPathChanged(
                            file_handle.path().to_string_lossy().to_string(),
                        )),
                        None => iced::Task::none(),
                    },
                )
            }
            Message::ThumbPathChanged(path) => {
                self.thumb_path = path;
                iced::Task::none()
            }
            Message::ArtworkPathChanged(path) => {
                self.artwork_path = path;
                iced::Task::none()
            }
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
                iced::Task::none()
            }
            Message::PerformSearch => {
                if self.search_query.is_empty() {
                    return iced::Task::none();
                }
                self.is_searching = true;
                let query = self.search_query.clone();

                let settings_handle = match self.app_state.read() {
                    Ok(state) => state.get_settings_ptr(),
                    Err(e) => {
                        error!("AddGameModal::update() Failed to acquire read lock on state_handle! | Err: {e}");
                        show_error("Failed to search for games!");
                        return Task::none();
                    }
                };

                Task::perform(
                    async move {
                        monarch_core::monarch_games::commands::search_games(
                            settings_handle,
                            query,
                            SearchFilter::default(),
                        )
                        .await
                    },
                    Message::UpdateSearchResults,
                )
            }
            Message::UpdateSearchResults(games) => self.update_games(games),
            Message::GameImgLoaded => iced::Task::none(),
            Message::GameCard(msg) => {
                if let GameCardMessage::GamePressed(id) = &msg {
                    if let Some(card) = self
                        .browser
                        .games
                        .games
                        .iter()
                        .find(|g| g.game.read().unwrap().id == *id)
                    {
                        if let Ok(game) = card.game.read() {
                            self.name = game.name.clone();
                            self.thumb_path = game.thumbnail_path.clone();
                            self.artwork_path = game.artwork_path.clone();
                        }
                        // We don't have exec path from search results obviously
                    }
                }
                self.browser.update(msg).map(Message::GameCard)
            }
            Message::AddGame | Message::Cancel => iced::Task::none(),
            Message::Tick => {
                if self.is_searching {
                    self.tick_counter = self.tick_counter.wrapping_add(1);
                    if self.tick_counter % 60 == 0 {
                        self.dot_count = (self.dot_count % 3) + 1;
                    }
                }
                self.browser
                    .update(GameCardMessage::Tick)
                    .map(Message::GameCard)
            }
        }
    }

    pub fn update_games(&mut self, games: Vec<MonarchWebApiGame>) -> iced::Task<Message> {
        self.is_searching = false;

        let processed_games: Vec<MonarchWebApiGame> = games
            .iter()
            .cloned()
            .map(|mut game| {
                game.thumbnail_path = "".to_string();
                game
            })
            .collect();

        let processed_game_handles: Vec<Arc<RwLock<MonarchGame>>> = processed_games
            .iter()
            .map(|g| Arc::new(RwLock::new(g.into_monarchgame())))
            .collect();

        // Trigger download tasks
        let download_tasks = iced::Task::batch(processed_game_handles.iter().cloned().map(|game| {
            iced::Task::perform(
                async move {
                    if let Err(e) = monarch_games::commands::download_thumbnail(game.clone()).await
                    {
                        error!(
                            "Failed to download thumbnail for game {}: {}",
                            game.read().unwrap().id,
                            e
                        );
                    }
                },
                |_| Message::GameImgLoaded,
            )
        }));

        // Update browser games
        let _ = self.browser.update(gamecard::GameCardMessage::UpdateGames(
            processed_game_handles,
        ));

        download_tasks
    }

    pub fn view(&self) -> Element<'_, Message> {
        let content = column![
            column![
                text("Manual Entry")
                    .size(20)
                    .font(crate::gui::styles::fonts::BOLD),
                Space::new().height(5),
                text("Game Name").size(16),
                input_field("Enter game name", &self.name, Message::NameChanged),
                text("Executable Path").size(16),
                row![
                    input_field(
                        "Path to game executable",
                        &self.exec_path,
                        Message::ExecPathChanged
                    ),
                    secondary_button("Browse", Some(Message::ExecPathDialog))
                ]
                .spacing(10),
                text("Thumbnail Path / URL").size(16),
                row![
                    input_field(
                        "Path or URL to game thumbnail",
                        &self.thumb_path,
                        Message::ThumbPathChanged
                    ),
                    secondary_button("Browse", Some(Message::CoverDialog))
                ]
                .spacing(10),
                text("Artwork Path / URL").size(16),
                row![
                    input_field(
                        "Path or URL to game thumbnail",
                        &self.artwork_path,
                        Message::ArtworkPathChanged
                    ),
                    secondary_button("Browse", Some(Message::ArtworkDialog))
                ]
                .spacing(10),
            ]
            .spacing(10),
            Space::new().height(20),
            column![
                text("Search & Autofill")
                    .size(20)
                    .font(crate::gui::styles::fonts::BOLD),
                Space::new().height(5),
                row![
                    input_field(
                        "Search for a game...",
                        &self.search_query,
                        Message::SearchQueryChanged
                    ),
                    button(text("Search"))
                        .on_press(Message::PerformSearch)
                        .padding(10)
                        .style(styles::button::primary),
                ]
                .spacing(10),
                if self.is_searching {
                    let dots = ".".repeat(self.dot_count as usize);
                    container(
                        text(format!("Searching for games{dots}"))
                            .size(24)
                            .font(crate::gui::styles::fonts::REGULAR),
                    )
                    .width(Length::Fill)
                    .height(Length::Fixed(300.0))
                    .align_x(alignment::Horizontal::Center)
                    .align_y(alignment::Vertical::Center)
                } else {
                    container(self.browser.view(true).map(Message::GameCard))
                        .height(Length::Fixed(300.0))
                },
            ]
            .spacing(10),
            row![
                Space::new().width(Length::Fill),
                secondary_button("Cancel", Some(Message::Cancel)),
                primary_button("Add Game", Some(Message::AddGame)),
            ]
            .spacing(10)
        ]
        .spacing(15);

        Modal::new("Add Game Manually", content)
            .on_close(Message::Cancel)
            .width(Length::Fixed(700.0))
            .view()
    }
}
