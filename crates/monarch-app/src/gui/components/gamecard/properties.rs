use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use iced::widget::{button, column, combo_box, container, row, text, text_input, Space};
use iced::{alignment, border, Element, Length, Task};
use monarch_core::monarch_utils::monarch_state::MonarchState;
use tracing::error;

use crate::gui::{show_error, styles};
use monarch_core::monarch_games::commands::{
    get_executables, proton_versions, update_game_properties,
};
use monarch_core::monarch_games::monarchgame::MonarchGame;
use monarch_core::monarch_utils::monarch_vdf::ProtonVersion;

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
    game: Arc<RwLock<MonarchGame>>,
    executables: combo_box::State<String>,
    executable_list: Vec<String>,
    selected_executable: Option<String>,
    hovered_executable: Option<String>,

    compatibility_layers: combo_box::State<ProtonVersion>,
    compatibility_list: Vec<ProtonVersion>,
    selected_compatibility: Option<ProtonVersion>,

    launch_args: String,

    app_state: Arc<RwLock<MonarchState>>,
}

impl PropertiesModal {
    pub fn new(
        state_handle: Arc<RwLock<MonarchState>>,
        game: Arc<RwLock<MonarchGame>>,
    ) -> (Self, Task<Message>) {
        let (launch_args, current_executable, _) = {
            let game_lock = game.read().unwrap();
            let launch_args = game_lock.launch_args.clone();
            let current_executable = game_lock.executable_path.clone();
            (
                launch_args,
                current_executable,
                game_lock.compatibility.clone(),
            )
        };

        let state_handle_clone = state_handle.clone();
        let game_handle_clone = game.clone();

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

            app_state: state_handle,
        };

        (
            modal,
            Task::batch(vec![
                Task::perform(
                    async move {
                        // get_executables is async (DB access) and holds no blocking work,
                        // so don't wrap it in spawn_blocking — just await it directly.
                        get_executables(state_handle_clone, game_handle_clone).await
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

                if let Some(game_compat) = self.game.read().unwrap().compatibility.clone() {
                    self.selected_compatibility = self
                        .compatibility_list
                        .iter()
                        .find(|v| v.path == game_compat || v.name == game_compat)
                        .cloned();

                    if self.selected_compatibility.is_none() {
                        let compat_dir = std::path::Path::new(&game_compat)
                            .parent()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .map(|n| n.to_string());

                        if let Some(dir) = compat_dir {
                            self.selected_compatibility = self
                                .compatibility_list
                                .iter()
                                .find(|v| v.name == dir)
                                .cloned();
                        }
                    }
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
                match self.game.write() {
                    Ok(mut game) => {
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
                    }
                    Err(e) => {
                        error!("Failed to acquire read lock on PropertiesModal::game! | Err: {e}");
                        show_error("Failed to update game properties!");
                        return Task::none();
                    }
                }

                let state_handle_clone = self.app_state.clone();
                let game_handle_clone = self.game.clone();
                Task::perform(
                    async move { update_game_properties(state_handle_clone, game_handle_clone).await },
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

    /// The editable property fields (without footer or modal wrapper); used
    /// both by `view` and by the combined edit modal on the details page.
    pub fn fields(&self) -> Element<'_, Message> {
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

        column![
            text("Executables").size(18),
            hovered_path,
            executables_combo,
            Space::new().height(Length::Fixed(10.0)),
            text("Compatibility Layer").size(18),
            compatibility_combo,
            Space::new().height(Length::Fixed(10.0)),
            text("Launch Arguments").size(18),
            launch_args_input,
        ]
        .spacing(10)
        .into()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let footer = row![
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
        .align_y(alignment::Vertical::Center);

        let content = column![
            self.fields(),
            Space::new().height(Length::Fixed(20.0)),
            footer,
        ]
        .spacing(10);

        crate::gui::components::modal::Modal::new("Game Properties", content)
            .width(Length::Fixed(800.0))
            .on_close(Message::Cancel)
            .view()
    }
}
