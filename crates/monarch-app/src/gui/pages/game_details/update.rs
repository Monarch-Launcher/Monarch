use tracing::error;

use crate::gui::{
    components::{
        gamecard::{
            actions::{self, ActionsModal},
            properties::{self, PropertiesModal},
        },
        modal::download_modal,
    },
    pages::game_details::{GameDetailsPage, Message},
};
use monarch_core::monarch_games::{self, games::GameType, monarchgame::MonarchGame};

impl GameDetailsPage {
    pub fn launch_game(&self) -> iced::Task<Message> {
        let game = match &self.game {
            Some(g) => g.lock().unwrap().clone(),
            None => {
                error!("No game in GameDetailsPage!");
                return iced::Task::none();
            }
        };

        iced::Task::perform(
            async move {
                if let Err(e) = monarch_games::commands::launch_game(&game).await {
                    error!("Failed to launch: {} | Err: {}", game.name, e)
                }
            },
            Message::Nop,
        )
    }

    pub fn open_properties(&mut self) -> iced::Task<Message> {
        if let Some(game) = &self.game {
            let (modal, task) = PropertiesModal::new(game.clone());
            self.properties_modal = Some(modal);
            return task.map(Message::Properties);
        }
        iced::Task::none()
    }

    pub fn open_actions(&mut self) -> iced::Task<Message> {
        if let Some(game) = &self.game {
            let (modal, _task) = ActionsModal::new(game.clone());
            self.actions_modal = Some(modal);
        }
        iced::Task::none()
    }

    pub fn update_properties_msg(&mut self, prop_msg: properties::Message) -> iced::Task<Message> {
        if let properties::Message::Cancel = prop_msg {
            self.properties_modal = None;
            return iced::Task::none();
        }

        if let Some(modal) = &mut self.properties_modal {
            return modal.update(prop_msg).map(Message::Properties);
        }
        iced::Task::none()
    }

    pub fn update_actions_msg(&mut self, actions_msg: actions::Message) -> iced::Task<Message> {
        if let actions::Message::Close = actions_msg {
            self.actions_modal = None;
            return iced::Task::none();
        }

        if let Some(modal) = &mut self.actions_modal {
            return modal.update(actions_msg).map(Message::Actions);
        }
        iced::Task::none()
    }

    pub fn download_game(&mut self) -> iced::Task<Message> {
        let game = self.game.as_ref().unwrap().lock().unwrap();

        let (modal, task) = download_modal::DownloadModal::new(
            game.name.to_string(),
            game.get_store_name(),
            game.get_store_id(),
        );
        self.download_modal = Some(modal);
        task.map(Message::DownloadModalMessage)
    }

    pub fn handle_download_modal_message(
        &mut self,
        msg: download_modal::Message,
    ) -> iced::Task<Message> {
        match msg {
            download_modal::Message::Confirm => {
                if let Some(modal) = self.download_modal.take() {
                    let mut opts = modal.options;
                    if let Some(compat) = modal.selected_compatibility {
                        opts.compatibility = Some(compat.name);
                    }

                    let game: MonarchGame = self.game.as_ref().unwrap().lock().unwrap().clone();

                    iced::Task::perform(
                        async move {
                            let _ = monarch_core::monarch_games::commands::download_game(&game, &mut opts).await;
                        },
                        |_| Message::BackPressed, // Redirect on download init or just stay
                    )
                } else {
                    iced::Task::none()
                }
            }
            download_modal::Message::Cancel => {
                self.download_modal = None;
                iced::Task::none()
            }
            other => {
                if let Some(modal) = &mut self.download_modal {
                    modal.update(other).map(Message::DownloadModalMessage)
                } else {
                    iced::Task::none()
                }
            }
        }
    }
}
