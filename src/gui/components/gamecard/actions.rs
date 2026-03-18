use std::sync::{Arc, Mutex};

use iced::widget::{column, row, Space};
use iced::{alignment, Element, Length, Task};
use tracing::error;

use crate::gui::components::common::{danger_button, secondary_button};
use crate::gui::show_error;
use crate::monarch_games;
use crate::monarch_games::games::GameType;
use crate::monarch_games::monarchgame::MonarchGame;

#[derive(Clone, Debug)]
pub enum Message {
    Uninstall,
    Close,
}

#[derive(Debug, Clone)]
pub struct ActionsModal {
    game: Arc<Mutex<MonarchGame>>,
}

impl ActionsModal {
    pub fn new(game: Arc<Mutex<MonarchGame>>) -> (Self, Task<Message>) {
        let modal = Self { game: game.clone() };

        (modal, iced::Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Uninstall => match self.game.lock() {
                Ok(game) => {
                    if &game.get_store_name() == "monarch" {
                        if let Err(e) = monarch_games::commands::manual_remove_game(&game) {
                            show_error(e);
                        }
                    } else {
                        return iced::Task::future(monarch_games::commands::remove_game(
                            game.name.clone(),
                            game.get_store_name(),
                            game.get_store_id(),
                        ))
                        .then(|out| {
                            if let Err(e) = out {
                                show_error(e);
                            }
                            iced::Task::<Message>::none()
                        });
                    }
                }
                Err(e) => {
                    error!("actions_modal::view() Failed to lock on self.game! | Err: {e}");
                    show_error("Failed to detect game to remove!");
                }
            },
            Message::Close => {}
        }
        iced::Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let remove_btn;

        match self.game.lock() {
            Ok(game) => {
                remove_btn = if &game.get_store_name() == "monarch" {
                    danger_button("Remove", Some(Message::Uninstall))
                } else {
                    danger_button("Uninstall", Some(Message::Uninstall))
                };
            }
            Err(e) => {
                error!("actions_modal::view() Failed to lock on self.game! | Err: {e}");
                show_error("Failed to open actions for selected game!");

                let content = column![];
                return crate::gui::components::modal::Modal::new("Actions", content)
                    .width(Length::Fixed(800.0))
                    .view();
            }
        }

        let content = column![
            remove_btn,
            Space::new().height(20),
            row![secondary_button("Done", Some(Message::Close))]
                .align_y(alignment::Vertical::Center)
        ]
        .spacing(10);

        crate::gui::components::modal::Modal::new("Actions", content)
            .width(Length::Fixed(800.0))
            .view()
    }
}
