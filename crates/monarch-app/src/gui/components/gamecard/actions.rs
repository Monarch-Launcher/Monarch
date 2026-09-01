use std::sync::{Arc, Mutex};

use iced::widget::{button, column, row, svg, text, Space};
use iced::{alignment, Color, Element, Length, Task, Theme};
use tracing::error;

use crate::gui::components::common::secondary_button;
use crate::gui::pages;
use crate::gui::{AppMessage, GUI_SENDER, resources, show_error, styles};
use monarch_core::monarch_games;
use monarch_core::monarch_games::games::GameType;
use monarch_core::monarch_games::integrity;
use monarch_core::monarch_games::monarchgame::MonarchGame;
use monarch_core::monarch_games::updates::GameUpdateCheck;

// Actions not yet implemented are rendered disabled (greyed out) in the view
// instead of being wired to dead handlers.
#[derive(Clone, Debug)]
pub enum Message {
    Uninstall,
    /// Result of an uninstall attempt; `Ok` carries the removed game id.
    Uninstalled(Result<String, String>),
    Close,
    CheckForUpdates,
    UpdatesChecked(Result<GameUpdateCheck, String>),
    VerifyIntegrity,
    /// Progress of a running integrity verification: files checked / total.
    IntegrityProgress(u64, u64),
    IntegrityChecked(Result<String, String>),
}

#[derive(Debug, Clone)]
pub struct ActionsModal {
    game: Arc<Mutex<MonarchGame>>,
    /// Progress/result of the last maintenance action, shown in the modal.
    status: Option<String>,
    /// Whether an integrity verification is currently running; progress
    /// messages arriving afterwards are ignored so they cannot overwrite the
    /// final result.
    verifying: bool,
}

impl ActionsModal {
    pub fn new(game: Arc<Mutex<MonarchGame>>) -> (Self, Task<Message>) {
        let modal = Self {
            game: game.clone(),
            status: None,
            verifying: false,
        };

        (modal, iced::Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CheckForUpdates => match self.game.lock() {
                Ok(game) => {
                    // Scoped check for this game only: same comparison as the
                    // start-up check, but a found update is queued for
                    // download without touching any other game's results.
                    self.status = Some(format!("Checking {} for updates...", game.name));
                    let game = game.clone();
                    return iced::Task::perform(
                        async move { monarch_games::commands::check_game_for_updates(&game).await },
                        Message::UpdatesChecked,
                    );
                }
                Err(e) => {
                    error!("actions_modal::update() Failed to lock on self.game! | Err: {e}");
                    show_error("Failed to detect game to check!");
                }
            },
            Message::UpdatesChecked(result) => match self.game.lock() {
                Ok(game) => {
                    let name = game.name.clone();
                    self.status = Some(match result {
                        Ok(GameUpdateCheck::UpToDate) => format!("{name} is up to date."),
                        Ok(GameUpdateCheck::UpdateAvailable {
                            latest_build_version,
                        }) => format!(
                            "Update for {name} found (build {latest_build_version}) and added to the download queue."
                        ),
                        Err(e) => format!("Failed to check for updates! {e}"),
                    });
                }
                Err(e) => {
                    error!("actions_modal::update() Failed to lock on self.game! | Err: {e}");
                }
            },
            Message::VerifyIntegrity => match self.game.lock() {
                Ok(game) => {
                    self.status = Some(String::from(
                        "Verifying integrity of game files... This can take a while.",
                    ));
                    self.verifying = true;

                    // Bridge verification progress from the background thread
                    // into the iced message loop via the global GUI sender.
                    let on_progress: integrity::ProgressCallback =
                        Arc::new(move |progress: integrity::VerificationProgress| {
                            if let Some(sender) = GUI_SENDER.lock().unwrap().as_mut() {
                                let _ = sender.unbounded_send(AppMessage::Page(
                                    pages::Message::GameDetails(
                                        pages::game_details::Message::Actions(
                                            Message::IntegrityProgress(
                                                progress.files_checked,
                                                progress.total_files,
                                            ),
                                        ),
                                    ),
                                ));
                            }
                        });

                    let game = game.clone();
                    return iced::Task::perform(
                        async move {
                            monarch_games::commands::verify_game_integrity(&game, Some(on_progress))
                                .await
                        },
                        Message::IntegrityChecked,
                    );
                }
                Err(e) => {
                    error!("actions_modal::update() Failed to lock on self.game! | Err: {e}");
                    show_error("Failed to detect game to verify!");
                }
            },
            Message::IntegrityProgress(files_checked, total_files) => {
                if self.verifying {
                    let percent = integrity::VerificationProgress {
                        files_checked,
                        total_files,
                    }
                    .percent();

                    self.status = Some(format!(
                        "Verifying integrity of game files... {files_checked} / {total_files} files ({percent}%)"
                    ));
                }
            }
            Message::IntegrityChecked(result) => {
                self.verifying = false;
                self.status = Some(match result {
                    Ok(summary) => summary,
                    Err(e) => format!("Failed to verify game files! {e}"),
                });
            }
            Message::Uninstall => match self.game.lock() {
                Ok(game) => {
                    self.status = Some(format!("Uninstalling {}...", game.name));
                    let game_id = game.id.clone();
                    let name = game.name.clone();
                    let store = game.get_store_name();
                    let store_id = game.get_store_id();
                    let is_manual = store == "monarch";
                    let game_clone = game.clone();

                    return iced::Task::perform(
                        async move {
                            let result = if is_manual {
                                monarch_games::commands::manual_remove_game(game_clone).await
                            } else {
                                monarch_games::commands::remove_game(name, store, store_id).await
                            };
                            result.map(|_| game_id)
                        },
                        Message::Uninstalled,
                    );
                }
                Err(e) => {
                    error!("actions_modal::update() Failed to lock on self.game! | Err: {e}");
                    show_error("Failed to detect game to remove!");
                }
            },
            Message::Uninstalled(result) => {
                if let Err(e) = result {
                    self.status = Some(format!("Failed to uninstall: {e}"));
                    show_error(e);
                }
                // Success is handled by the parent (remove card + navigate).
            },
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

        // Maintenance actions only work on installs Monarch downloaded itself;
        // anything else stays disabled (greyed out).
        let maintenance_available = game.is_installed && game.managed_by_monarch;

        // Update checks compare against Epic's Live builds, so they are only
        // offered for games installed through monarch_egs.
        let updates_available =
            maintenance_available && game.stores.iter().any(|store| store.name == "epicgames");

        let mut content = column![
            section_header("Maintenance"),
            action_item(
                resources::REFRESH.clone(),
                "Verify Integrity of Files",
                maintenance_available.then_some(Message::VerifyIntegrity),
            ),
            action_item(
                resources::UPDATE.clone(),
                "Check for Updates",
                updates_available.then_some(Message::CheckForUpdates),
            ),
        ];

        if let Some(status) = &self.status {
            content = content.push(
                text(status.clone())
                    .size(13)
                    .color(Color::from_rgb8(150, 150, 150)),
            );
        }

        let content = content
            .push(Space::new().height(Length::Fixed(8.0)))
            .push(section_header("Files"))
            .push(action_item(resources::FOLDER.clone(), "Open Install Location", None))
            .push(action_item(
                resources::ADD_FOLDER.clone(),
                "Move Install Folder",
                None,
            ))
            .push(Space::new().height(Length::Fixed(8.0)))
            .push(section_header("Extras"))
            .push(action_item(
                resources::FAVORITE_OUTLINE.clone(),
                "Add to Favorites",
                None
            ))
            .push(action_item(resources::VIEW.clone(), "Create Desktop Shortcut", None))
            .push(action_item(store_icon(&store_name), "View on Store", None))
            .push(Space::new().height(Length::Fixed(8.0)))
            .push(danger_action_item(
                resources::TRASH.clone(),
                remove_label,
                Some(Message::Uninstall)
            ))
            .push(Space::new().height(Length::Fixed(20.0)))
            .push(row![secondary_button("Done", Some(Message::Close))]
                .align_y(alignment::Vertical::Center))
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
    let enabled = on_press.is_some();
    let icon_color = if enabled {
        Color::from_rgb8(220, 220, 220)
    } else {
        Color::from_rgba8(220, 220, 220, 0.3)
    };

    button(
        row![
            svg(icon)
                .width(18)
                .height(18)
                .style(move |_theme: &Theme, _status| svg::Style {
                    color: Some(icon_color),
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
    let enabled = on_press.is_some();
    let icon_color = if enabled {
        Color::WHITE
    } else {
        Color::from_rgba8(255, 255, 255, 0.3)
    };

    button(
        row![
            svg(icon)
                .width(18)
                .height(18)
                .style(move |_theme: &Theme, _status| svg::Style {
                    color: Some(icon_color),
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
