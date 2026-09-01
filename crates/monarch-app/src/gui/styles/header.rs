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

pub fn window_control(theme: &Theme, status: widget_button::Status) -> widget_button::Style {
    let palette = theme.palette();

    match status {
        widget_button::Status::Active => widget_button::Style {
            background: None,
            text_color: Color {
                a: 0.5,
                ..palette.text
            },
            ..Default::default()
        },
        widget_button::Status::Hovered => widget_button::Style {
            background: Some(Color::from_rgba8(60, 60, 75, 0.9).into()),
            text_color: palette.text,
            ..Default::default()
        },
        widget_button::Status::Pressed => widget_button::Style {
            background: Some(Color::from_rgba8(40, 40, 55, 1.0).into()),
            text_color: palette.text,
            ..Default::default()
        },
        _ => widget_button::Style::default(),
    }
}

pub fn window_control_close(theme: &Theme, status: widget_button::Status) -> widget_button::Style {
    let palette = theme.palette();

    match status {
        widget_button::Status::Active => widget_button::Style {
            background: None,
            text_color: Color {
                a: 0.5,
                ..palette.text
            },
            ..Default::default()
        },
        widget_button::Status::Hovered => widget_button::Style {
            background: Some(palette.danger.into()),
            text_color: Color::WHITE,
            ..Default::default()
        },
        widget_button::Status::Pressed => widget_button::Style {
            background: Some(Color {
                a: 0.8,
                ..palette.danger
            }
            .into()),
            text_color: Color::WHITE,
            ..Default::default()
        },
        _ => widget_button::Style::default(),
    }
}

pub fn drag(_theme: &Theme, status: widget_button::Status) -> widget_button::Style {
    match status {
        widget_button::Status::Active => widget_button::Style {
            background: None,
            text_color: Color::TRANSPARENT,
            border: Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        },
        widget_button::Status::Hovered => widget_button::Style {
            background: None,
            text_color: Color::TRANSPARENT,
            border: Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        },
        widget_button::Status::Pressed => widget_button::Style {
            background: None,
            text_color: Color::TRANSPARENT,
            border: Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        },
        _ => widget_button::Style::default(),
    }
}
