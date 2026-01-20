use iced::widget::pick_list;
use iced::{overlay, Border, Color, Theme};

pub fn default(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let palette = theme.palette();

    let active_style = pick_list::Style {
        text_color: palette.text,
        placeholder_color: Color::from_rgb8(150, 150, 150),
        handle_color: palette.text,
        background: Color::from_rgb8(30, 30, 35).into(),
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: Color::from_rgb8(60, 60, 60),
        },
    };

    match status {
        pick_list::Status::Active => active_style,
        pick_list::Status::Hovered => pick_list::Style {
            background: Color::from_rgb8(40, 40, 45).into(),
            ..active_style
        },
        pick_list::Status::Opened { .. } => pick_list::Style {
            background: Color::from_rgb8(40, 40, 45).into(),
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: palette.primary,
            },
            ..active_style
        },
    }
}

pub fn menu(theme: &Theme) -> overlay::menu::Style {
    let palette = theme.palette();

    overlay::menu::Style {
        text_color: palette.text,
        background: Color::from_rgb8(30, 30, 35).into(),
        border: Border {
            width: 1.0,
            color: Color::from_rgb8(60, 60, 60),
            radius: 4.0.into(),
        },
        selected_text_color: Color::WHITE,
        selected_background: palette.primary.into(),
        shadow: Default::default(),
    }
}
