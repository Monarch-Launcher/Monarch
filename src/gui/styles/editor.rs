use iced::widget::text_editor;
use iced::{Border, Color, Theme};

pub fn default(theme: &Theme, status: text_editor::Status) -> text_editor::Style {
    let palette = theme.palette();

    match status {
        text_editor::Status::Active
        | text_editor::Status::Focused { .. }
        | text_editor::Status::Hovered => text_editor::Style {
            background: Color::from_rgb8(30, 30, 35).into(),
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: Color::from_rgb8(60, 60, 60),
            },
            placeholder: Color::from_rgb8(150, 150, 150),
            value: palette.text,
            selection: Color::from_rgb8(80, 80, 80),
        },
        text_editor::Status::Disabled => text_editor::Style {
            background: Color::from_rgb8(20, 20, 25).into(),
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: Color::from_rgb8(40, 40, 40),
            },
            placeholder: Color::from_rgb8(100, 100, 100),
            value: Color::from_rgb8(100, 100, 100),
            selection: Color::from_rgb8(50, 50, 50),
        },
    }
}
