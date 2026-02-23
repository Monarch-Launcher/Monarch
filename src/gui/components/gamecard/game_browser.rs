use crate::gui::components::gamecard::container::GameCardContainer;
use crate::gui::components::gamecard::properties::PropertiesModal;
use crate::gui::components::gamecard::GameCardMessage;
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
            GameCardMessage::GamePressed(_) => {
                // Now intercepted at the page level
            }
            GameCardMessage::Tick => {
                // No more drawer animation logic needed here
            }
            GameCardMessage::CloseDrawer => {
                // No longer used
            }
            GameCardMessage::OpenStorePage(url) => {
                #[cfg(target_os = "linux")]
                {
                    std::process::Command::new("xdg-open")
                        .arg(url)
                        .spawn()
                        .unwrap();
                }
                #[cfg(target_os = "windows")]
                {
                    std::process::Command::new("cmd.exe")
                        .arg("/C")
                        .arg(format!("start {}", url))
                        .spawn()
                        .unwrap();
                }
                return iced::Task::none();
            }
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
            GameCardMessage::UpdateGames(_) => {
                // Handled in container update, but we might want to reset selection?
            }
            _ => {}
        }

        self.games.update(msg)
    }

    fn view_grid(&self, interactive: bool) -> Element<'_, GameCardMessage> {
        if self.games.is_empty() {
            container(
                text("No games found")
                    .size(32)
                    .style(|_theme: &iced::Theme| text::Style {
                        color: Some(Color::from_rgb8(100, 100, 100)),
                    }),
            )
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
