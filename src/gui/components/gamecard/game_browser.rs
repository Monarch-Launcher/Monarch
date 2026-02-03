use crate::gui::components::gamecard::container::GameCardContainer;
use crate::gui::components::gamecard::drawer::GameDrawer;
use crate::gui::components::gamecard::GameCardMessage;
use crate::monarch_games::monarchgame::{MonarchGame, MonarchWebApiPlatform};
use iced::widget::{container, stack, text};
use iced::{alignment, Color, Element, Length};
use tracing::info;

#[derive(Default)]
pub struct GameBrowser {
    pub games: GameCardContainer,
    drawer: GameDrawer,
    selected_game: Option<MonarchGame>,
    drawer_animation: f32,
}

impl GameBrowser {
    pub fn update(&mut self, msg: GameCardMessage) -> iced::Task<GameCardMessage> {
        match &msg {
            GameCardMessage::GamePressed(id) => {
                if let Some(game_card) = self.games.games.iter().find(|g| g.game.id == *id) {
                    self.selected_game = Some(game_card.game.clone());
                    // Start animation from current state (usually 0.0 if closed)
                    self.drawer_animation = 0.0;
                }
            }
            GameCardMessage::Tick => {
                // Simple slide-in for the main panel if selected
                if self.selected_game.is_some() {
                    let target = 1.0;
                    if (self.drawer_animation - target).abs() > 0.001 {
                        self.drawer_animation += (target - self.drawer_animation) * 0.2;
                    } else {
                        self.drawer_animation = target;
                    }
                } else {
                    let target = 0.0;
                    if (self.drawer_animation - target).abs() > 0.001 {
                        self.drawer_animation += (target - self.drawer_animation) * 0.2;
                    } else {
                        self.drawer_animation = target;
                    }
                }
            }
            GameCardMessage::CloseDrawer => {
                self.selected_game = None;
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
        if let Some(game) = &self.selected_game {
            iced::widget::responsive(move |size| {
                let drawer_width = size.width * 0.5;
                let padding_left = size.width - (drawer_width * self.drawer_animation);
                let platforms: Vec<MonarchWebApiPlatform> = game
                    .stores
                    .iter()
                    .cloned()
                    .map(MonarchWebApiPlatform::from)
                    .collect();

                let drawer_layer = container(self.drawer.view(game, platforms))
                    .width(Length::Fixed(drawer_width))
                    .height(Length::Fill);

                let drawer_wrapper = container(drawer_layer)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(iced::Padding::default().left(padding_left))
                    .align_x(alignment::Horizontal::Left);

                stack![
                    self.view_grid(false),
                    container(
                        iced::widget::mouse_area(
                            container(
                                iced::widget::Space::new()
                                    .width(Length::Fill)
                                    .height(Length::Fill)
                            )
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .style(move |_theme: &iced::Theme| container::Style {
                                background: Some(
                                    Color::from_rgba8(0, 0, 0, 0.5 * self.drawer_animation).into()
                                ),
                                ..Default::default()
                            }),
                        )
                        .on_press(GameCardMessage::CloseDrawer),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill),
                    drawer_wrapper
                ]
                .into()
            })
            .into()
        } else {
            self.view_grid(true)
        }
    }
}
