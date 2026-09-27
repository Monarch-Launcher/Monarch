use iced::widget::{container, text};
use iced::{alignment, Element, Length, Task};
use monarch_core::monarch_utils::monarch_game_downloader::MonarchDownloader;
use std::sync::{Arc, RwLock};
use tracing::error;

use monarch_core::monarch_games::monarchgame::MonarchGame;
use monarch_core::monarch_utils::monarch_state::MonarchState;

use crate::gui::components::modal::download_modal;
use crate::gui::{show_error, styles};

mod update;
mod view;

#[derive(Clone, Debug)]
pub enum Message {
    BackPressed,
    DownloadGame(Arc<RwLock<MonarchGame>>),
    DownloadModalMessage(download_modal::Message),
    OpenStorePage(String),
    ArtworkDownloaded,
    PropertiesLoaded,
}

pub struct StoreDetailsPage {
    game: Option<Arc<RwLock<MonarchGame>>>,
    pub artwork_loaded: bool,
    pub download_modal: Option<download_modal::DownloadModal>,

    app_state: Arc<RwLock<MonarchState>>,
    downloader: Arc<RwLock<MonarchDownloader>>,
}

impl StoreDetailsPage {
    pub fn new(
        state_handle: Arc<RwLock<MonarchState>>,
        downloader_handle: Arc<RwLock<MonarchDownloader>>,
    ) -> Self {
        Self {
            game: None,
            artwork_loaded: false,
            download_modal: None,

            app_state: state_handle,
            downloader: downloader_handle,
        }
    }

    pub fn set_game(&mut self, game: Arc<RwLock<MonarchGame>>) -> iced::Task<Message> {
        self.game = Some(game.clone());
        self.artwork_loaded = false;
        self.download_modal = None;

        let game_handle_clone = game.clone();
        let state_handle_clone = self.app_state.clone();
        iced::Task::perform(
            async move {
                let has_props = match game_handle_clone.read() {
                    Ok(game) => game.has_properties(),
                    Err(e) => {
                        error!("StoreDetailsPage::set_game() Failed to acquire read lock on game_handle! | Err: {e}");
                        false
                    }
                };

                if !has_props {
                    monarch_core::monarch_games::commands::get_game_properties(
                        state_handle_clone,
                        game_handle_clone,
                    )
                    .await;
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
            Message::DownloadGame(game) => {
                let settings_handle = match self.app_state.read() {
                    Ok(state) => state.get_settings_ptr(),
                    Err(e) => {
                        error!("StoreDetailsPage::update() Failed to acquire read lock on state_handle! | Err: {e}");
                        show_error("Failed to download game!");
                        return Task::none();
                    }
                };
                match download_modal::DownloadModal::new(settings_handle, game) {
                    Ok((modal, task)) => {
                        self.download_modal = Some(modal);
                        task.map(Message::DownloadModalMessage)
                    }
                    Err(e) => {
                        error!("StoreDetailsPage::update() Failed to create new DownloadModal! | Err: {e}");
                        show_error("Failed to download new game!");
                        return Task::none();
                    }
                }
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
