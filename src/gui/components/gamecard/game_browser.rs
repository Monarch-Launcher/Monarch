use crate::gui::components::gamecard::container::GameCardContainer;
use crate::gui::components::gamecard::properties::PropertiesModal;
use crate::gui::components::gamecard::GameCardMessage;
use crate::gui::styles;
use iced::widget::{container, stack, text};
use iced::{alignment, Color, Element, Length};

#[derive(Default, Debug, Clone)]
pub struct GameBrowser {
    pub games: GameCardContainer,
    properties_modal: Option<PropertiesModal>,
}

impl GameBrowser {
    pub fn update(&mut self, msg: GameCardMessage) -> iced::Task<GameCardMessage> {
        match &msg {
            GameCardMessage::Properties(prop_msg) => {
                if let Some(modal) = &mut self.properties_modal {
                    match prop_msg {
                        crate::gui::components::gamecard::properties::Message::Cancel => {
                            self.properties_modal = None;
                        }
                        _ => {
                            return modal
                                .update(prop_msg.clone())
                                .map(GameCardMessage::Properties);
                        }
                    }
                }
            }
            _ => {}
        }

        self.games.update(msg)
    }

    fn view_grid(&self, interactive: bool) -> Element<'_, GameCardMessage> {
        if self.games.is_empty() {
            container(text("No games found").size(32).font(styles::fonts::REGULAR))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
                .into()
        } else {
            container(iced::widget::scrollable(
                container(self.games.view(interactive))
                    .width(Length::Fill)
                    .padding(20)
                    .align_x(alignment::Horizontal::Center),
            ))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        }
    }

    pub fn view(&self) -> Element<'_, GameCardMessage> {
        let content = self.view_grid(true);

        if let Some(modal) = &self.properties_modal {
            stack![content, modal.view().map(GameCardMessage::Properties)].into()
        } else {
            content
        }
    }
}
