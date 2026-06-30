use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use iced::widget::{button, column, combo_box, container, row, text, text_input, Space};
use iced::{alignment, border, Element, Length, Task};
use tracing::error;

use crate::gui::{show_error, styles};
use crate::monarch_games::commands::{get_executables, proton_versions, update_game_properties};
use crate::monarch_games::monarchgame::MonarchGame;
use crate::monarch_utils::monarch_vdf::ProtonVersion;

#[derive(Clone, Debug)]
pub enum Message {
    ExecutablesLoaded(Vec<PathBuf>),
    CompatibilityLoaded(Vec<ProtonVersion>),
    ExecutableSelected(String),
    ExecutableHovered(String),
    CompatibilitySelected(ProtonVersion),
    LaunchArgsChanged(String),
    Save,
    Cancel,
}

#[derive(Debug, Clone)]
pub struct PropertiesModal {
    game: Arc<Mutex<MonarchGame>>,
    executables: combo_box::State<String>,
    executable_list: Vec<String>,
    selected_executable: Option<String>,
    hovered_executable: Option<String>,

    compatibility_layers: combo_box::State<ProtonVersion>,
    compatibility_list: Vec<ProtonVersion>,
    selected_compatibility: Option<ProtonVersion>,

    launch_args: String,
}

impl PropertiesModal {
    pub fn new(game: Arc<Mutex<MonarchGame>>) -> (Self, Task<Message>) {
        let (launch_args, current_executable, _) = {
            let game_lock = game.lock().unwrap();
            let launch_args = game_lock.launch_args.clone();
            let current_executable = game_lock.executable_path.clone();
            (
                launch_args,
                current_executable,
                game_lock.compatibility.clone(),
            )
        };

        let modal = Self {
            game: game.clone(),
            executables: combo_box::State::new(Vec::new()),
            executable_list: vec!["None".to_string()],
            selected_executable: current_executable,
            hovered_executable: None,

            compatibility_layers: combo_box::State::new(Vec::new()),
            compatibility_list: vec![ProtonVersion {
                name: "Native".to_string(),
                path: "".to_string(),
            }],
            selected_compatibility: None,

            launch_args: launch_args.unwrap_or_default(),
        };

        (
            modal,
            Task::batch(vec![
                Task::perform(
                    async move {
                        let mut game_inner = game.lock().unwrap().clone();
                        tokio::task::spawn_blocking(move || get_executables(&mut game_inner))
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
                self.executable_list.append(
                    &mut exes
                        .into_iter()
                        .map(|p| p.to_string_lossy().into_owned())
                        .collect::<Vec<String>>(),
                );
                self.executables = combo_box::State::new(self.executable_list.clone());

                if self.selected_executable.is_none() && !self.executable_list.is_empty() {
                    // Don't auto-select
                }
                Task::none()
            }
            Message::CompatibilityLoaded(mut versions) => {
                self.compatibility_list.append(&mut versions);
                self.compatibility_layers = combo_box::State::new(self.compatibility_list.clone());

                if let Some(game_compat) = self.game.lock().unwrap().compatibility.clone() {
                    self.selected_compatibility = self
                        .compatibility_list
                        .iter()
                        .find(|v| v.name == game_compat)
                        .cloned();
                }

                Task::none()
            }
            Message::ExecutableSelected(exe) => {
                self.selected_executable = if exe == "None" { None } else { Some(exe) };
                self.hovered_executable = None;
                Task::none()
            }
            Message::ExecutableHovered(exe) => {
                self.hovered_executable = Some(exe);
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
                let mut game = self.game.lock().unwrap();
                game.executable_path = self.selected_executable.clone();

                match &self.selected_compatibility {
                    Some(compat) => game.compatibility = Some(compat.path.clone()),
                    None => game.compatibility = None,
                }

                game.launch_args = if self.launch_args.is_empty() {
                    None
                } else {
                    Some(self.launch_args.clone())
                };

                let game_clone = game.clone();
                Task::perform(
                    async move { update_game_properties(&game_clone).await },
                    |res| {
                        if let Err(e) = res {
                            show_error(e);
                        }
                        Message::Cancel
                    },
                )
            }
            Message::Cancel => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let executables_combo = combo_box(
            &self.executables,
            "Select Executable",
            self.selected_executable.as_ref(),
            Message::ExecutableSelected,
        )
        .on_option_hovered(Message::ExecutableHovered)
        .width(Length::Fill);

        let hovered_path = if let Some(path) = &self.hovered_executable {
            container(text(path).size(12))
                .padding(5)
                .style(|theme: &iced::Theme| {
                    let palette = theme.palette();
                    container::Style {
                        background: Some(iced::Color::from_rgb8(30, 30, 45).into()),
                        border: border::Border {
                            color: palette.primary,
                            width: 1.0,
                            radius: crate::gui::styles::radius::SUBTLE.into(),
                        },
                        text_color: Some(palette.text),
                        ..Default::default()
                    }
                })
                .width(Length::Fill)
        } else {
            container(Space::new().height(Length::Fixed(28.0)))
        };

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
            hovered_path,
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
