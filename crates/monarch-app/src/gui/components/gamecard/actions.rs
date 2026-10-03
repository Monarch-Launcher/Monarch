use std::sync::{Arc, RwLock};

use iced::widget::{button, column, row, svg, text, Space};
use iced::{alignment, Color, Element, Length, Task, Theme};
use monarch_core::monarch_utils::monarch_game_downloader::MonarchDownloader;
use tracing::error;

use crate::gui::components::common::secondary_button;
use crate::gui::pages;
use crate::gui::{resources, show_error, styles, AppMessage, GUI_SENDER};
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
    game: Arc<RwLock<MonarchGame>>,
    /// Progress/result of the last maintenance action, shown in the modal.
    status: Option<String>,
    /// Whether an integrity verification is currently running; progress
    /// messages arriving afterwards are ignored so they cannot overwrite the
    /// final result.
    verifying: bool,

    downloader_handle: Arc<RwLock<MonarchDownloader>>,
}

impl ActionsModal {
    pub fn new(
        game: Arc<RwLock<MonarchGame>>,
        downloader_handle: Arc<RwLock<MonarchDownloader>>,
    ) -> (Self, Task<Message>) {
        let modal = Self {
            game: game,
            status: None,
            verifying: false,
            downloader_handle,
        };

        (modal, iced::Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CheckForUpdates => {
                // Scoped check for this game only: same comparison as the
                // start-up check, but a found update is queued for
                // download without touching any other game's results.
                if let Ok(game) = self.game.read() {
                    self.status = Some(format!("Checking {} for updates...", game.name));
                }

                let state_handle_clone = match self.downloader_handle.read() {
                    Ok(downloader) => downloader.state_handle.clone(),
                    Err(e) => {
                        error!("ActionsModal::update() Failed to acquire read lock on downloader_handle! | Err: {e}");
                        show_error("Failed to check for updates!");
                        return Task::none();
                    }
                };

                let downloader_handle_clone = self.downloader_handle.clone();
                let game_handle_clone = self.game.clone();

                return iced::Task::perform(
                    async move {
                        monarch_games::commands::check_game_for_updates(
                            state_handle_clone,
                            downloader_handle_clone,
                            game_handle_clone,
                        )
                        .await
                    },
                    Message::UpdatesChecked,
                );
            }
            Message::UpdatesChecked(result) => match self.game.read() {
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
            Message::VerifyIntegrity => {
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
                                pages::Message::GameDetails(pages::game_details::Message::Actions(
                                    Message::IntegrityProgress(
                                        progress.files_checked,
                                        progress.total_files,
                                    ),
                                )),
                            ));
                        }
                    });

                let settings_handle_clone = match self.downloader_handle.read() {
                    Ok(downlaoder) => match downlaoder.state_handle.read() {
                        Ok(state) => state.get_settings_ptr(),
                        Err(e) => {
                            error!("ActionsModal::update() Failed to acquire read lock on state_handle! | Err: {e}");
                            show_error("Failed to verify game integrity!");
                            return Task::none();
                        }
                    },
                    Err(e) => {
                        error!("ActionsModal::update() Failed to acquire read lock on downloader_handle! | Err: {e}");
                        show_error("Failed to verify game integrity!");
                        return Task::none();
                    }
                };

                let game_handle_clone = self.game.clone();
                return iced::Task::perform(
                    async move {
                        monarch_games::commands::verify_game_integrity(
                            settings_handle_clone,
                            game_handle_clone,
                            Some(on_progress),
                        )
                        .await
                    },
                    Message::IntegrityChecked,
                );
            }
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
            Message::Uninstall => {
                let is_manual: bool;
                let game_id: String;
                match self.game.read() {
                    Ok(game) => {
                        self.status = Some(format!("Uninstalling {}...", game.name));
                        game_id = game.id.clone();
                        is_manual = game.get_store_name() == "monarch";
                    }
                    Err(e) => {
                        error!("ActionsModal::update() Failed to acquire read lock on game_handle! | Err: {e}");
                        show_error("Failed to uninstall game!");
                        return Task::none();
                    }
                }

                let state_handle_clone;
                let settings_handle_clone;

                match self.downloader_handle.read() {
                    Ok(downloader) => {
                        state_handle_clone = downloader.state_handle.clone();
                        match state_handle_clone.read() {
                            Ok(state) => {
                                settings_handle_clone = state.get_settings_ptr();
                            }
                            Err(e) => {
                                error!("ActionsModal::update() Failed to acquire read lock on state_handle! | Err: {e}");
                                show_error("Failed to uninstall game!");
                                return Task::none();
                            }
                        }
                    }
                    Err(e) => {
                        error!("ActionsModal::update() Failed to acquire read lock on downloader_handle! | Err: {e}");
                        show_error("Failed to uninstall game!");
                        return Task::none();
                    }
                };
                let game_handle_clone = self.game.clone();

                return iced::Task::perform(
                    async move {
                        let result = if is_manual {
                            monarch_games::commands::manual_remove_game(
                                state_handle_clone,
                                game_handle_clone,
                            )
                            .await
                        } else {
                            monarch_games::commands::remove_game(
                                settings_handle_clone,
                                game_handle_clone,
                            )
                            .await
                        };
                        result.map(|_| game_id)
                    },
                    Message::Uninstalled,
                );
            }
            Message::Uninstalled(result) => {
                if let Err(e) = result {
                    self.status = Some(format!("Failed to uninstall: {e}"));
                    show_error(e);
                }
                // Success is handled by the parent (remove card + navigate).
            }
            Message::Close => {}
        }
        iced::Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        match self.game.read() {
            Ok(game) => {
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
                let updates_available = maintenance_available
                    && game.stores.iter().any(|store| store.name == "epicgames");

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
                    .push(action_item(
                        resources::FOLDER.clone(),
                        "Open Install Location",
                        None,
                    ))
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
                        None,
                    ))
                    .push(action_item(
                        resources::VIEW.clone(),
                        "Create Desktop Shortcut",
                        None,
                    ))
                    .push(action_item(store_icon(&store_name), "View on Store", None))
                    .push(Space::new().height(Length::Fixed(8.0)))
                    .push(danger_action_item(
                        resources::TRASH.clone(),
                        remove_label,
                        Some(Message::Uninstall),
                    ))
                    .push(Space::new().height(Length::Fixed(20.0)))
                    .push(
                        row![secondary_button("Done", Some(Message::Close))]
                            .align_y(alignment::Vertical::Center),
                    )
                    .spacing(10);

                crate::gui::components::modal::Modal::new("Actions", content)
                    .width(Length::Fixed(800.0))
                    .on_close(Message::Close)
                    .view()
            }
            Err(e) => {
                error!("actions_modal::view() Failed to lock on self.game! | Err: {e}");
                show_error("Failed to open actions for selected game!");

                let content = column![];
                crate::gui::components::modal::Modal::new("Actions", content)
                    .width(Length::Fixed(800.0))
                    .view()
            }
        }
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
