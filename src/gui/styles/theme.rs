use iced::theme::{Custom, Palette};
use iced::{Color, Theme};
use std::sync::Arc;

pub fn gaming() -> Theme {
    Theme::Custom(Arc::new(Custom::new(
        "Monarch".to_string(),
        Palette {
            background: Color::from_rgb8(30, 33, 43), // Dark grey/blue
            text: Color::from_rgb8(240, 240, 240),
            primary: Color::from_rgb8(255, 140, 0), // Dark Orange
            success: Color::from_rgb8(50, 205, 50),
            warning: Color::from_rgb8(255, 204, 0),
            danger: Color::from_rgb8(220, 20, 60),
        },
    )))
}
