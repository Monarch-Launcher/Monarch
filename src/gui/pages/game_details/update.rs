use tracing::error;

use crate::{
    gui::{
        components::gamecard::{
            actions::{self, ActionsModal},
            properties::{self, PropertiesModal},
        },
        pages::game_details::{GameDetailsPage, Message},
    },
    monarch_games,
};

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
}
