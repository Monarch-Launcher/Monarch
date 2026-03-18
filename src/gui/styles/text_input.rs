use iced::widget::text_input;
use iced::{Border, Color, Theme};

pub fn search(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let palette = theme.palette();

    match status {
        text_input::Status::Active
        | text_input::Status::Focused { .. }
        | text_input::Status::Hovered => text_input::Style {
            background: Color::from_rgb8(25, 25, 25).into(),
            border: Border {
                radius: 10.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            icon: palette.text,
            placeholder: Color::from_rgb8(100, 100, 100),
            value: palette.text,
            selection: Color::from_rgb8(50, 50, 50),
        },
        text_input::Status::Disabled => text_input::Style {
            background: Color::from_rgb8(25, 25, 25).into(),
            border: Border {
                radius: 10.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            icon: Color::from_rgb8(100, 100, 100),
            placeholder: Color::from_rgb8(100, 100, 100),
            value: Color::from_rgb8(100, 100, 100),
            selection: Color::from_rgb8(50, 50, 50),
        },
    }
}

pub fn default(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let palette = theme.palette();

    match status {
        text_input::Status::Active
        | text_input::Status::Focused { .. }
        | text_input::Status::Hovered => text_input::Style {
            background: Color::from_rgb8(30, 30, 35).into(),
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: Color::from_rgb8(60, 60, 60),
            },
            icon: palette.text,
            placeholder: Color::from_rgb8(150, 150, 150),
            value: palette.text,
            selection: Color::from_rgb8(80, 80, 80),
        },
        text_input::Status::Disabled => text_input::Style {
            background: Color::from_rgb8(20, 20, 25).into(),
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: Color::from_rgb8(40, 40, 40),
            },
            icon: Color::from_rgb8(100, 100, 100),
            placeholder: Color::from_rgb8(100, 100, 100),
            value: Color::from_rgb8(100, 100, 100),
            selection: Color::from_rgb8(50, 50, 50),
        },
    }
}
