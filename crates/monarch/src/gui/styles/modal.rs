use iced::border;
use iced::widget::container;
use iced::{Color, Theme};

use super::radius;

pub fn overlay(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgba8(0, 0, 0, 0.8).into()),
        ..Default::default()
    }
}

pub fn content(theme: &Theme) -> container::Style {
    let palette = theme.palette();

    container::Style {
        background: Some(Color::from_rgb8(20, 20, 30).into()),
        text_color: Some(palette.text),
        border: border::Border {
            color: palette.primary,
            width: 1.0,
            radius: radius::SUBTLE.into(),
        },
        ..Default::default()
    }
}
