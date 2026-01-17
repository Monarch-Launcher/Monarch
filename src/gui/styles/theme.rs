use iced::theme::{Custom, Palette};
use iced::{Color, Theme};
use std::sync::Arc;

pub fn monarch() -> Theme {
    Theme::Custom(Arc::new(Custom::new(
        "Monarch".to_string(),
        Palette {
            background: Color::from_rgb8(10, 10, 10), // Almost black
            text: Color::from_rgb8(240, 240, 240),
            primary: Color::from_rgb8(255, 127, 0), // Vivid Orange
            success: Color::from_rgb8(50, 205, 50),
            warning: Color::from_rgb8(255, 204, 0),
            danger: Color::from_rgb8(220, 20, 60),
        },
    )))
}
