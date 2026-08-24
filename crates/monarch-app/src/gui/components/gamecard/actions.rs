use std::sync::{Arc, Mutex};

use iced::widget::{button, column, row, svg, text, Space};
use iced::{alignment, Color, Element, Length, Task, Theme};
use tracing::error;

use crate::gui::components::common::secondary_button;
use crate::gui::{resources, show_error, styles};
use monarch_core::monarch_games;
use monarch_core::monarch_games::games::GameType;
use monarch_core::monarch_games::monarchgame::MonarchGame;

#[derive(Clone, Debug)]
pub enum Message {
    Launch,
    VerifyIntegrity,
    CheckForUpdates,
    OpenInstallLocation,
    MoveInstall,
    ToggleFavorite,
    CreateDesktopShortcut,
    ViewOnStore,
    Uninstall,
    Close,
}

#[derive(Debug, Clone)]
pub struct ActionsModal {
    game: Arc<Mutex<MonarchGame>>,
}

impl ActionsModal {
    pub fn new(game: Arc<Mutex<MonarchGame>>) -> (Self, Task<Message>) {
        let modal = Self { game: game.clone() };

        (modal, iced::Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Uninstall => match self.game.lock() {
                Ok(game) => {
                    if &game.get_store_name() == "monarch" {
                        return iced::Task::future(monarch_games::commands::manual_remove_game(
                            game.clone(),
                        ))
                        .then(|out| {
                            if let Err(e) = out {
                                show_error(e);
                            }
                            iced::Task::none()
                        });
                    } else {
                        return iced::Task::future(monarch_games::commands::remove_game(
                            game.name.clone(),
                            game.get_store_name(),
                            game.get_store_id(),
                        ))
                        .then(|out| {
                            if let Err(e) = out {
                                show_error(e);
                            }
                            iced::Task::<Message>::none()
                        });
                    }
                }
                Err(e) => {
                    error!("actions_modal::update() Failed to lock on self.game! | Err: {e}");
                    show_error("Failed to detect game to remove!");
                }
            },
            // Placeholder actions, implemented purely as UI for now.
            Message::Launch
            | Message::VerifyIntegrity
            | Message::CheckForUpdates
            | Message::OpenInstallLocation
            | Message::MoveInstall
            | Message::ToggleFavorite
            | Message::CreateDesktopShortcut
            | Message::ViewOnStore => {}
            Message::Close => {}
        }
        iced::Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let game = match self.game.lock() {
            Ok(game) => game.clone(),
            Err(e) => {
                error!("actions_modal::view() Failed to lock on self.game! | Err: {e}");
                show_error("Failed to open actions for selected game!");

                let content = column![];
                return crate::gui::components::modal::Modal::new("Actions", content)
                    .width(Length::Fixed(800.0))
                    .view();
            }
        };

        let store_name = game.get_store_name();

        let remove_label = if store_name == "monarch" {
            "Remove from Monarch"
        } else {
            "Uninstall Game"
        };

        let content = column![
            section_header("General"),
            action_item(
                resources::PLAY.clone(),
                "Launch Game",
                Some(Message::Launch)
            ),
            Space::new().height(Length::Fixed(8.0)),
            section_header("Maintenance"),
            action_item(
                resources::REFRESH.clone(),
                "Verify Integrity of Files",
                Some(Message::VerifyIntegrity),
            ),
            action_item(
                resources::UPDATE.clone(),
                "Check for Updates",
                Some(Message::CheckForUpdates),
            ),
            Space::new().height(Length::Fixed(8.0)),
            section_header("Files"),
            action_item(
                resources::FOLDER.clone(),
                "Open Install Location",
                Some(Message::OpenInstallLocation),
            ),
            action_item(
                resources::ADD_FOLDER.clone(),
                "Move Install Folder",
                Some(Message::MoveInstall),
            ),
            Space::new().height(Length::Fixed(8.0)),
            section_header("Extras"),
            action_item(
                resources::FAVORITE_OUTLINE.clone(),
                "Add to Favorites",
                Some(Message::ToggleFavorite),
            ),
            action_item(
                resources::VIEW.clone(),
                "Create Desktop Shortcut",
                Some(Message::CreateDesktopShortcut),
            ),
            action_item(
                store_icon(&store_name),
                "View on Store",
                Some(Message::ViewOnStore),
            ),
            Space::new().height(Length::Fixed(8.0)),
            danger_action_item(
                resources::TRASH.clone(),
                remove_label,
                Some(Message::Uninstall)
            ),
            Space::new().height(Length::Fixed(20.0)),
            row![secondary_button("Done", Some(Message::Close))]
                .align_y(alignment::Vertical::Center)
        ]
        .spacing(10);

        crate::gui::components::modal::Modal::new("Actions", content)
            .width(Length::Fixed(800.0))
            .on_close(Message::Close)
            .view()
    }
}

fn store_icon(store: &str) -> svg::Handle {
    match store {
        "steam" | "steamcmd" => resources::STEAM.clone(),
        "epicgames" => resources::EPIC.clone(),
        "gog" => resources::GOG.clone(),
        "itch" => resources::ITCH.clone(),
        _ => resources::MONARCH.clone(),
    }
}

fn section_header(label: &str) -> Element<'_, Message> {
    text(label)
        .size(13)
        .color(Color::from_rgb8(140, 140, 140))
        .into()
}

fn action_item(icon: svg::Handle, label: &str, on_press: Option<Message>) -> Element<'_, Message> {
    button(
        row![
            svg(icon)
                .width(18)
                .height(18)
                .style(|_theme: &Theme, _status| svg::Style {
                    color: Some(Color::from_rgb8(220, 220, 220)),
                }),
            text(label).size(15),
            Space::new().width(Length::Fill),
        ]
        .spacing(12)
        .align_y(alignment::Vertical::Center),
    )
    .on_press_maybe(on_press)
    .style(styles::button::secondary)
    .width(Length::Fill)
    .padding(12)
    .into()
}

fn danger_action_item(
    icon: svg::Handle,
    label: &str,
    on_press: Option<Message>,
) -> Element<'_, Message> {
    button(
        row![
            svg(icon)
                .width(18)
                .height(18)
                .style(|_theme: &Theme, _status| svg::Style {
                    color: Some(Color::WHITE),
                }),
            text(label).size(15),
            Space::new().width(Length::Fill),
        ]
        .spacing(12)
        .align_y(alignment::Vertical::Center),
    )
    .on_press_maybe(on_press)
    .style(styles::button::destructive)
    .width(Length::Fill)
    .padding(12)
    .into()
}
