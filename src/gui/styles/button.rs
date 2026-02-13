use iced::widget::button;
use iced::{Background, Border, Color, Theme};

pub fn primary(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.palette();

    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(palette.primary)),
            text_color: Color::WHITE,
            border: Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Default::default(),
            snap: false,
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color {
                a: 0.9,
                ..palette.primary
            })),
            text_color: Color::WHITE,
            border: Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Default::default(),
            snap: false,
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color {
                a: 0.8,
                ..palette.primary
            })),
            text_color: Color::WHITE,
            border: Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Default::default(),
            snap: false,
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color {
                a: 0.3,
                ..palette.primary
            })),
            text_color: Color {
                a: 0.3,
                ..Color::WHITE
            },
            border: Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Default::default(),
            snap: false,
        },
    }
}

pub fn secondary(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.palette();

    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(Color::from_rgba8(40, 40, 50, 0.85))),
            text_color: Color::WHITE,
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: Color::from_rgba8(255, 255, 255, 0.2),
            },
            shadow: Default::default(),
            snap: false,
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgba8(60, 60, 75, 0.95))),
            text_color: Color::WHITE,
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: Color::from_rgba8(255, 255, 255, 0.3),
            },
            shadow: Default::default(),
            snap: false,
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgba8(30, 30, 40, 1.0))),
            text_color: Color::WHITE,
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: Color::from_rgba8(255, 255, 255, 0.4),
            },
            shadow: Default::default(),
            snap: false,
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgba8(40, 40, 50, 0.4))),
            text_color: Color {
                a: 0.3,
                ..Color::WHITE
            },
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: Color::from_rgba8(255, 255, 255, 0.1),
            },
            shadow: Default::default(),
            snap: false,
        },
    }
}

pub fn destructive(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.palette();

    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(palette.danger)),
            text_color: Color::WHITE,
            border: Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Default::default(),
            snap: false,
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color {
                a: 0.8,
                ..palette.danger
            })),
            text_color: Color::WHITE,
            border: Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Default::default(),
            snap: false,
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color {
                a: 0.6,
                ..palette.danger
            })),
            text_color: Color::WHITE,
            border: Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Default::default(),
            snap: false,
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color {
                a: 0.3,
                ..palette.danger
            })),
            text_color: Color {
                a: 0.3,
                ..Color::WHITE
            },
            border: Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Default::default(),
            snap: false,
        },
    }
}

pub fn text(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.palette();

    match status {
        button::Status::Active => button::Style {
            background: None,
            text_color: palette.text,
            border: Border::default(),
            shadow: Default::default(),
            snap: false,
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color {
                a: 0.1,
                ..palette.text
            })),
            text_color: palette.text,
            border: Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Default::default(),
            snap: false,
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color {
                a: 0.2,
                ..palette.text
            })),
            text_color: palette.text,
            border: Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Default::default(),
            snap: false,
        },
        button::Status::Disabled => button::Style {
            background: None,
            text_color: Color {
                a: 0.3,
                ..palette.text
            },
            border: Border::default(),
            shadow: Default::default(),
            snap: false,
        },
    }
}
