mod update;
mod view;

use std::sync::{Arc, RwLock};

use iced::widget::{container, stack, text};
use iced::{alignment, Element, Length};
use monarch_core::monarch_utils::monarch_game_downloader::MonarchDownloader;

use crate::gui::components::gamecard::actions::{self, ActionsModal};
use crate::gui::components::gamecard::properties::{self, PropertiesModal};
use crate::gui::components::modal::download_modal;
use crate::gui::styles;
use monarch_core::monarch_games::monarchgame::MonarchGame;
use monarch_core::monarch_utils::monarch_state::MonarchState;

#[derive(Clone, Debug)]
pub enum Message {
    BackPressed,
    /// Uninstall finished successfully; parent should drop the library card
    /// and navigate back to the library page.
    GameUninstalled(String),
    LaunchGame,
    DownloadGame,
    DownloadModalMessage(download_modal::Message),
    OpenProperties,
    OpenActions,
    Properties(properties::Message),
    Actions(actions::Message),
    Nop(()),
}

pub struct GameDetailsPage {
    game: Option<Arc<RwLock<MonarchGame>>>,
    properties_modal: Option<PropertiesModal>,
    actions_modal: Option<ActionsModal>,
    download_modal: Option<download_modal::DownloadModal>,
    app_state: Arc<RwLock<MonarchState>>,
    downloader: Arc<RwLock<MonarchDownloader>>,
}

impl GameDetailsPage {
    pub fn new(
        state_handle: Arc<RwLock<MonarchState>>,
        downloader_handle: Arc<RwLock<MonarchDownloader>>,
    ) -> Self {
        Self {
            game: None,
            properties_modal: None,
            actions_modal: None,
            download_modal: None,
            app_state: state_handle,
            downloader: downloader_handle,
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
            Message::GameUninstalled(_) => {
                // Handled by the parent App (library remove + navigate).
                iced::Task::none()
            }
            Message::LaunchGame => self.launch_game(),
            Message::DownloadGame => self.download_game(),
            Message::DownloadModalMessage(m) => self.handle_download_modal_message(m),
            Message::OpenProperties => self.open_properties(),
            Message::OpenActions => self.open_actions(),
            Message::Actions(actions_msg) => self.update_actions_msg(actions_msg),
            Message::Properties(prop_msg) => self.update_properties_msg(prop_msg),
            _ => iced::Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.game.is_some() {
            let mut content = self.view_game_details();

            if let Some(modal) = &self.properties_modal {
                let mut layers = stack![content].width(Length::Fill).height(Length::Fill);

                layers = layers.push(modal.view().map(Message::Properties));
                content = layers.into();
            } else if let Some(modal) = &self.actions_modal {
                let mut layers = stack![content].width(Length::Fill).height(Length::Fill);

                layers = layers.push(modal.view().map(Message::Actions));
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
