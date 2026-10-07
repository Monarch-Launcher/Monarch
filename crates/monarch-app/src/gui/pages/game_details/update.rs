use iced::Task;
use tracing::error;

use crate::gui::{
    components::{
        gamecard::{
            actions,
            edit_modal::{self, EditModal},
            properties,
        },
        modal::download_modal,
    },
    pages::game_details::{GameDetailsPage, Message},
    show_error,
};
use monarch_core::monarch_games::{self, games::GameType};

impl GameDetailsPage {
    pub fn launch_game(&self) -> iced::Task<Message> {
        if self.game.is_none() {
            show_error("Failed to launch game! (No game selected)");
            return Task::none();
        }

        let game_handle_clone = self.game.as_ref().unwrap().clone();
        let settings_handle_clone = match self.app_state.read() {
            Ok(state) => state.get_settings_ptr(),
            Err(e) => {
                error!("GameDetailsPage::launch_game() Failed to acquire read lock on state_handle! | Err: {e}");
                show_error("Failed to launch game!");
                return Task::none();
            }
        };

        iced::Task::perform(
            async move {
                if let Err(e) =
                    monarch_games::commands::launch_game(settings_handle_clone, game_handle_clone)
                        .await
                {
                    error!(
                        "GameDetailsPage::launch_game() Failed to launch game! | Err: {}",
                        e
                    )
                }
            },
            Message::Nop,
        )
    }

    pub fn open_properties(&mut self) -> iced::Task<Message> {
        if let Some(game) = &self.game {
            let (modal, task) = EditModal::new(
                self.app_state.clone(),
                game.clone(),
                self.downloader.clone(),
            );
            self.edit_modal = Some(modal);
            return task.map(super::map_edit_message);
        }
        iced::Task::none()
    }

    pub fn update_properties_msg(&mut self, prop_msg: properties::Message) -> iced::Task<Message> {
        // Cancel is also produced by the ✕ button and by a completed save,
        // and closes the whole edit modal.
        if let properties::Message::Cancel = prop_msg {
            self.edit_modal = None;
            return iced::Task::none();
        }

        if let Some(modal) = &mut self.edit_modal {
            return modal
                .update(edit_modal::Message::Properties(prop_msg))
                .map(super::map_edit_message);
        }
        iced::Task::none()
    }

    pub fn update_actions_msg(&mut self, actions_msg: actions::Message) -> iced::Task<Message> {
        if let actions::Message::Close = actions_msg {
            self.edit_modal = None;
            return iced::Task::none();
        }

        if let actions::Message::Uninstalled(Ok(game_id)) = &actions_msg {
            self.edit_modal = None;
            return iced::Task::done(Message::GameUninstalled(game_id.clone()));
        }

        if let Some(modal) = &mut self.edit_modal {
            return modal
                .update(edit_modal::Message::Actions(actions_msg))
                .map(super::map_edit_message);
        }
        iced::Task::none()
    }

    pub fn download_game(&mut self) -> iced::Task<Message> {
        let game_handle_clone = self.game.as_ref().unwrap().clone();
        let settings_handle = match self.app_state.read() {
            Ok(state) => state.get_settings_ptr(),
            Err(e) => {
                error!("GameDetailsPage::download_game() Failed to acquire read lock on state_handle! | Err: {e}");
                show_error("Failed to download game!");
                return Task::none();
            }
        };

        match game_handle_clone.read() {
            Ok(game) => {
                if !game.get_store().store_enabled(settings_handle.clone()) {
                    show_error("Monarch is not allowed to download from this source. Please check your enabled stores in the settings.");
                    return Task::none()
                }
            }
            Err(e) => {
                error!("GameDetailsPage::download_game() Failed to acquire read lock on game_handle! | Err: {e}");
                show_error("Failed to check if Monarch is allowed to manage games from current source!");
                return Task::none()
            }
        }

        let modal_result =
            download_modal::DownloadModal::new(settings_handle, game_handle_clone);

        match modal_result {
            Ok((modal, task)) => {
                self.download_modal = Some(modal);
                task.map(Message::DownloadModalMessage)
            }
            Err(e) => {
                error!(
                    "GameDetailsPage::download_game() Failed to create DownloadModal! | Err: {e}"
                );
                show_error("Failed to download game!");
                Task::none()
            }
        }
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
                        // Proton launch uses PROTONPATH, so store the path/codename.
                        if !compat.path.is_empty() {
                            opts.compatibility = Some(compat.path);
                        }
                    }

                    let game_handle_clone = self.game.as_ref().unwrap().clone();
                    let downloader_handle_clone = self.downloader.clone();

                    iced::Task::perform(
                        async move {
                            let _ = monarch_core::monarch_games::commands::download_game(
                                downloader_handle_clone,
                                game_handle_clone,
                                opts,
                            )
                            .await;
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
