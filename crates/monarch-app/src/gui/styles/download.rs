use iced::widget::{button, container};
use iced::{Background, Border, Color, Theme};

use super::radius;

pub fn queue_panel(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(16, 16, 24))),
        border: Border {
            width: 1.0,
            color: Color::from_rgba8(255, 255, 255, 0.06),
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn queue_item_active(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Color::from_rgba8(255, 127, 0, 0.18),
        button::Status::Pressed => Color::from_rgba8(255, 127, 0, 0.25),
        _ => Color::from_rgba8(255, 127, 0, 0.12),
    };

    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color::WHITE,
        border: Border {
            radius: radius::SUBTLE.into(),
            width: 1.0,
            color: Color::from_rgba8(255, 127, 0, 0.55),
        },
        shadow: Default::default(),
        snap: false,
    }
}

pub fn queue_item(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Color::from_rgba8(255, 255, 255, 0.06),
        button::Status::Pressed => Color::from_rgba8(255, 255, 255, 0.08),
        _ => Color::from_rgba8(255, 255, 255, 0.03),
    };

    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color::WHITE,
        border: Border {
            radius: radius::SUBTLE.into(),
            width: 1.0,
            color: Color::from_rgba8(255, 255, 255, 0.06),
        },
        shadow: Default::default(),
        snap: false,
    }
}

pub fn stat_card(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba8(22, 22, 32, 0.92))),
        border: Border {
            radius: radius::SUBTLE.into(),
            color: Color::from_rgba8(255, 255, 255, 0.14),
            width: 1.0,
        },
        ..Default::default()
    }
}

pub fn graph_panel(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba8(22, 22, 32, 0.92))),
        border: Border {
            radius: radius::SUBTLE.into(),
            color: Color::from_rgba8(255, 255, 255, 0.12),
            width: 1.0,
        },
        ..Default::default()
    }
}

pub fn body_panel(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(10, 10, 17))),
        ..Default::default()
    }
}

pub fn progress_track(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba8(255, 255, 255, 0.08))),
        border: Border {
            radius: radius::SUBTLE.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn progress_fill(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(255, 127, 0))),
        border: Border {
            radius: radius::SUBTLE.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn speed_widget(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, border) = match status {
        button::Status::Hovered => (
            Color::from_rgba8(255, 127, 0, 0.2),
            Color::from_rgba8(255, 127, 0, 0.7),
        ),
        button::Status::Pressed => (
            Color::from_rgba8(255, 127, 0, 0.28),
            Color::from_rgb8(255, 127, 0),
        ),
        _ => (
            Color::from_rgba8(255, 127, 0, 0.12),
            Color::from_rgba8(255, 127, 0, 0.45),
        ),
    };

    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color::from_rgb8(255, 127, 0),
        border: Border {
            radius: radius::SUBTLE.into(),
            width: 1.0,
            color: border,
        },
        shadow: Default::default(),
        snap: false,
    }
}
