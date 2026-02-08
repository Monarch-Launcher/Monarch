mod update;
mod view;

use iced::widget::{container, text};
use iced::window::Id;
use iced::{alignment, Color, Element, Length, Theme};

use crate::gui::components::gamecard::properties::{self, PropertiesModal};
use crate::monarch_games::monarchgame::MonarchGame;

#[derive(Clone, Debug)]
pub enum Message {
    BackPressed,
    LaunchGame,
    OpenTerminal(Id),
    CloseTerminal(Id),
    OpenProperties,
    Properties(properties::Message),
    Nop(()),
}

pub struct GameDetailsPage {
    game: Option<MonarchGame>,
    properties_modal: Option<PropertiesModal>,
}

impl GameDetailsPage {
    pub fn new() -> Self {
        Self {
            game: None,
            properties_modal: None,
        }
    }

    pub fn set_game(&mut self, game: MonarchGame) {
        self.game = Some(game);
    }

    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::BackPressed => {
                // This will be handled by the parent to navigate back
                iced::Task::none()
            }
            Message::LaunchGame => self.launch_game(),
            Message::OpenProperties => self.open_properties(),
            Message::Properties(prop_msg) => self.update_properties_msg(prop_msg),
            _ => iced::Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if let Some(game) = &self.game {
            self.view_game_details(game)
        } else {
            container(
                text("No game selected")
                    .size(32)
                    .style(|_theme: &Theme| text::Style {
                        color: Some(Color::from_rgb8(100, 100, 100)),
                    }),
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
