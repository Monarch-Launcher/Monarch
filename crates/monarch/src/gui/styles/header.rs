use iced::widget::{button as widget_button, container as widget_container};
use iced::{Border, Color, Theme, Vector};

use super::radius;

pub fn container(theme: &Theme) -> widget_container::Style {
    let palette = theme.palette();

    widget_container::Style {
        background: Some(palette.background.into()),
        border: Border {
            width: 0.0,
            radius: 0.0.into(),
            color: Color::TRANSPARENT,
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            offset: Vector::new(0.0, -2.0),
            blur_radius: 5.0,
        },
        ..Default::default()
    }
}

pub fn button(theme: &Theme, status: widget_button::Status) -> widget_button::Style {
    let palette = theme.palette();

    match status {
        widget_button::Status::Active => widget_button::Style {
            background: None,
            text_color: palette.text,
            ..Default::default()
        },
        widget_button::Status::Hovered => widget_button::Style {
            background: None,
            text_color: palette.text,
            border: Border {
                radius: radius::SUBTLE.into(),
                ..Default::default()
            },
            ..Default::default()
        },
        widget_button::Status::Pressed => widget_button::Style {
            background: None,
            text_color: palette.text,
            border: Border {
                radius: radius::SUBTLE.into(),
                ..Default::default()
            },
            ..Default::default()
        },
        _ => widget_button::Style::default(),
    }
}

pub fn active_button(theme: &Theme, status: widget_button::Status) -> widget_button::Style {
    let palette = theme.palette();

    match status {
        widget_button::Status::Active => widget_button::Style {
            background: None,
            text_color: palette.primary,
            ..Default::default()
        },
        widget_button::Status::Hovered => widget_button::Style {
            background: None,
            text_color: palette.primary,
            border: Border {
                radius: radius::SUBTLE.into(),
                ..Default::default()
            },
            ..Default::default()
        },
        widget_button::Status::Pressed => widget_button::Style {
            background: None,
            text_color: palette.primary,
            border: Border {
                radius: radius::SUBTLE.into(),
                ..Default::default()
            },
            ..Default::default()
        },
        _ => widget_button::Style::default(),
    }
}
