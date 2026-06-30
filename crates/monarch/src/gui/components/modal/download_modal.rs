use iced::widget::{button, column, combo_box, radio, row, text, text_input, Space};
use iced::{alignment, Element, Length, Task};

use crate::gui::styles;
use crate::monarch_games::commands::proton_versions;
use crate::monarch_games::stores::DownloadOptions;
use crate::monarch_utils::monarch_vdf::ProtonVersion;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsTarget {
    Linux,
    Windows,
}

#[derive(Clone, Debug)]
pub enum Message {
    FolderChanged(String),
    OsSelected(OsTarget),
    CompatibilityLoaded(Vec<ProtonVersion>),
    CompatibilitySelected(ProtonVersion),
    Confirm,
    Cancel,
}

#[derive(Debug, Clone)]
pub struct DownloadModal {
    pub options: DownloadOptions,
    pub compatibility_list: Vec<ProtonVersion>,
    pub compatibility_layers: combo_box::State<ProtonVersion>,
    pub selected_compatibility: Option<ProtonVersion>,
    pub os_target: OsTarget,
}

impl DownloadModal {
    pub fn new(name: String, store: String, store_id: String) -> (Self, Task<Message>) {
        let options = DownloadOptions {
            folder: "".to_string(),
            store: store.clone(),
            game_name: name,
            game_store: store,
            game_store_id: store_id,
            os: std::env::consts::OS.to_string(),
            compatibility: None,
        };

        let is_linux = std::env::consts::OS == "linux";

        let modal = Self {
            options,
            compatibility_list: vec![ProtonVersion {
                name: "None".to_string(),
                path: "".to_string(),
            }],
            compatibility_layers: combo_box::State::new(Vec::new()),
            selected_compatibility: None,
            os_target: if is_linux {
                OsTarget::Linux
            } else {
                OsTarget::Windows
            },
        };

        if is_linux {
            let task = Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || proton_versions())
                        .await
                        .unwrap()
                },
                |res| match res {
                    Ok(versions) => Message::CompatibilityLoaded(versions),
                    Err(_) => Message::CompatibilityLoaded(vec![]),
                },
            );
            (modal, task)
        } else {
            (modal, Task::none())
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FolderChanged(f) => {
                self.options.folder = f;
                Task::none()
            }
            Message::OsSelected(os) => {
                self.os_target = os;
                self.options.os = match os {
                    OsTarget::Linux => "linux".to_string(),
                    OsTarget::Windows => "windows".to_string(),
                };
                Task::none()
            }
            Message::CompatibilityLoaded(mut versions) => {
                self.compatibility_list.append(&mut versions);
                self.compatibility_layers = combo_box::State::new(self.compatibility_list.clone());
                Task::none()
            }
            Message::CompatibilitySelected(version) => {
                self.selected_compatibility = Some(version);
                Task::none()
            }
            Message::Confirm | Message::Cancel => Task::none(), // Handled by parent
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let os_selector = if cfg!(target_os = "linux") {
            column![
                Space::new().height(Length::Fixed(15.0)),
                text("OS Target").size(16),
                row![
                    radio(
                        "Native",
                        OsTarget::Linux,
                        Some(self.os_target),
                        Message::OsSelected
                    ),
                    Space::new().width(Length::Fixed(15.0)),
                    radio(
                        "Windows",
                        OsTarget::Windows,
                        Some(self.os_target),
                        Message::OsSelected
                    ),
                ]
                .spacing(10)
            ]
            .spacing(5)
        } else {
            column![]
        };

        let compat_selector = if cfg!(target_os = "linux") && self.os_target == OsTarget::Windows {
            column![
                Space::new().height(Length::Fixed(15.0)),
                text("Compatibility Layer").size(16),
                combo_box(
                    &self.compatibility_layers,
                    "Select Compatibility Layer",
                    self.selected_compatibility.as_ref(),
                    Message::CompatibilitySelected,
                )
                .width(Length::Fill),
            ]
            .spacing(5)
        } else {
            column![]
        };

        let content = column![
            text(format!("Download {}", self.options.game_name)).size(24),
            Space::new().height(Length::Fixed(15.0)),
            text(format!("Store: {}", self.options.store)).size(16),
            Space::new().height(Length::Fixed(15.0)),
            text("Installation Folder").size(16),
            text_input(
                "Leave blank for default install location...",
                &self.options.folder
            )
            .on_input(Message::FolderChanged)
            .padding(10),
            os_selector,
            compat_selector,
            Space::new().height(Length::Fixed(20.0)),
            row![
                button(text("Download").align_x(alignment::Horizontal::Center))
                    .on_press(Message::Confirm)
                    .padding(10)
                    .style(styles::button::primary),
                Space::new().width(Length::Fixed(10.0)),
                button(text("Cancel").align_x(alignment::Horizontal::Center))
                    .on_press(Message::Cancel)
                    .padding(10)
                    .style(styles::button::secondary),
            ]
            .align_y(alignment::Vertical::Center)
        ]
        .spacing(5);

        super::Modal::new("Verify Download Options", content)
            .width(Length::Fixed(600.0))
            .on_close(Message::Cancel)
            .view()
    }
}
