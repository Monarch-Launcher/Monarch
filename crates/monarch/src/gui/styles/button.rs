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

pub fn secondary(_theme: &Theme, status: button::Status) -> button::Style {
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

pub fn scanner(_theme: &Theme, status: button::Status) -> button::Style {
    let accent = Color::from_rgb8(255, 127, 0); // Monarch Orange

    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(Color::from_rgba8(30, 30, 35, 1.0))),
            text_color: accent,
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: Color { a: 0.5, ..accent },
            },
            ..button::Style::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(accent)),
            text_color: Color::BLACK,
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: accent,
            },
            ..button::Style::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color { a: 0.8, ..accent })),
            text_color: Color::BLACK,
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: accent,
            },
            ..button::Style::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgba8(30, 30, 35, 0.5))),
            text_color: Color { a: 0.3, ..accent },
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: Color { a: 0.2, ..accent },
            },
            ..button::Style::default()
        },
    }
}

pub fn transparent(_theme: &Theme, status: button::Status) -> button::Style {
    let accent = Color::from_rgb8(255, 127, 0); // Monarch Orange

    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(Color::from_rgba8(0, 0, 0, 0.0))),
            text_color: accent,
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                },
            },
            ..button::Style::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color { a: 0.2, ..accent })),
            text_color: Color::BLACK,
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                },
            },
            ..button::Style::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color { a: 0.2, ..accent })),
            text_color: Color::BLACK,
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                },
            },
            ..button::Style::default()
        },
        button::Status::Disabled => button::Style {
            ..button::Style::default()
        },
    }
}
