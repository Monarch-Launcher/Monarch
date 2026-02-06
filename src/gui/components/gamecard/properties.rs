use std::path::PathBuf;

use iced::widget::{button, column, combo_box, row, text, text_input, Space};
use iced::{alignment, Element, Length, Task};
use tracing::error;

use crate::gui::styles;
use crate::monarch_games::commands::{get_executables, proton_versions, update_game_properties};
use crate::monarch_games::monarchgame::MonarchGame;
use crate::monarch_utils::monarch_vdf::ProtonVersion;

#[derive(Clone, Debug)]
pub enum Message {
    ExecutablesLoaded(Vec<PathBuf>),
    CompatibilityLoaded(Vec<ProtonVersion>),
    ExecutableSelected(String),
    CompatibilitySelected(ProtonVersion),
    LaunchArgsChanged(String),
    Save,
    Cancel,
    NoOp,
}

pub struct PropertiesModal {
    game: MonarchGame,
    executables: combo_box::State<String>,
    executable_list: Vec<String>,
    selected_executable: Option<String>,

    compatibility_layers: combo_box::State<ProtonVersion>,
    compatibility_list: Vec<ProtonVersion>,
    selected_compatibility: Option<ProtonVersion>,

    launch_args: String,
}

impl PropertiesModal {
    pub fn new(game: MonarchGame) -> (Self, Task<Message>) {
        let launch_args = game.launch_args.clone();
        let current_executable = if game.executable_path.is_empty() {
            None
        } else {
            Some(game.executable_path.clone())
        };

        let _current_compatibility = if game.compatibility.is_empty() {
            None
        } else {
            Some(game.compatibility.clone())
        };

        let modal = Self {
            game: game.clone(),
            executables: combo_box::State::new(Vec::new()),
            executable_list: Vec::new(),
            selected_executable: current_executable,

            compatibility_layers: combo_box::State::new(Vec::new()),
            compatibility_list: Vec::new(),
            selected_compatibility: None,

            launch_args,
        };

        (
            modal,
            Task::batch(vec![
                Task::perform(
                    async move {
                        let game_clone = game.clone();
                        tokio::task::spawn_blocking(move || get_executables(game_clone))
                            .await
                            .unwrap()
                    },
                    |res| match res {
                        Ok(exes) => Message::ExecutablesLoaded(exes),
                        Err(e) => {
                            error!("PropertiesModal: Failed to load executables: {}", e);
                            Message::ExecutablesLoaded(vec![])
                        }
                    },
                ),
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || proton_versions())
                            .await
                            .unwrap()
                    },
                    |res| match res {
                        Ok(versions) => Message::CompatibilityLoaded(versions),
                        Err(e) => {
                            error!("PropertiesModal: Failed to load proton versions: {}", e);
                            Message::CompatibilityLoaded(vec![])
                        }
                    },
                ),
            ]),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ExecutablesLoaded(exes) => {
                self.executable_list = exes
                    .into_iter()
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect();
                self.executables = combo_box::State::new(self.executable_list.clone());

                if self.selected_executable.is_none() && !self.executable_list.is_empty() {
                    // Don't auto-select
                }
                Task::none()
            }
            Message::CompatibilityLoaded(versions) => {
                self.compatibility_list = versions;
                self.compatibility_layers = combo_box::State::new(self.compatibility_list.clone());

                if !self.game.compatibility.is_empty() {
                    self.selected_compatibility = self
                        .compatibility_list
                        .iter()
                        .find(|v| v.name == self.game.compatibility)
                        .cloned();
                }

                Task::none()
            }
            Message::ExecutableSelected(exe) => {
                self.selected_executable = Some(exe);
                Task::none()
            }
            Message::CompatibilitySelected(version) => {
                self.selected_compatibility = Some(version);
                Task::none()
            }
            Message::LaunchArgsChanged(args) => {
                self.launch_args = args;
                Task::none()
            }
            Message::Save => {
                let mut game = self.game.clone();
                if let Some(exe) = &self.selected_executable {
                    game.executable_path = exe.clone();
                }
                if let Some(compat) = &self.selected_compatibility {
                    game.compatibility = compat.name.clone();
                }
                game.launch_args = self.launch_args.clone();

                Task::perform(async move { update_game_properties(game).await }, |res| {
                    if let Err(e) = res {
                        error!("Failed to save properties: {}", e);
                    }
                    Message::Cancel
                })
            }
            Message::Cancel => Task::none(),
            Message::NoOp => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let executables_combo = combo_box(
            &self.executables,
            "Select Executable",
            self.selected_executable.as_ref(),
            Message::ExecutableSelected,
        )
        .width(Length::Fill);

        let compatibility_combo = combo_box(
            &self.compatibility_layers,
            "Select Compatibility Layer",
            self.selected_compatibility.as_ref(),
            Message::CompatibilitySelected,
        )
        .width(Length::Fill);

        let launch_args_input = text_input("Custom Launch Arguments", &self.launch_args)
            .on_input(Message::LaunchArgsChanged)
            .padding(10);

        let content = column![
            text("Executables").size(18),
            executables_combo,
            Space::new().height(Length::Fixed(10.0)),
            text("Compatibility Layer").size(18),
            compatibility_combo,
            Space::new().height(Length::Fixed(10.0)),
            text("Launch Arguments").size(18),
            launch_args_input,
            Space::new().height(Length::Fixed(20.0)),
            row![
                button(text("Save"))
                    .on_press(Message::Save)
                    .padding(10)
                    .style(styles::button::primary),
                Space::new().width(Length::Fixed(10.0)),
                button(text("Cancel"))
                    .on_press(Message::Cancel)
                    .padding(10)
                    .style(styles::button::secondary),
            ]
            .align_y(alignment::Vertical::Center)
        ]
        .spacing(10);

        crate::gui::components::modal::Modal::new("Game Properties", content)
            .width(Length::Fixed(800.0))
            .on_close(Message::Cancel)
            .view()
    }
}
