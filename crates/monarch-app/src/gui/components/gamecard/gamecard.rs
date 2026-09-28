use std::sync::{Arc, RwLock};

use crate::gui::components::gamecard::GameCardMessage;
use iced::widget::{button, container, image, mouse_area};
use iced::{alignment, Color, Element, Length};
use monarch_core::monarch_games::monarchgame::MonarchGame;

#[derive(Debug, Clone)]
pub struct GameCard {
    pub game: Arc<RwLock<MonarchGame>>,
    hover: bool,
    hover_factor: f32,
}

impl GameCard {
    pub fn new(game: Arc<RwLock<MonarchGame>>) -> Self {
        Self {
            game,
            hover: false,
            hover_factor: 0.0,
        }
    }

    pub fn update_game(&mut self, game: Arc<RwLock<MonarchGame>>) {
        self.game = game;
    }
}

impl GameCard {
    pub fn update(&mut self, msg: GameCardMessage) -> iced::Task<GameCardMessage> {
        match msg {
            GameCardMessage::GameHovered(id) => {
                if let Ok(game) = self.game.read() {
                    if game.id == id {
                        self.hover = true;
                    }
                }
                iced::Task::none()
            }
            GameCardMessage::GameUnhovered(id) => {
                if let Ok(game) = self.game.read() {
                    if game.id == id {
                        self.hover = false;
                    }
                }
                iced::Task::none()
            }
            GameCardMessage::Tick => {
                let speed = 0.3; // Animation speed (lerp factor)

                let target = if self.hover { 1.0 } else { 0.0 };
                let current = self.hover_factor;

                if (current - target).abs() > 0.001 {
                    let new_val = current + (target - current) * speed;
                    self.hover_factor = new_val;
                }

                iced::Task::none()
            }
            GameCardMessage::UpdateGames(_games) => iced::Task::none(),
            _ => iced::Task::none(),
        }
    }

    pub fn view(&self, interactive: bool) -> Element<'_, GameCardMessage> {
        self.view_scaled(interactive, 1.0)
    }

    pub fn view_scaled(&self, interactive: bool, size_scale: f32) -> Element<'_, GameCardMessage> {
        let (base_width, base_height) = (240.0 * size_scale, 360.0 * size_scale);
        let scale = 1.0 + (self.hover_factor * 0.05);
        let (width, height) = (base_width * scale, base_height * scale);

        let mut cover_exists: bool = true;
        if let Ok(game) = self.game.read() {
            cover_exists =
                !game.cover_path.is_empty() && std::path::Path::new(&game.cover_path).exists();
        }

        let image_widget: Element<'_, GameCardMessage> = if !cover_exists {
            self.draw_fallback_cover(width, height)
        } else {
            match self.game.read() {
                Ok(game) => self.draw_game_cover(&game, width, height),
                Err(_) => self.draw_fallback_cover(width, height),
            }
        };

        let card_button = button(image_widget)
            .on_press_maybe(if interactive {
                match self.game.read() {
                    Ok(game) => Some(GameCardMessage::GamePressed(game.id.clone())),
                    Err(_) => None,
                }
            } else {
                None
            })
            .padding(0)
            .style(|_theme: &iced::Theme, _status| button::Style {
                background: None,
                border: iced::Border {
                    width: 0.0,
                    radius: 0.0.into(),
                    color: Color::TRANSPARENT,
                },
                ..Default::default()
            });

        let mut area = mouse_area(
            container(card_button)
                .padding(10.0 * (1.0 - self.hover_factor))
                .width(base_width + 20.0)
                .height(base_height + 20.0)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center),
        );

        if interactive {
            if let Ok(game) = self.game.read() {
                area = area
                    .on_enter(GameCardMessage::GameHovered(game.id.clone()))
                    .on_exit(GameCardMessage::GameUnhovered(game.id.clone()));
            }
        }

        area.into()
    }

    fn draw_fallback_cover(&self, w: f32, h: f32) -> Element<'_, GameCardMessage> {
        container(
            image(crate::gui::resources::LOGO_LARGE.clone())
                .width(Length::Fixed(w))
                .height(Length::Fixed(h))
                .content_fit(iced::ContentFit::Cover),
        )
        .clip(true)
        .style(move |_theme: &iced::Theme| container::Style {
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: if self.hover { 2.0 } else { 0.0 },
                radius: 0.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba8(0, 0, 0, 0.6 * self.hover_factor),
                offset: iced::Vector::new(0.0, 10.0 * self.hover_factor),
                blur_radius: 20.0 * self.hover_factor,
            },
            ..Default::default()
        })
        .into()
    }

    fn draw_game_cover(&self, game: &MonarchGame, w: f32, h: f32) -> Element<'_, GameCardMessage> {
        let handle = if !game.is_installed {
            let grey = game.greyscale_path();
            if !grey.is_empty() && std::path::Path::new(&grey).exists() {
                iced::widget::image::Handle::from_path(grey)
            } else {
                iced::widget::image::Handle::from_path(game.cover_path.clone())
            }
        } else {
            iced::widget::image::Handle::from_path(game.cover_path.clone())
        };

        container(
            image(handle)
                .width(Length::Fixed(w))
                .height(Length::Fixed(h))
                .content_fit(iced::ContentFit::Cover),
        )
        .clip(true)
        .style(move |_theme: &iced::Theme| container::Style {
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: if self.hover { 2.0 } else { 0.0 },
                radius: 0.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba8(0, 0, 0, 0.6 * self.hover_factor),
                offset: iced::Vector::new(0.0, 10.0 * self.hover_factor),
                blur_radius: 20.0 * self.hover_factor,
            },
            ..Default::default()
        })
        .into()
    }
}
