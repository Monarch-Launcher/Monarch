use crate::gui::{
    components::modal::download_modal,
    pages::store_details::{Message, StoreDetailsPage},
};
use monarch_core::monarch_utils;

impl StoreDetailsPage {
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

                    let downloader_handle_clone = self.downloader.clone();
                    let game_handle_clone = self.game.as_ref().unwrap().clone();
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

    pub fn open_store_page(&mut self, url: &str) -> iced::Task<Message> {
        monarch_utils::commands::open_external_link(&url);
        iced::Task::none()
    }
}
