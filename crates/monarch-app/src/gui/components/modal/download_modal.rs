use iced::widget::{button, column, combo_box, radio, row, text, text_input, Space};
use iced::{alignment, Element, Length, Task};

use crate::gui::components::common::{open_folder_dialog, secondary_button};
use crate::gui::styles;
use monarch_core::monarch_games::commands::proton_versions;
use monarch_core::monarch_games::stores::DownloadOptions;
use monarch_core::monarch_utils::monarch_vdf::ProtonVersion;
use monarch_egs::SupportedPlatforms;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsTarget {
    Linux,
    Windows,
}

#[derive(Clone, Debug)]
pub enum Message {
    FolderChanged(String),
    BrowseFolder,
    OsSelected(OsTarget),
    CompatibilityLoaded(Vec<ProtonVersion>),
    CompatibilitySelected(ProtonVersion),
    PlatformSupportLoaded(Result<SupportedPlatforms, String>),
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
    pub platform_support: Option<SupportedPlatforms>,
    /// True while we're waiting for the EGS platform support check to return.
    loading_platform_support: bool,
}

impl DownloadModal {
    pub fn new(name: String, store: String, store_id: String) -> (Self, Task<Message>) {
        let options = DownloadOptions {
            folder: "".to_string(),
            store: store.clone(),
            game_name: name.clone(),
            game_store: store.clone(),
            game_store_id: store_id.clone(),
            os: std::env::consts::OS.to_string(),
            compatibility: None,
        };

        let is_linux = std::env::consts::OS == "linux";
        let needs_platform_check = store == "epicgames";

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
            platform_support: None,
            loading_platform_support: needs_platform_check,
        };

        let mut tasks = Vec::new();

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
            tasks.push(task);
        }

        // Check platform support for Epic Games Store games
        if needs_platform_check {
            let task = Task::perform(
                async move {
                    let game = monarch_core::monarch_games::monarchgame::MonarchGame::new(
                        &name, -1, "epicgames", &store_id, "", "", "",
                    );
                    monarch_core::monarch_games::commands::check_egs_platform_support(&game).await
                },
                Message::PlatformSupportLoaded,
            );
            tasks.push(task);
        }

        if tasks.is_empty() {
            (modal, Task::none())
        } else {
            (modal, Task::batch(tasks))
        }
    }

    /// Returns true if Linux is supported for download.
    pub fn linux_supported(&self) -> bool {
        self.platform_support
            .as_ref()
            .map(|p| p.linux)
            .unwrap_or(true) // Default to true if not loaded yet
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FolderChanged(f) => {
                self.options.folder = f;
                Task::none()
            }
            Message::BrowseFolder => Task::future(open_folder_dialog()).then(|handle| match handle {
                Some(file_handle) => Task::done(Message::FolderChanged(
                    file_handle.path().to_string_lossy().to_string(),
                )),
                None => Task::none(),
            }),
            Message::OsSelected(os) => {
                // Don't allow selecting while loading or if not supported
                if self.loading_platform_support {
                    return Task::none();
                }
                if os == OsTarget::Linux && !self.linux_supported() {
                    return Task::none();
                }
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
            Message::PlatformSupportLoaded(result) => {
                self.loading_platform_support = false;
                match result {
                    Ok(support) => {
                        let linux_supported = support.linux;
                        self.platform_support = Some(support);
                        // If Linux is not supported and currently selected, switch to Windows
                        if !linux_supported && self.os_target == OsTarget::Linux {
                            self.os_target = OsTarget::Windows;
                            self.options.os = "windows".to_string();
                        }
                    }
                    Err(e) => {
                        // On error, assume all platforms are supported
                        tracing::error!("Failed to check platform support: {}", e);
                        self.platform_support = Some(SupportedPlatforms {
                            windows: true,
                            linux: true,
                            macos: true,
                        });
                    }
                }
                Task::none()
            }
            Message::Confirm | Message::Cancel => Task::none(), // Handled by parent
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let linux_supported = self.linux_supported();

        let os_selector = if cfg!(target_os = "linux") {
            let native_option = if self.loading_platform_support {
                column![
                    text("Native").style(|theme: &iced::Theme| {
                        iced::widget::text::Style {
                            color: Some(theme.palette().text.scale_alpha(0.3)),
                        }
                    }),
                ]
            } else if linux_supported {
                column![
                    radio(
                        "Native",
                        OsTarget::Linux,
                        Some(self.os_target),
                        Message::OsSelected
                    ),
                ]
            } else {
                column![
                    text("Native (Not available)").style(|theme: &iced::Theme| {
                        iced::widget::text::Style {
                            color: Some(theme.palette().text.scale_alpha(0.3)),
                        }
                    }),
                ]
            };

            let windows_option = if self.loading_platform_support {
                column![
                    text("Windows").style(|theme: &iced::Theme| {
                        iced::widget::text::Style {
                            color: Some(theme.palette().text.scale_alpha(0.3)),
                        }
                    }),
                ]
            } else {
                column![
                    radio(
                        "Windows",
                        OsTarget::Windows,
                        Some(self.os_target),
                        Message::OsSelected
                    ),
                ]
            };

            let loading_text = if self.loading_platform_support {
                column![
                    text("Getting OS versions...").style(|theme: &iced::Theme| {
                        iced::widget::text::Style {
                            color: Some(theme.palette().text.scale_alpha(0.5)),
                        }
                    }),
                ]
            } else {
                column![]
            };

            column![
                Space::new().height(Length::Fixed(15.0)),
                text("OS Target").size(16),
                row![
                    native_option,
                    Space::new().width(Length::Fixed(15.0)),
                    windows_option,
                ]
                .spacing(10),
                loading_text,
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
            row![
                text_input(
                    "Leave blank for default install location...",
                    &self.options.folder
                )
                .on_input(Message::FolderChanged)
                .padding(10)
                .width(Length::Fill),
                Space::new().width(Length::Fixed(10.0)),
                secondary_button("Browse", Some(Message::BrowseFolder)),
            ]
            .align_y(alignment::Vertical::Center),
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
