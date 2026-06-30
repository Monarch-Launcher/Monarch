use std::sync::{Arc, Mutex};

use iced::widget::{container, text};
use iced::{alignment, Element, Length};

use crate::gui::components::modal::download_modal;
use crate::gui::styles;
use crate::monarch_games::monarchgame::MonarchGame;

mod update;
mod view;

#[derive(Clone, Debug)]
pub enum Message {
    BackPressed,
    DownloadGame(String, String, String),
    DownloadModalMessage(download_modal::Message),
    OpenStorePage(String),
    ArtworkDownloaded,
    PropertiesLoaded,
}

pub struct StoreDetailsPage {
    game: Option<Arc<Mutex<MonarchGame>>>,
    pub artwork_loaded: bool,
    pub download_modal: Option<download_modal::DownloadModal>,
}

impl StoreDetailsPage {
    pub fn new() -> Self {
        Self {
            game: None,
            artwork_loaded: false,
            download_modal: None,
        }
    }

    pub fn set_game(&mut self, game: Arc<Mutex<MonarchGame>>) -> iced::Task<Message> {
        self.game = Some(game.clone());
        self.artwork_loaded = false;
        self.download_modal = None;

        let game_clone = game.clone();
        iced::Task::perform(
            async move {
                let (mut game_copy, has_props) = {
                    let game_lock = game_clone.lock().unwrap();
                    (game_lock.clone(), game_lock.has_properties())
                };

                if !has_props {
                    crate::monarch_games::commands::get_game_properties(&mut game_copy).await;
                    let mut game_lock = game_clone.lock().unwrap();
                    *game_lock = game_copy;
                }
            },
            |_| Message::PropertiesLoaded,
        )
    }

    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::BackPressed => {
                // Handled in parent
                iced::Task::none()
            }
            Message::DownloadGame(name, store, store_id) => {
                let (modal, task) = download_modal::DownloadModal::new(name, store, store_id);
                self.download_modal = Some(modal);
                task.map(Message::DownloadModalMessage)
            }
            Message::DownloadModalMessage(m) => self.handle_download_modal_message(m),
            Message::OpenStorePage(url) => self.open_store_page(&url),
            Message::ArtworkDownloaded => {
                self.artwork_loaded = true;
                iced::Task::none()
            }
            Message::PropertiesLoaded => {
                // Properties are updated in the background task because game is Arc<Mutex<MonarchGame>>
                iced::Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.game.is_some() {
            self.view_store_details()
        } else {
            container(
                text("No game selected")
                    .size(32)
                    .font(styles::fonts::REGULAR),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .into()
        }
    }
}

impl Default for StoreDetailsPage {
    fn default() -> Self {
        Self::new()
    }
}
