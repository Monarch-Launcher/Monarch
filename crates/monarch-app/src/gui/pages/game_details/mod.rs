mod update;
mod view;

use std::sync::{Arc, RwLock};

use iced::widget::{container, stack, text};
use iced::{alignment, Element, Length};
use monarch_core::monarch_utils::monarch_game_downloader::MonarchDownloader;

use crate::gui::components::gamecard::actions;
use crate::gui::components::gamecard::edit_modal::{self, EditModal};
use crate::gui::components::gamecard::properties;
use crate::gui::components::modal::download_modal;
use crate::gui::styles;
use monarch_core::monarch_games::monarchgame::MonarchGame;
use monarch_core::monarch_utils::monarch_state::MonarchState;

#[derive(Clone, Debug)]
pub enum Message {
    BackPressed,
    BackHovered(bool),
    EditHovered(bool),
    /// Uninstall finished successfully; parent should drop the library card
    /// and navigate back to the library page.
    GameUninstalled(String),
    LaunchGame,
    DownloadGame,
    DownloadModalMessage(download_modal::Message),
    OpenProperties,
    Properties(properties::Message),
    Actions(actions::Message),
    /// Internal edit-modal view toggles (open/close the launch properties
    /// editor).
    EditModalMessage(edit_modal::Message),
    Nop(()),
}

pub struct GameDetailsPage {
    game: Option<Arc<RwLock<MonarchGame>>>,
    edit_modal: Option<EditModal>,
    download_modal: Option<download_modal::DownloadModal>,
    app_state: Arc<RwLock<MonarchState>>,
    downloader: Arc<RwLock<MonarchDownloader>>,
    is_back_hovered: bool,
    is_edit_hovered: bool,
}

/// Map a combined edit-modal message onto the page's message enum.
fn map_edit_message(message: edit_modal::Message) -> Message {
    match message {
        edit_modal::Message::Properties(p) => Message::Properties(p),
        edit_modal::Message::Actions(a) => Message::Actions(a),
        other => Message::EditModalMessage(other),
    }
}

impl GameDetailsPage {
    pub fn new(
        state_handle: Arc<RwLock<MonarchState>>,
        downloader_handle: Arc<RwLock<MonarchDownloader>>,
    ) -> Self {
        Self {
            game: None,
            edit_modal: None,
            download_modal: None,
            app_state: state_handle,
            downloader: downloader_handle,
            is_back_hovered: false,
            is_edit_hovered: false,
        }
    }

    pub fn set_game(&mut self, game: Arc<RwLock<MonarchGame>>) {
        self.game = Some(game);
    }

    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::BackPressed => {
                // This will be handled by the parent to navigate back
                iced::Task::none()
            }
            Message::BackHovered(hovered) => {
                self.is_back_hovered = hovered;
                iced::Task::none()
            }
            Message::EditHovered(hovered) => {
                self.is_edit_hovered = hovered;
                iced::Task::none()
            }
            Message::GameUninstalled(_) => {
                // Handled by the parent App (library remove + navigate).
                iced::Task::none()
            }
            Message::LaunchGame => self.launch_game(),
            Message::DownloadGame => self.download_game(),
            Message::DownloadModalMessage(m) => self.handle_download_modal_message(m),
            Message::OpenProperties => self.open_properties(),
            Message::Actions(actions_msg) => self.update_actions_msg(actions_msg),
            Message::EditModalMessage(msg) => {
                if let Some(modal) = &mut self.edit_modal {
                    return modal.update(msg).map(map_edit_message);
                }
                iced::Task::none()
            }
            Message::Properties(prop_msg) => self.update_properties_msg(prop_msg),
            _ => iced::Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.game.is_some() {
            let mut content = self.view_game_details();

            if let Some(modal) = &self.edit_modal {
                let mut layers = stack![content].width(Length::Fill).height(Length::Fill);

                layers = layers.push(modal.view().map(map_edit_message));
                content = layers.into();
            }

            if let Some(modal) = &self.download_modal {
                let mut layers = stack![content].width(Length::Fill).height(Length::Fill);

                layers = layers.push(modal.view().map(Message::DownloadModalMessage));
                content = layers.into();
            }

            content
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
