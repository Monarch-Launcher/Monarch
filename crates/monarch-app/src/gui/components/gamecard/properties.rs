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
    CompatibilityLoaded(Vec<ProtonVersion>),
    ExecutableSelected(String),
    ExecutableHovered(String),
    CompatibilitySelected(ProtonVersion),
    LaunchArgsChanged(String),
    /// Open the Proton/Wine options page.
    OpenCompatOpts,
    /// Leave the Proton/Wine options page and return to launch options.
    CompatOptsBack,
    /// An option was mutated in place from its row's callback; this message
    /// only makes the runtime redraw with the fresh values.
    CompatOptsChanged,
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
        let (launch_args, current_executable, compat_opts_opt) = {
            let game_lock = game.read().unwrap();
            (
                game_lock.launch_args.clone(),
                game_lock.executable_path.clone(),
                game_lock.compatibility_opts.clone(),
            )
        };

        let compat_opts = compat_opts_opt.unwrap_or_default();

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
            Message::OpenCompatOpts => {
                self.show_compat_opts = true;
                Task::none()
            }
            Message::CompatOptsBack => {
                self.show_compat_opts = false;
                Task::none()
            }
            Message::CompatOptsChanged => Task::none(),
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

                        game.compatibility_opts = Some(self.compat_opts.borrow().clone());
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

        let launch_args_input = text_input(
            "Custom Launch Arguments",
            &view_launch_command(
                self.game.clone(),
                &self.selected_executable.as_ref().unwrap_or(&"".to_string()),
            )
            .unwrap_or_default(),
        )
        .on_input(Message::LaunchArgsChanged)
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
                text("Launch Arguments").size(18),
                launch_args_input,
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
                text("Launch Arguments").size(18),
                launch_args_input,
            ]
            .spacing(10)
            .into()
        }
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
                "PROTON_DXVK_D3D8",
                "Use DXVK's D3D8 implementation.",
                o.proton_dxvk_d3d8,
                |on| {
                    self.compat_opts.borrow_mut().proton_dxvk_d3d8 = on;
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
                "PROTON_DISABLE_NVAPI",
                "Disable NVIDIA NVAPI support.",
                o.proton_disable_nvapi,
                |on| {
                    self.compat_opts.borrow_mut().proton_disable_nvapi = on;
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
                "PROTON_PREFER_SDL",
                "Prefer SDL input over HIDRAW/Steam Input (alias of PROTON_USE_SDL).",
                o.proton_prefer_sdl,
                |on| {
                    self.compat_opts.borrow_mut().proton_prefer_sdl = on;
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
                "PROTON_ENABLE_WAYLAND",
                "Enable the Wine Wayland driver (alias of PROTON_USE_WAYLAND).",
                o.proton_enable_wayland,
                |on| {
                    self.compat_opts.borrow_mut().proton_enable_wayland = on;
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
                "WINEARCH",
                "Prefix architecture, used when a prefix is created.",
                "win32 or win64",
                &o.wine_arch,
                |v| {
                    self.compat_opts.borrow_mut().wine_arch = v;
                    Message::CompatOptsChanged
                }
            ),
            compat_input_row(
                "WINEDEBUG",
                "Control Wine debugging output.",
                "e.g. +all or -all",
                &o.wine_debug,
                |v| {
                    self.compat_opts.borrow_mut().wine_debug = v;
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
            compat_input_row(
                "WINESERVER",
                "Wineserver executable to use.",
                "/path/to/wineserver",
                &o.wine_server,
                |v| {
                    self.compat_opts.borrow_mut().wine_server = v;
                    Message::CompatOptsChanged
                }
            ),
            compat_input_row(
                "WINELOADER",
                "Wine loader executable to use.",
                "/path/to/wine",
                &o.wine_loader,
                |v| {
                    self.compat_opts.borrow_mut().wine_loader = v;
                    Message::CompatOptsChanged
                }
            ),
            compat_input_row(
                "WINEDLLPATH",
                "Extra DLL search paths.",
                "/path/to/dlls",
                &o.wine_dll_path,
                |v| {
                    self.compat_opts.borrow_mut().wine_dll_path = v;
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
            compat_input_row(
                "WINE_FULLSCREEN_FSR_STRENGTH",
                "FSR sharpening strength.",
                "0-5",
                &o.wine_fullscreen_fsr_strength,
                |v| {
                    self.compat_opts.borrow_mut().wine_fullscreen_fsr_strength = v;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "WINE_FULLSCREEN_INTEGER_SCALING",
                "Integer scaling for fullscreen games.",
                o.wine_fullscreen_integer_scaling,
                |on| {
                    self.compat_opts
                        .borrow_mut()
                        .wine_fullscreen_integer_scaling = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "WINE_USE_KWIN_HACKS",
                "KDE-specific window-management workarounds.",
                o.wine_use_kwin_hacks,
                |on| {
                    self.compat_opts.borrow_mut().wine_use_kwin_hacks = on;
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
                "DXVK_HUD value",
                "Custom HUD contents; overrides the simple checkbox when set.",
                "full or devinfo,fps,frametimes",
                &o.dxvk_hud_custom,
                |v| {
                    self.compat_opts.borrow_mut().dxvk_hud_custom = v;
                    Message::CompatOptsChanged
                }
            ),
            compat_input_row(
                "DXVK_CONFIG",
                "Configure DXVK directly through the environment.",
                "e.g. dxgi.syncInterval = 0",
                &o.dxvk_config,
                |v| {
                    self.compat_opts.borrow_mut().dxvk_config = v;
                    Message::CompatOptsChanged
                }
            ),
            compat_flag_row(
                "DXVK_SHADER_CACHE off",
                "Disable DXVK's internal shader cache (sets the variable to 0).",
                o.dxvk_shader_cache_disabled,
                |on| {
                    self.compat_opts.borrow_mut().dxvk_shader_cache_disabled = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_input_row(
                "DXVK_SHADER_CACHE_PATH",
                "Directory for the DXVK shader cache.",
                "/path/to/cache",
                &o.dxvk_shader_cache_path,
                |v| {
                    self.compat_opts.borrow_mut().dxvk_shader_cache_path = v;
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

        column![compat_flag_row(
            "Enable",
            "Run the game through gamemoderun.",
            o.gamemode_auto,
            |on| {
                self.compat_opts.borrow_mut().gamemode_auto = on;
                Message::CompatOptsChanged
            }
        ),]
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
            compat_flag_row(
                "ENABLE_GAMESCOPE_WSI",
                "Enable the Gamescope Vulkan WSI layer (for Gamescope sessions).",
                o.gamescope_wsi,
                |on| {
                    self.compat_opts.borrow_mut().gamescope_wsi = on;
                    Message::CompatOptsChanged
                }
            ),
            compat_input_row(
                "GAMESCOPE_FSR_STRENGTH",
                "FSR sharpening strength used by Gamescope.",
                "0-5",
                &o.gamescope_fsr_strength,
                |v| {
                    self.compat_opts.borrow_mut().gamescope_fsr_strength = v;
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

        column![compat_flag_row(
            "MANGOHUD",
            "Show the MangoHud performance overlay.",
            o.mango_hud,
            |on| {
                self.compat_opts.borrow_mut().mango_hud = on;
                Message::CompatOptsChanged
            }
        ),]
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
