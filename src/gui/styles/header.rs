use iced::widget::{button as widget_button, container as widget_container};
use iced::{Border, Color, Theme, Vector};

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
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
            offset: Vector::new(0.0, 2.0),
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
            background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.1).into()),
            text_color: palette.text,
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        },
        widget_button::Status::Pressed => widget_button::Style {
            background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.05).into()),
            text_color: palette.text,
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        },
        _ => widget_button::Style::default(),
    }
}
