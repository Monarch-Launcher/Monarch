use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use iced::widget::{
    button, checkbox, column, combo_box, container, row, scrollable, text, text_input, Space,
};
use iced::{alignment, border, Color, Element, Length, Task};
use monarch_core::monarch_games::games::CompatOptions;
use monarch_core::monarch_utils::monarch_state::MonarchState;
use tracing::error;

use crate::gui::{show_error, styles};
use monarch_core::monarch_games::commands::{
    get_executables, proton_versions, update_game_properties, view_launch_command,
};
use monarch_core::monarch_games::monarchgame::MonarchGame;
use monarch_core::monarch_utils::monarch_vdf::ProtonVersion;

/// Signals for the properties modal. An option row only needs a signal to
/// say "something changed": each row mutates its field of `compat_opts`
/// directly, in place.
#[derive(Clone, Debug)]
pub enum Message {
    ExecutablesLoaded(Vec<PathBuf>),
    ExecutableSelected(String),
    ExecutableHovered(String),
    CompatibilitySelected(ProtonVersion),
    /// Game specific launch args appended at the end of the launch command.
    GameArgsChanged(String),
    /// Env args prepended at the front of the launch command.
    EnvArgsChanged(String),
    /// Open the Proton/Wine options page.
    OpenCompatOpts,
    /// Leave the Proton/Wine options page and return to launch options.
    CompatOptsBack,
    /// An option was mutated in place from its row's callback; this message
    /// only makes the runtime redraw with the fresh values.
    CompatOptsChanged,

    LaunchCommandGenerated(String),

    SaveCompatOpts,
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
    selected_compatibility: Option<ProtonVersion>,

    /// Launch args appended at the end of the launch command. For Steam
    /// games this matches whatever comes after %command%.
    game_args: String,

    /// Env args prepended at the front of the launch command, e.g.
    /// "PROTON_ENABLE_WAYLAND=1 ...".
    env_args: String,

    /// The generated (read-only) launch command shown on the main page.
    launch_command: String,

    /// Whether the Proton/Wine options page is shown instead of the regular
    /// launch options fields.
    show_compat_opts: bool,

    /// The Proton/Wine options being edited. Read and mutated in place by the
    /// option rows; Save writes it into the game.
    compat_opts: RefCell<CompatOptions>,

    app_state: Arc<RwLock<MonarchState>>,
}

impl PropertiesModal {
    pub fn new(
        state_handle: Arc<RwLock<MonarchState>>,
        game: Arc<RwLock<MonarchGame>>,
    ) -> (Self, Task<Message>) {
        let (game_args, env_args, current_executable, current_compat, compat_opts_opt) = {
            let game_lock = game.read().unwrap();
            (
                game_lock.launch_args.clone(),
                game_lock.env_args.clone(),
                game_lock.executable_path.clone(),
                game_lock.compatibility.clone(),
                game_lock.compatibility_opts.clone(),
            )
        };

        let state_handle_clone = state_handle.clone();
        let game_handle_clone = game.clone();
        let state_handle_clone2 = state_handle.clone();
        let game_handle_clone2 = game.clone();

        let compat_opts = compat_opts_opt.unwrap_or_default();

        let mut compat_list: Vec<ProtonVersion> = vec![ProtonVersion {
            name: "Native".to_string(),
            path: "".to_string(),
        }];
        if let Ok(mut more_protons) = proton_versions() {
            compat_list.append(&mut more_protons);
        }

        let selected_compat: Option<ProtonVersion> = match current_compat {
            Some(compat) => compat_list.iter().find(|&c| c.path == compat).cloned(),
            None => None,
        };

        let modal = Self {
            game: game.clone(),
            executables: combo_box::State::new(Vec::new()),
            executable_list: vec!["None".to_string()],
            selected_executable: current_executable.clone(),
            hovered_executable: None,

            compatibility_layers: combo_box::State::new(compat_list),
            selected_compatibility: selected_compat,
            game_args: game_args.unwrap_or_default(),
            env_args: env_args.unwrap_or_default(),
            launch_command: String::new(),

            show_compat_opts: false,
            compat_opts: RefCell::new(compat_opts),

            app_state: state_handle,
        };

        (
            modal,
            Task::batch(vec![
                Task::perform(
                    async move {
                        // get_executables is async (DB access) and holds no blocking work,
                        // so don't wrap it in spawn_blocking — just await it directly.
                        get_executables(state_handle_clone.clone(), game_handle_clone.clone()).await
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
                        view_launch_command(
                            game_handle_clone2.clone(),
                            state_handle_clone2.clone(),
                            &current_executable.unwrap_or("".to_string()),
                        )
                        .await
                    },
                    |res| match res {
                        Ok(command) => Message::LaunchCommandGenerated(command),
                        Err(e) => {
                            show_error(e);
                            Message::Cancel
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
            Message::ExecutableSelected(exe) => {
                self.selected_executable = if exe == "None" { None } else { Some(exe) };
                self.hovered_executable = None;

                let game_handle_clone = self.game.clone();
                let state_handle_clone = self.app_state.clone();
                let exe: String = self.selected_executable.clone().unwrap_or("".to_string());

                Task::perform(
                    async move {
                        view_launch_command(
                            game_handle_clone.clone(),
                            state_handle_clone.clone(),
                            exe.as_str(),
                        )
                        .await
                    },
                    |res| match res {
                        Ok(command) => Message::LaunchCommandGenerated(command),
                        Err(e) => {
                            show_error(e);
                            Message::Cancel
                        }
                    },
                )
            }
            Message::ExecutableHovered(exe) => {
                self.hovered_executable = Some(exe);
                Task::none()
            }
            Message::CompatibilitySelected(version) => {
                self.selected_compatibility = Some(version);
                Task::none()
            }
            Message::GameArgsChanged(args) => {
                self.game_args = args;
                Task::none()
            }
            Message::EnvArgsChanged(args) => {
                self.env_args = args;
                Task::none()
            }
            Message::OpenCompatOpts => {
                self.show_compat_opts = true;
                Task::none()
            }
            Message::CompatOptsBack => {
                self.show_compat_opts = false;
                Task::none()
            }
            Message::CompatOptsChanged => Task::none(),
            Message::LaunchCommandGenerated(command) => {
                self.launch_command = command;
                Task::none()
            }
            Message::SaveCompatOpts => {
                let mut game_exe: String = String::new();
                match self.game.write() {
                    Ok(mut game) => {
                        game.compatibility_opts = Some(self.compat_opts.borrow().clone());
                        if let Some(path) = &game.executable_path {
                            game_exe = path.clone();
                        }
                    }
                    Err(e) => {
                        error!("Failed to acquire read lock on PropertiesModal::game! | Err: {e}");
                        show_error("Failed to update game properties!");
                        return Task::none();
                    }
                }

                let state_handle_clone = self.app_state.clone();
                let game_handle_clone = self.game.clone();
                let state_handle_clone2 = self.app_state.clone();
                let game_handle_clone2 = self.game.clone();

                Task::batch(vec![
                    Task::perform(
                        async move {
                            view_launch_command(
                                game_handle_clone.clone(),
                                state_handle_clone.clone(),
                                game_exe.as_str(),
                            )
                            .await
                        },
                        |res| match res {
                            Ok(command) => Message::LaunchCommandGenerated(command),
                            Err(e) => {
                                show_error(e);
                                Message::Cancel
                            }
                        },
                    ),
                    Task::perform(
                        async move {
                            update_game_properties(
                                state_handle_clone2.clone(),
                                game_handle_clone2.clone(),
                            )
                            .await
                        },
                        |res| {
                            if let Err(e) = res {
                                show_error(e);
                            }
                            Message::CompatOptsBack
                        },
                    ),
                ])
            }
            Message::Save => {
                let mut game_exe: String = String::new();
                match self.game.write() {
                    Ok(mut game) => {
                        game.executable_path = self.selected_executable.clone();
                        if let Some(path) = &self.selected_executable {
                            game_exe = path.clone();
                        }

                        match &self.selected_compatibility {
                            Some(compat) => game.compatibility = Some(compat.path.clone()),
                            None => game.compatibility = None,
                        }

                        game.launch_args = if self.game_args.trim().is_empty() {
                            None
                        } else {
                            Some(self.game_args.trim().to_string())
                        };

                        game.env_args = if self.env_args.trim().is_empty() {
                            None
                        } else {
                            Some(self.env_args.trim().to_string())
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
                let state_handle_clone2 = self.app_state.clone();
                let game_handle_clone2 = self.game.clone();

                Task::batch(vec![
                    Task::perform(
                        async move {
                            view_launch_command(
                                game_handle_clone.clone(),
                                state_handle_clone.clone(),
                                game_exe.as_str(),
                            )
                            .await
                        },
                        |res| match res {
                            Ok(command) => Message::LaunchCommandGenerated(command),
                            Err(e) => {
                                show_error(e);
                                Message::Cancel
                            }
                        },
                    ),
                    Task::perform(
                        async move {
                            update_game_properties(
                                state_handle_clone2.clone(),
                                game_handle_clone2.clone(),
                            )
                            .await
                        },
                        |res| {
                            if let Err(e) = res {
                                show_error(e);
                            }
                            Message::CompatOptsBack
                        },
                    ),
                ])
            }
            Message::Cancel => Task::none(),
        }
    }

    /// Whether the Proton/Wine options page is currently shown instead of
    /// the regular launch-option fields.
    pub fn showing_compat_opts(&self) -> bool {
        self.show_compat_opts
    }

    /// The editable property fields (without footer or modal wrapper); used
    /// both by `view` and by the combined edit modal on the details page.
    pub fn fields(&self) -> Element<'_, Message> {
        if self.show_compat_opts {
            return self.compat_opts_fields();
        }

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

        // Read-only display of the launch command Monarch generates (from the
        // compatibility options, env args and game args).
        let launch_command_display = text_input("Launch Command", &self.launch_command).padding(10);

        let game_args_input = text_input("Game Specific Launch Args", &self.game_args)
            .on_input(Message::GameArgsChanged)
            .padding(10);

        let env_args_input = text_input("Env Args", &self.env_args)
            .on_input(Message::EnvArgsChanged)
            .padding(10);

        // Only display the compatibility options to Linux users
        #[cfg(target_os = "linux")]
        {
            let open_compat_opts_button = button(
                row![
                    text("Set Proton/Wine Options").size(15),
                    Space::new().width(Length::Fill),
                ]
                .align_y(alignment::Vertical::Center),
            )
            .on_press(Message::OpenCompatOpts)
            .style(styles::button::secondary)
            .width(Length::Fill)
            .padding(12);

            column![
                text("Executables").size(18),
                hovered_path,
                executables_combo,
                Space::new().height(Length::Fixed(10.0)),
                text("Compatibility Layer").size(18),
                compatibility_combo,
                open_compat_opts_button,
                Space::new().height(Length::Fixed(10.0)),
                text("Launch Command").size(18),
                text("This is a simplified launch command. Monarch performs some more steps under the hood.")
                    .size(12)
                    .color(Color::from_rgb8(140, 140, 140)),
                launch_command_display,
                Space::new().height(Length::Fixed(10.0)),
                text("Game Specific Launch Args").size(18),
                text("Launch arguments appended at the end of the launch command. For Steam games this is whatever comes after %command%.")
                    .size(12)
                    .color(Color::from_rgb8(140, 140, 140)),
                game_args_input,
                Space::new().height(Length::Fixed(10.0)),
                text("Env Args").size(18),
                text("Environment arguments prepended to the front of the launch command.")
                    .size(12)
                    .color(Color::from_rgb8(140, 140, 140)),
                env_args_input,
            ]
            .spacing(10)
            .into()
        }

        // Only display the compatibility options to Linux users
        #[cfg(not(target_os = "linux"))]
        {
            column![
                text("Executables").size(18),
                hovered_path,
                executables_combo,
                Space::new().height(Length::Fixed(10.0)),
                text("Compatibility Layer").size(18),
                compatibility_combo,
                Space::new().height(Length::Fixed(10.0)),
                text("Game Specific Launch Args").size(18),
                text("Launch arguments appended at the end of the launch command.")
                    .size(12)
                    .color(Color::from_rgb8(140, 140, 140)),
                game_args_input,
                Space::new().height(Length::Fixed(10.0)),
                text("Env Args").size(18),
                text("Environment arguments prepended to the front of the launch command.")
                    .size(12)
                    .color(Color::from_rgb8(140, 140, 140)),
                env_args_input,
            ]
            .spacing(10)
            .into()
        }
    }

    /// The whole properties modal (fields + footer wrapped in a `Modal`),
    /// used when `PropertiesModal` stands on its own (`game_browser`).
    pub fn view(&self) -> Element<'_, Message> {
        crate::gui::components::modal::Modal::new("Game Properties", self.content())
            .width(Length::Fixed(800.0))
            .on_close(Message::Cancel)
            .view()
    }

    /// The fields plus the Save/Cancel footer, without the modal wrapper.
    /// Used by callers that provide their own modal chrome (e.g. the
    /// combined edit modal on the details page).
    pub fn content(&self) -> Element<'_, Message> {
        let save_button = if self.show_compat_opts {
            button(text("Save"))
                .on_press(Message::SaveCompatOpts)
                .padding(10)
                .style(styles::button::primary)
        } else {
            button(text("Save"))
                .on_press(Message::Save)
                .padding(10)
                .style(styles::button::primary)
        };

        let back_button = if self.show_compat_opts {
            button(text("Back"))
                .on_press(Message::CompatOptsBack)
                .padding(10)
                .style(styles::button::secondary)
        } else {
            button(text("Cancel"))
                .on_press(Message::Cancel)
                .padding(10)
                .style(styles::button::secondary)
        };

        let footer = row![
            save_button,
            Space::new().width(Length::Fixed(10.0)),
            back_button,
        ]
        .align_y(alignment::Vertical::Center);

        column![
            self.fields(),
            Space::new().height(Length::Fixed(20.0)),
            footer,
        ]
        .spacing(10)
        .into()
    }

    /// The Proton/Wine options editor, shown as its own page in place of the
    /// regular launch-option fields. Every row reads and writes its field of
    /// `self.compat_opts` directly, mutating it in place.
    fn compat_opts_fields(&self) -> Element<'_, Message> {
        let content = column![
            text("Proton/Wine Options")
                .size(20)
                .font(styles::fonts::BOLD),
            compat_section_header("Gamemode"),
            self.gamemode_rows_section(),
            compat_section_header("Gamescope"),
            self.gamescope_rows_section(),
            compat_section_header("Misc"),
            self.misc_rows_section(),
            compat_section_header("Proton"),
            self.proton_rows_section(),
            compat_section_header("Wine"),
            self.wine_rows_section(),
            compat_section_header("DXVK"),
            self.dxvk_rows_section(),
        ]
        .spacing(10);

        // The scrollbar is drawn on the scrollable's right edge on top of the
        // content, so inset the page slightly so it doesn't overlap the
        // text inputs.
        let content = container(content)
            .width(Length::Fill)
            .padding(iced::Padding {
                right: 14.0,
                ..Default::default()
            });

        scrollable(content).height(Length::Fixed(560.0)).into()
    }

    /// The Proton section rows.
    fn proton_rows_section(&self) -> Element<'_, Message> {
        let o = self.compat_opts.borrow();

        column![
            compat_flag_row(
                "PROTON_USE_WINED3D",
                "Use WineD3D/OpenGL instead of DXVK for D3D9/10/11.",
                o.proton_use_wined3d,
                |on| {
                    self.compat_opts.borrow_mut().proton_use_wined3d = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "PROTON_NO_D3D12",
                "Disable D3D12.",
                o.proton_no_d3d12,
                |on| {
                    self.compat_opts.borrow_mut().proton_no_d3d12 = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "PROTON_NO_D3D11",
                "Disable D3D11.",
                o.proton_no_d3d11,
                |on| {
                    self.compat_opts.borrow_mut().proton_no_d3d11 = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "PROTON_NO_D3D10",
                "Disable D3D10/DXGI.",
                o.proton_no_d3d10,
                |on| {
                    self.compat_opts.borrow_mut().proton_no_d3d10 = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "PROTON_USE_ESYNC",
                "Disable futex-based synchronization (ESync).",
                o.proton_use_esync,
                |on| {
                    self.compat_opts.borrow_mut().proton_no_esync = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "PROTON_USE_FSYNC",
                "Disable futex-based synchronization (FSync).",
                o.proton_use_fsync,
                |on| {
                    self.compat_opts.borrow_mut().proton_no_fsync = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "PROTON_USE_NTSYNC",
                "Disable NTSync.",
                o.proton_use_ntsync,
                |on| {
                    self.compat_opts.borrow_mut().proton_no_ntsync = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "PROTON_NO_ESYNC",
                "Disable futex-based synchronization (ESync).",
                o.proton_no_esync,
                |on| {
                    self.compat_opts.borrow_mut().proton_no_esync = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "PROTON_NO_FSYNC",
                "Disable futex-based synchronization (FSync).",
                o.proton_no_fsync,
                |on| {
                    self.compat_opts.borrow_mut().proton_no_fsync = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "PROTON_NO_NTSYNC",
                "Disable NTSync.",
                o.proton_no_ntsync,
                |on| {
                    self.compat_opts.borrow_mut().proton_no_ntsync = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "PROTON_ENABLE_NVAPI",
                "Enable NVIDIA's NVAPI GPU support library.",
                o.proton_enable_nvapi,
                |on| {
                    self.compat_opts.borrow_mut().proton_enable_nvapi = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "PROTON_USE_SECCOMP",
                "Enable a seccomp-bpf filter to emulate native syscalls, required for some DRM protections to work.",
                o.proton_use_seccomp,
                |on| {
                    self.compat_opts.borrow_mut().proton_use_seccomp = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "PROTON_USE_SDL",
                "Use SDL input instead of HIDRAW/Steam Input.",
                o.proton_use_sdl,
                |on| {
                    self.compat_opts.borrow_mut().proton_use_sdl = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "PROTON_ENABLE_WAYLAND",
                "Enable the Wine Wayland driver (alias of PROTON_USE_WAYLAND).",
                o.proton_enable_wayland,
                |on| {
                    self.compat_opts.borrow_mut().proton_enable_wayland = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "PROTON_USE_WAYLAND",
                "Enable the Wine Wayland driver.",
                o.proton_use_wayland,
                |on| {
                    self.compat_opts.borrow_mut().proton_use_wayland = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "PROTON_USE_XALIA",
                "Enable the Xalia gamepad UI.",
                o.proton_use_xalia,
                |on| {
                    self.compat_opts.borrow_mut().proton_use_xalia = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_input_row(
                "Misc Args",
                "Any arbitrary args, e.g. PROTON_LOG=1.",
                "e.g. PROTON_LOG=1",
                &o.proton_args,
                |v| {
                    self.compat_opts.borrow_mut().proton_args = v;
                    Message::CompatOptsChanged
                }
            ),
        ]
        .spacing(12)
        .into()
    }

    /// The Wine section rows.
    fn wine_rows_section(&self) -> Element<'_, Message> {
        let o = self.compat_opts.borrow();

        column![
            compat_input_row(
                "WINEPREFIX",
                "Wine prefix directory for the game.",
                "/path/to/prefix",
                &o.wine_prefix,
                |v| {
                    self.compat_opts.borrow_mut().wine_prefix = v;
                    Message::CompatOptsChanged
                }
            ),
            compat_input_row(
                "WINEDLLOVERRIDES",
                "Control builtin vs native DLL selection.",
                "e.g. msvcp140=n,nvapi=b",
                &o.wine_dll_overrides,
                |v| {
                    self.compat_opts.borrow_mut().wine_dll_overrides = v;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "WINEESYNC",
                "Legacy control of ESync.",
                o.wine_esync,
                |on| {
                    self.compat_opts.borrow_mut().wine_esync = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "WINEFSYNC",
                "Legacy control of FSync.",
                o.wine_fsync,
                |on| {
                    self.compat_opts.borrow_mut().wine_fsync = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "WINE_FULLSCREEN_FSR",
                "AMD FSR upscaling for fullscreen games.",
                o.wine_fullscreen_fsr,
                |on| {
                    self.compat_opts.borrow_mut().wine_fullscreen_fsr = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "WINE_LARGE_ADDRESS_AWARE",
                "This environment variable can prevent 32bit games running through dxvk from crashing after reaching 4gb of RAM.",
                o.wine_large_address_aware,
                |on| {
                    self.compat_opts.borrow_mut().wine_large_address_aware = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_input_row(
                "Misc Args",
                "Any arbitrary args, e.g. WINEDEBUG=-all.",
                "e.g. WINEDEBUG=-all",
                &o.wine_args,
                |v| {
                    self.compat_opts.borrow_mut().wine_args = v;
                    Message::CompatOptsChanged
                }
            ),
        ]
        .spacing(12)
        .into()
    }

    /// The DXVK section rows.
    fn dxvk_rows_section(&self) -> Element<'_, Message> {
        let o = self.compat_opts.borrow();

        column![
            compat_flag_row(
                "DXVK_HUD",
                "Show a HUD with FPS and GPU information.",
                o.dxvk_hud,
                |on| {
                    self.compat_opts.borrow_mut().dxvk_hud = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_input_row(
                "Misc Args",
                "Any arbitrary args. KEY=VALUE tokens become environment variables, other tokens are appended to the command.",
                "e.g. DXVK_LOG_LEVEL=info",
                &o.dxvk_args,
                |v| {
                    self.compat_opts.borrow_mut().dxvk_args = v;
                    Message::CompatOptsChanged
                }
            ),
        ]
        .spacing(12)
        .into()
    }

    /// The Gamemode section rows.
    fn gamemode_rows_section(&self) -> Element<'_, Message> {
        let o = self.compat_opts.borrow();

        column![
            compat_flag_row(
                "Enable",
                "Run the game through gamemoderun.",
                o.gamemoderun,
                |on| {
                    self.compat_opts.borrow_mut().gamemoderun = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_input_row(
                "Misc Args",
                "Any arbitrary args passed to the gamemoderun wrapper.",
                "e.g. -a 300",
                &o.gamemode_args,
                |v| {
                    self.compat_opts.borrow_mut().gamemode_args = v;
                    Message::CompatOptsChanged
                }
            ),
        ]
        .spacing(12)
        .into()
    }

    /// The Gamescope section, including the grouped resolution row
    /// (w / h / r).
    fn gamescope_rows_section(&self) -> Element<'_, Message> {
        let o = self.compat_opts.borrow();

        column![
            compat_flag_row(
                "Enable",
                "Run the game inside a Gamescope session.",
                o.gamescope_enable,
                |on| {
                    self.compat_opts.borrow_mut().gamescope_enable = on;
                    Message::CompatOptsChanged
                }
            ),
            self.compat_resolution_row(),
            compat_flag_row(
                "Enable HDR",
                "Enable HDR output (requires a Wayland compositor, game and monitor with HDR support).",
                o.gamescope_hdr,
                |on| {
                    self.compat_opts.borrow_mut().gamescope_hdr = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "Expose Wayland",
                "Pass the Wayland compositor through to the game.",
                o.gamescope_expose_wayland,
                |on| {
                    self.compat_opts.borrow_mut().gamescope_expose_wayland = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "VRR support",
                "Enable adaptive synchronization for displays with variable refresh rate support.",
                o.gamescope_vrr,
                |on| {
                    self.compat_opts.borrow_mut().gamescope_vrr = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "Force grab cursor",
                "Keep the mouse cursor inside the Gamescope window.",
                o.gamescope_force_grab_cursor,
                |on| {
                    self.compat_opts.borrow_mut().gamescope_force_grab_cursor = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_input_row(
                "Misc Args",
                "Any arbitrary args passed to Gamescope, e.g. -F or -O 0x... .",
                "e.g. --fullscreen",
                &o.gamescope_args,
                |v| {
                    self.compat_opts.borrow_mut().gamescope_args = v;
                    Message::CompatOptsChanged
                }
            ),
        ]
        .spacing(12)
        .into()
    }

    /// The gamescope resolution row: three inputs side by side (w / h / r).
    fn compat_resolution_row(&self) -> Element<'_, Message> {
        let o = self.compat_opts.borrow();

        column![
            text("Resolution").size(14),
            text("Gamescope window resolution and refresh rate.")
                .size(12)
                .color(Color::from_rgb8(140, 140, 140)),
            row![
                compat_resolution_part("w", "WIDTH", &o.gamescope_width, |v| {
                    self.compat_opts.borrow_mut().gamescope_width = v;
                    Message::CompatOptsChanged
                }),
                compat_resolution_part("h", "HEIGHT", &o.gamescope_height, |v| {
                    self.compat_opts.borrow_mut().gamescope_height = v;
                    Message::CompatOptsChanged
                }),
                compat_resolution_part("r", "REFRESH (Hz)", &o.gamescope_refresh, |v| {
                    self.compat_opts.borrow_mut().gamescope_refresh = v;
                    Message::CompatOptsChanged
                }),
            ]
            .spacing(8)
        ]
        .spacing(4)
        .into()
    }

    /// The Misc section rows.
    fn misc_rows_section(&self) -> Element<'_, Message> {
        let o = self.compat_opts.borrow();

        column![
            compat_flag_row(
                "MANGOHUD",
                "Show the MangoHud performance overlay.",
                o.mango_hud,
                |on| {
                    self.compat_opts.borrow_mut().mango_hud = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_input_row(
                "Misc Args",
                "Any arbitrary args, e.g. MANGOHUD_CONFIG=fps,cpu,gpu.",
                "e.g. MANGOHUD_CONFIG=fps,cpu",
                &o.misc_args,
                |v| {
                    self.compat_opts.borrow_mut().misc_args = v;
                    Message::CompatOptsChanged
                }
            ),
        ]
        .spacing(12)
        .into()
    }
}

/// A checkbox row bound to one bool field of `CompatOptions`.
fn compat_flag_row<'a>(
    name: &'static str,
    description: &'static str,
    checked: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        checkbox(checked)
            .label(name)
            .on_toggle(on_toggle)
            .text_size(14),
        text(description)
            .size(12)
            .color(Color::from_rgb8(140, 140, 140)),
    ]
    .spacing(2)
    .into()
}

/// A free-form text row bound to one String field of `CompatOptions`.
fn compat_input_row<'a>(
    name: &'static str,
    description: &'static str,
    placeholder: &'static str,
    value: &str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        text(name).size(14),
        text(description)
            .size(12)
            .color(Color::from_rgb8(140, 140, 140)),
        text_input(placeholder, value)
            .on_input(on_input)
            .size(14)
            .padding(8),
    ]
    .spacing(4)
    .into()
}

/// One input of the Gamescope resolution row (w / h / r).
fn compat_resolution_part<'a>(
    label: &'static str,
    placeholder: &'static str,
    value: &str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        text(label).size(12).color(Color::from_rgb8(140, 140, 140)),
        text_input(placeholder, value)
            .on_input(on_input)
            .size(14)
            .padding(8),
    ]
    .spacing(2)
    .width(Length::Fill)
    .into()
}

/// Section label for the Proton/Wine options page, preceded by a divider
/// line that separates the sections.
fn compat_section_header(label: &str) -> Element<'_, Message> {
    iced::widget::column![
        Space::new().height(14),
        iced::widget::rule::horizontal(1).style(|_theme: &iced::Theme| iced::widget::rule::Style {
            color: Color::from_rgba8(255, 255, 255, 0.15),
            radius: 0.0.into(),
            fill_mode: iced::widget::rule::FillMode::Padded(8),
            snap: true,
        }),
        Space::new().height(14),
        text(label).size(20),
        Space::new().height(14),
    ]
    .into()
}
