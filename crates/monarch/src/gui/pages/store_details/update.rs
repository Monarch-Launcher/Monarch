use crate::{
    gui::{
        components::modal::download_modal,
        pages::store_details::{Message, StoreDetailsPage},
    }, monarch_games::monarchgame::MonarchGame, monarch_utils,
};

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
                        opts.compatibility = Some(compat.name);
                    }

                    let game: MonarchGame = self.game.as_ref().unwrap().lock().unwrap().clone();

                    iced::Task::perform(
                        async move {
                            let _ = crate::monarch_games::commands::download_game(&game, &mut opts).await;
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
