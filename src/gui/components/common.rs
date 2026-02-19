use crate::gui::resources::{PLAY, REFRESH};
use crate::gui::styles;
use iced::widget::{button, pick_list, row, svg, text_editor, text_input, Text};
use iced::{alignment, Color, Element, Length, Theme};
use std::borrow::Cow;

pub fn primary_button<'a, Message>(
    label: &str,
    on_press: Option<Message>,
) -> Element<'a, Message, Theme>
where
    Message: Clone + 'a,
{
    // Text alignment in Iced 0.14 uses align_x
    button(Text::new(label.to_owned()).align_x(alignment::Horizontal::Center))
        .on_press_maybe(on_press)
        .style(styles::button::primary)
        .padding(10)
        .into()
}

pub fn scanner_button<'a, Message>(
    label: &str,
    on_press: Option<Message>,
    is_hovered: bool,
) -> Element<'a, Message, Theme>
where
    Message: Clone + 'a,
{
    button(
        row![
            svg(REFRESH.clone())
                .width(20)
                .height(20)
                .style(move |_theme: &Theme, _status| {
                    iced::widget::svg::Style {
                        color: Some(if is_hovered {
                            Color::BLACK
                        } else {
                            Color::from_rgb8(255, 127, 0)
                        }),
                    }
                }),
            Text::new(label.to_owned())
                .align_x(alignment::Horizontal::Center)
                .size(16)
        ]
        .spacing(10)
        .align_y(alignment::Vertical::Center),
    )
    .on_press_maybe(on_press)
    .style(styles::button::scanner)
    .padding(12)
    .into()
}

pub fn secondary_button<'a, Message>(
    label: &str,
    on_press: Option<Message>,
) -> Element<'a, Message, Theme>
where
    Message: Clone + 'a,
{
    button(Text::new(label.to_owned()).align_x(alignment::Horizontal::Center))
        .on_press_maybe(on_press)
        .style(styles::button::secondary)
        .padding(10)
        .into()
}

pub fn _text_button<'a, Message>(
    label: &str,
    on_press: Option<Message>,
) -> Element<'a, Message, Theme>
where
    Message: Clone + 'a,
{
    button(Text::new(label.to_owned()).align_x(alignment::Horizontal::Center))
        .on_press_maybe(on_press)
        .style(styles::button::text)
        .padding(5)
        .into()
}

pub fn launch_button<'a, Message>(
    label: &str,
    on_press: Option<Message>,
) -> Element<'a, Message, Theme>
where
    Message: Clone + 'a,
{
    // Text alignment in Iced 0.14 uses align_x
    button(
        row![
            svg(PLAY.clone())
                .width(35)
                .height(35)
                .style(move |_theme: &Theme, _status| {
                    iced::widget::svg::Style {
                        color: Some(Color::WHITE),
                    }
                }),
            Text::new(label.to_owned())
                .align_x(alignment::Horizontal::Center)
                .size(30)
        ]
        .spacing(20)
        .align_y(alignment::Vertical::Center),
    )
    .on_press_maybe(on_press)
    .style(styles::button::primary)
    .padding(10)
    .width(300)
    .height(60)
    .into()
}

pub fn input_field<'a, Message>(
    placeholder: &str,
    value: &str,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message, Theme>
where
    Message: Clone + 'a,
{
    text_input(placeholder, value)
        .on_input(on_change)
        .style(styles::text_input::default)
        .padding(10)
        .width(Length::Fill)
        .into()
}

pub fn _text_area<'a, Message>(
    content: &'a text_editor::Content,
    on_action: impl Fn(text_editor::Action) -> Message + 'a,
) -> Element<'a, Message, Theme>
where
    Message: Clone + 'a,
{
    text_editor(content)
        .on_action(on_action)
        .style(styles::editor::default)
        .padding(10)
        .height(Length::Fixed(150.0)) // Default height
        .into()
}

pub fn _dropdown<'a, Message, T>(
    options: impl Into<Cow<'a, [T]>>,
    selected: Option<T>,
    on_selected: impl Fn(T) -> Message + 'a,
) -> Element<'a, Message, Theme>
where
    T: ToString + Eq + Clone + 'static,
    Message: Clone + 'a,
    [T]: ToOwned<Owned = Vec<T>>,
{
    let options: Cow<'a, [T]> = options.into();
    pick_list(options, selected, on_selected)
        .style(styles::pick_list::default)
        .padding(10)
        .width(Length::Fill)
        .into()
}

pub fn error_view<'a, Message>(
    title: &'a str,
    message: &'a str,
    retry_msg: Option<Message>,
) -> Element<'a, Message, Theme>
where
    Message: Clone + 'a,
{
    use iced::widget::{column, container, text, Space};

    container(
        column![
            text(title).size(32).color(Color::from_rgb(0.9, 0.3, 0.3)), // Soft red
            Space::new().height(15),
            text(message)
                .size(18)
                .color(Color::from_rgb(0.6, 0.6, 0.6))
                .align_x(alignment::Horizontal::Center),
            Space::new().height(30),
            if let Some(msg) = retry_msg {
                primary_button("Try Again", Some(msg))
            } else {
                Element::from(Space::new())
            }
        ]
        .spacing(10)
        .align_x(alignment::Horizontal::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .padding(40)
    .into()
}
