use crate::gui::components::common::{input_field, primary_button, secondary_button};
use crate::gui::components::gamecard::game_browser::GameBrowser;
use crate::gui::components::gamecard::{self, GameCardMessage};
use crate::gui::components::modal::Modal;
use crate::gui::styles;
use crate::monarch_games;
use crate::monarch_games::games::SearchResult;
use crate::monarch_games::monarchgame::{MonarchGame, MonarchWebApiGame};
use crate::monarch_games::stores::SearchFilter;
use iced::widget::{button, column, container, row, text, Space};
use iced::{alignment, Element, Length};
use tracing::error;

#[derive(Clone, Debug)]
pub enum Message {
    NameChanged(String),
    ExecPathChanged(String),
    ThumbPathChanged(String),
    SearchQueryChanged(String),
    PerformSearch,
    UpdateSearchResults(Vec<MonarchWebApiGame>),
    GameImgLoaded(MonarchWebApiGame),
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
    pub search_query: String,
    pub browser: GameBrowser,
    pub is_searching: bool,
    pub dot_count: u8,
    pub tick_counter: u8,
}

impl Default for AddGameModal {
    fn default() -> Self {
        Self {
            name: String::new(),
            exec_path: String::new(),
            thumb_path: String::new(),
            search_query: String::new(),
            browser: GameBrowser::default(),
            is_searching: false,
            dot_count: 3,
            tick_counter: 0,
        }
    }
}

impl AddGameModal {
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
            Message::ThumbPathChanged(path) => {
                self.thumb_path = path;
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
                iced::Task::perform(
                    async move {
                        crate::monarch_games::commands::search_games(query, SearchFilter::default())
                            .await
                    },
                    Message::UpdateSearchResults,
                )
            }
            Message::UpdateSearchResults(games) => self.update_games(games),
            Message::GameImgLoaded(game) => self.game_img_loaded(game),
            Message::GameCard(msg) => {
                if let GameCardMessage::GamePressed(id) = &msg {
                    if let Some(card) = self.browser.games.games.iter().find(|g| g.game.id == *id) {
                        self.name = card.game.name.clone();
                        self.thumb_path = card.game.thumbnail_path.clone();
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

        // Trigger download tasks
        let download_tasks = iced::Task::batch(games.iter().cloned().map(|game| {
            iced::Task::perform(
                async move {
                    if let Err(e) =
                        monarch_games::commands::download_thumbnail(&game.into_monarchgame()).await
                    {
                        error!("Failed to download thumbnail for game {}: {}", game.id, e);
                    }
                    game
                },
                Message::GameImgLoaded,
            )
        }));

        // Update browser games
        let _ = self.browser.update(gamecard::GameCardMessage::UpdateGames(
            processed_games
                .iter()
                .map(|g| g.into_monarchgame())
                .collect(),
        ));

        download_tasks
    }

    pub fn game_img_loaded(&mut self, game: MonarchWebApiGame) -> iced::Task<Message> {
        if let Some(card) = self
            .browser
            .games
            .games
            .iter_mut()
            .find(|c| c.game.id == game.id)
        {
            card.game.thumbnail_path = game.thumbnail_path.clone();
        }
        iced::Task::none()
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
                input_field(
                    "Path to game executable",
                    &self.exec_path,
                    Message::ExecPathChanged
                ),
                text("Thumbnail Path / URL").size(16),
                input_field(
                    "Path or URL to game thumbnail",
                    &self.thumb_path,
                    Message::ThumbPathChanged
                ),
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
