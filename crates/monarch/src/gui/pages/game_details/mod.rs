mod update;
mod view;

use std::sync::{Arc, Mutex};

use iced::widget::{container, stack, text};
use iced::{alignment, Element, Length};

use crate::gui::components::gamecard::actions::{self, ActionsModal};
use crate::gui::components::gamecard::properties::{self, PropertiesModal};
use crate::gui::styles;
use crate::monarch_games::monarchgame::MonarchGame;

#[derive(Clone, Debug)]
pub enum Message {
    BackPressed,
    LaunchGame,
    DownloadGame,
    OpenProperties,
    OpenActions,
    Properties(properties::Message),
    Actions(actions::Message),
    Nop(()),
}

pub struct GameDetailsPage {
    game: Option<Arc<Mutex<MonarchGame>>>,
    properties_modal: Option<PropertiesModal>,
    actions_modal: Option<ActionsModal>,
}

impl GameDetailsPage {
    pub fn new() -> Self {
        Self {
            game: None,
            properties_modal: None,
            actions_modal: None,
        }
    }

    pub fn set_game(&mut self, game: Arc<Mutex<MonarchGame>>) {
        self.game = Some(game);
    }

    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::BackPressed => {
                // This will be handled by the parent to navigate back
                iced::Task::none()
            }
            Message::LaunchGame => self.launch_game(),
            Message::LaunchGame => self.download_game(),
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

impl Default for GameDetailsPage {
    fn default() -> Self {
        Self::new()
    }
}
