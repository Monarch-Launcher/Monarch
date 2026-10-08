use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use iced::widget::{
    button, checkbox, column, combo_box, container, row, scrollable, text, text_input, Space,
};
use iced::{alignment, border, Color, Element, Length, Task};
use monarch_core::monarch_utils::monarch_state::MonarchState;
use tracing::error;

use crate::gui::{show_error, styles};
use monarch_core::monarch_games::commands::{
    get_executables, proton_versions, update_game_properties,
};
use monarch_core::monarch_games::monarchgame::MonarchGame;
use monarch_core::monarch_utils::monarch_vdf::ProtonVersion;

/// Identifies one of the environment variables editable on the Proton/Wine
/// variables page. The page only collects the values; applying them to the
/// launched game is not wired up yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnvVarKey {
    // Proton
    ProtonUseWined3d,
    ProtonNoD3D11,
    ProtonNoD3D10,
    ProtonDxvkD3D8,
    ProtonNoFsync,
    ProtonNoNtsync,
    ProtonDisableNvapi,
    ProtonEnableNvapi,
    ProtonUseSeccomp,
    ProtonUseSdl,
    ProtonPreferSdl,
    ProtonUseWayland,
    ProtonEnableWayland,
    ProtonUseXalia,

    // Wine
    WinePrefix,
    WineArch,
    WineDebug,
    WineDllOverrides,
    WineServer,
    WineLoader,
    WineDllPath,
    WineEsync,
    WineFsync,
    WineFullscreenFsr,
    WineFullscreenFsrStrength,
    WineFullscreenIntegerScaling,
    WineUseKwinHacks,

    // DXVK
    DxvkHud,
    DxvkHudCustom,
    DxvkConfig,
    DxvkShaderCacheDisabled,
    DxvkShaderCachePath,

    // Gamemode / Gamescope / Misc
    GamemodeAuto,
    MangoHud,
    GamescopeEnable,
    GamescopeWidth,
    GamescopeHeight,
    GamescopeRefresh,
    GamescopeHdr,
    GamescopeExposeWayland,
    GamescopeVrr,
    GamescopeForceGrabCursor,
    GamescopeWsi,
    GamescopeFsrStrength,
}

/// The widget used to edit one variable on the Proton/Wine variables page:
/// a checkbox for variables that are either 1 or 0, or a text input for
/// everything else.
#[derive(Clone, Copy, Debug)]
pub enum EnvRowKind {
    Flag,
    /// Text input with a placeholder shown while the field is empty.
    Text(&'static str),
}
#[derive(Clone, Debug)]
pub enum Message {
    ExecutablesLoaded(Vec<PathBuf>),
    CompatibilityLoaded(Vec<ProtonVersion>),
    ExecutableSelected(String),
    ExecutableHovered(String),
    CompatibilitySelected(ProtonVersion),
    LaunchArgsChanged(String),
    /// Open the Proton/Wine environment variables page.
    OpenEnvVars,
    /// Leave the environment variables page and return to launch options.
    EnvVarsBack,
    /// (Un)set one of the 0/1 variables on the environment variables page.
    EnvVarToggled(EnvVarKey, bool),
    /// Change the value of a free-form variable on that page.
    EnvVarChanged(EnvVarKey, String),
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

    /// Whether the Proton/Wine environment variables page is shown instead
    /// of the regular launch options fields.
    show_env_vars: bool,
    /// Values collected on the environment variables page: "1" for every
    /// checked flag, the entered text for free-form variables.
    env_vars: HashMap<EnvVarKey, String>,

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

            show_env_vars: false,
            env_vars: HashMap::new(),

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
            Message::OpenEnvVars => {
                self.show_env_vars = true;
                Task::none()
            }
            Message::EnvVarsBack => {
                self.show_env_vars = false;
                Task::none()
            }
            Message::EnvVarToggled(key, checked) => {
                if checked {
                    self.env_vars.insert(key, "1".to_string());
                } else {
                    self.env_vars.remove(&key);
                }
                Task::none()
            }
            Message::EnvVarChanged(key, value) => {
                self.env_vars.insert(key, value);
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

    /// Whether the Proton/Wine environment variables page is currently
    /// shown instead of the regular launch-option fields.
    pub fn showing_env_vars(&self) -> bool {
        self.show_env_vars
    }

    /// The editable property fields (without footer or modal wrapper); used
    /// both by `view` and by the combined edit modal on the details page.
    pub fn fields(&self) -> Element<'_, Message> {
        if self.show_env_vars {
            return self.env_vars_fields();
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

        let launch_args_input = text_input("Custom Launch Arguments", &self.launch_args)
            .on_input(Message::LaunchArgsChanged)
            .padding(10);

        let open_env_vars_button = button(
            row![
                text("Set Proton/Wine Variables").size(15),
                Space::new().width(Length::Fill),
            ]
            .align_y(alignment::Vertical::Center),
        )
        .on_press(Message::OpenEnvVars)
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
            open_env_vars_button,
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

    /// The Proton/Wine environment variables editor, shown as its own page
    /// in place of the regular launch-option fields.
    fn env_vars_fields(&self) -> Element<'_, Message> {
        let content = column![
            text("Proton/Wine Variables")
                .size(20)
                .font(styles::fonts::BOLD),
            env_section_header("Proton"),
            self.env_rows_section(&PROTON_ROWS),
            env_section_header("Wine"),
            self.env_rows_section(&WINE_ROWS),
            env_section_header("DXVK"),
            self.env_rows_section(&DXVK_ROWS),
            env_section_header("Gamemode"),
            self.env_rows_section(&GAMEMODE_ROWS),
            env_section_header("Gamescope"),
            self.env_gamescope_section(),
            env_section_header("Misc"),
            self.env_rows_section(&MISC_ROWS),
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

    /// Whether a 0/1 variable is currently checked on the variables page.
    fn env_flag(&self, key: EnvVarKey) -> bool {
        self.env_vars.get(&key).map(|v| v == "1").unwrap_or(false)
    }

    /// The text currently stored for a free-form variable.
    fn env_text(&self, key: EnvVarKey) -> &str {
        self.env_vars
            .get(&key)
            .map(String::as_str)
            .unwrap_or_default()
    }

    /// One section of rows for the environment variables page.
    fn env_rows_section(
        &self,
        rows: &'static [(EnvVarKey, EnvRowKind, &'static str, &'static str)],
    ) -> Element<'_, Message> {
        let mut section = column![].spacing(12);

        for (key, kind, name, description) in rows {
            let name = *name;
            let description = *description;
            match kind {
                EnvRowKind::Flag => {
                    section =
                        section.push(env_flag_row(self.env_flag(*key), name, description, *key))
                }
                EnvRowKind::Text(placeholder) => {
                    section = section.push(env_input_row(
                        name,
                        description,
                        placeholder,
                        self.env_text(*key),
                        *key,
                    ))
                }
            }
        }

        section.into()
    }

    /// The Gamescope section, separate from the generic row tables because of
    /// the grouped resolution row (w / h / r).
    fn env_gamescope_section(&self) -> Element<'_, Message> {
        let section = column![
            env_flag_row(
                self.env_flag(EnvVarKey::GamescopeEnable),
                "Enable",
                "Run the game inside a Gamescope session.",
                EnvVarKey::GamescopeEnable
            ),
            self.env_resolution_row(),
            env_flag_row(
                self.env_flag(EnvVarKey::GamescopeHdr),
                "Enable HDR",
                "Enable HDR output (requires a Wayland compositor, game and monitor with HDR support).",
                EnvVarKey::GamescopeHdr
            ),
            env_flag_row(
                self.env_flag(EnvVarKey::GamescopeExposeWayland),
                "Expose Wayland",
                "Pass the Wayland compositor through to the game.",
                EnvVarKey::GamescopeExposeWayland
            ),
            env_flag_row(
                self.env_flag(EnvVarKey::GamescopeVrr),
                "VRR support",
                "Enable adaptive synchronization for displays with variable refresh rate support.",
                EnvVarKey::GamescopeVrr
            ),
            env_flag_row(
                self.env_flag(EnvVarKey::GamescopeForceGrabCursor),
                "Force grab cursor",
                "Keep the mouse cursor inside the Gamescope window.",
                EnvVarKey::GamescopeForceGrabCursor
            ),
            env_flag_row(
                self.env_flag(EnvVarKey::GamescopeWsi),
                "ENABLE_GAMESCOPE_WSI",
                "Enable the Gamescope Vulkan WSI layer (for Gamescope sessions).",
                EnvVarKey::GamescopeWsi
            ),
            env_input_row(
                "GAMESCOPE_FSR_STRENGTH",
                "FSR sharpening strength used by Gamescope.",
                "0-5",
                self.env_text(EnvVarKey::GamescopeFsrStrength),
                EnvVarKey::GamescopeFsrStrength
            ),
        ]
        .spacing(12);

        section.into()
    }

    /// The gamescope resolution row: three inputs side by side (w / h / r).
    fn env_resolution_row(&self) -> Element<'_, Message> {
        let part = |label: &'static str, placeholder: &'static str, key: EnvVarKey, value: &str| {
            column![
                text(label).size(12).color(Color::from_rgb8(140, 140, 140)),
                text_input(placeholder, value)
                    .on_input(move |val| Message::EnvVarChanged(key, val))
                    .size(14)
                    .padding(8),
            ]
            .spacing(2)
            .width(Length::Fill)
        };

        column![
            text("Resolution").size(14),
            text("Gamescope window resolution and refresh rate.")
                .size(12)
                .color(Color::from_rgb8(140, 140, 140)),
            row![
                part(
                    "w",
                    "WIDTH",
                    EnvVarKey::GamescopeWidth,
                    self.env_text(EnvVarKey::GamescopeWidth)
                ),
                part(
                    "h",
                    "HEIGHT",
                    EnvVarKey::GamescopeHeight,
                    self.env_text(EnvVarKey::GamescopeHeight)
                ),
                part(
                    "r",
                    "REFRESH (Hz)",
                    EnvVarKey::GamescopeRefresh,
                    self.env_text(EnvVarKey::GamescopeRefresh)
                ),
            ]
            .spacing(8)
        ]
        .spacing(4)
        .into()
    }
}

/// A checkbox row for the environment variables page.
fn env_flag_row(
    checked: bool,
    name: &'static str,
    description: &'static str,
    key: EnvVarKey,
) -> Element<'static, Message> {
    column![
        checkbox(checked)
            .label(name)
            .on_toggle(move |toggle| Message::EnvVarToggled(key, toggle))
            .text_size(14),
        text(description)
            .size(12)
            .color(Color::from_rgb8(140, 140, 140)),
    ]
    .spacing(2)
    .into()
}

/// A free-form text row for the environment variables page.
fn env_input_row<'a>(
    name: &'static str,
    description: &'static str,
    placeholder: &'static str,
    value: &'a str,
    key: EnvVarKey,
) -> Element<'a, Message> {
    column![
        text(name).size(14),
        text(description)
            .size(12)
            .color(Color::from_rgb8(140, 140, 140)),
        text_input(placeholder, value)
            .on_input(move |input| Message::EnvVarChanged(key, input))
            .size(14)
            .padding(8),
    ]
    .spacing(4)
    .into()
}

/// Section label for the environment variables page, preceded by a divider
/// line that separates the sections.
fn env_section_header(label: &str) -> Element<'_, Message> {
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

/// Rows of the Proton section of the Proton/Wine variables page. Each row is
/// (variable key, widget, displayed name, description).
const PROTON_ROWS: &[(EnvVarKey, EnvRowKind, &str, &str)] = &[
    (
        EnvVarKey::ProtonUseWined3d,
        EnvRowKind::Flag,
        "PROTON_USE_WINED3D",
        "Use WineD3D/OpenGL instead of DXVK for D3D9/10/11.",
    ),
    (
        EnvVarKey::ProtonNoD3D11,
        EnvRowKind::Flag,
        "PROTON_NO_D3D11",
        "Disable D3D11.",
    ),
    (
        EnvVarKey::ProtonNoD3D10,
        EnvRowKind::Flag,
        "PROTON_NO_D3D10",
        "Disable D3D10/DXGI.",
    ),
    (
        EnvVarKey::ProtonDxvkD3D8,
        EnvRowKind::Flag,
        "PROTON_DXVK_D3D8",
        "Use DXVK's D3D8 implementation.",
    ),
    (
        EnvVarKey::ProtonNoFsync,
        EnvRowKind::Flag,
        "PROTON_NO_FSYNC",
        "Disable futex-based synchronization (FSync).",
    ),
    (
        EnvVarKey::ProtonNoNtsync,
        EnvRowKind::Flag,
        "PROTON_NO_NTSYNC",
        "Disable NTSync.",
    ),
    (
        EnvVarKey::ProtonDisableNvapi,
        EnvRowKind::Flag,
        "PROTON_DISABLE_NVAPI",
        "Disable NVIDIA NVAPI support.",
    ),
    (
        EnvVarKey::ProtonEnableNvapi,
        EnvRowKind::Flag,
        "PROTON_ENABLE_NVAPI",
        "Enable NVIDIA's NVAPI GPU support library.",
    ),
    (
        EnvVarKey::ProtonUseSeccomp,
        EnvRowKind::Flag,
        "PROTON_USE_SECCOMP",
        "Enable a seccomp-bpf filter to emulate native syscalls, required for some DRM protections to work.",
    ),
    (
        EnvVarKey::ProtonUseSdl,
        EnvRowKind::Flag,
        "PROTON_USE_SDL",
        "Use SDL input instead of HIDRAW/Steam Input.",
    ),
    (
        EnvVarKey::ProtonPreferSdl,
        EnvRowKind::Flag,
        "PROTON_PREFER_SDL",
        "Prefer SDL input over HIDRAW/Steam Input (alias of PROTON_USE_SDL).",
    ),
    (
        EnvVarKey::ProtonUseWayland,
        EnvRowKind::Flag,
        "PROTON_USE_WAYLAND",
        "Enable the Wine Wayland driver.",
    ),
    (
        EnvVarKey::ProtonEnableWayland,
        EnvRowKind::Flag,
        "PROTON_ENABLE_WAYLAND",
        "Enable the Wine Wayland driver (alias of PROTON_USE_WAYLAND).",
    ),
    (
        EnvVarKey::ProtonUseXalia,
        EnvRowKind::Flag,
        "PROTON_USE_XALIA",
        "Enable the Xalia gamepad UI.",
    ),
];

/// Rows of the Wine section of the Proton/Wine variables page.
const WINE_ROWS: &[(EnvVarKey, EnvRowKind, &str, &str)] = &[
    (
        EnvVarKey::WinePrefix,
        EnvRowKind::Text("/path/to/prefix"),
        "WINEPREFIX",
        "Wine prefix directory for the game.",
    ),
    (
        EnvVarKey::WineArch,
        EnvRowKind::Text("win32 or win64"),
        "WINEARCH",
        "Prefix architecture, used when a prefix is created.",
    ),
    (
        EnvVarKey::WineDebug,
        EnvRowKind::Text("e.g. +all or -all"),
        "WINEDEBUG",
        "Control Wine debugging output.",
    ),
    (
        EnvVarKey::WineDllOverrides,
        EnvRowKind::Text("e.g. msvcp140=n,nvapi=b"),
        "WINEDLLOVERRIDES",
        "Control builtin vs native DLL selection.",
    ),
    (
        EnvVarKey::WineServer,
        EnvRowKind::Text("/path/to/wineserver"),
        "WINESERVER",
        "Wineserver executable to use.",
    ),
    (
        EnvVarKey::WineLoader,
        EnvRowKind::Text("/path/to/wine"),
        "WINELOADER",
        "Wine loader executable to use.",
    ),
    (
        EnvVarKey::WineDllPath,
        EnvRowKind::Text("/path/to/dlls"),
        "WINEDLLPATH",
        "Extra DLL search paths.",
    ),
    (
        EnvVarKey::WineEsync,
        EnvRowKind::Flag,
        "WINEESYNC",
        "Legacy control of ESync.",
    ),
    (
        EnvVarKey::WineFsync,
        EnvRowKind::Flag,
        "WINEFSYNC",
        "Legacy control of FSync.",
    ),
    (
        EnvVarKey::WineFullscreenFsr,
        EnvRowKind::Flag,
        "WINE_FULLSCREEN_FSR",
        "AMD FSR upscaling for fullscreen games.",
    ),
    (
        EnvVarKey::WineFullscreenFsrStrength,
        EnvRowKind::Text("0-5"),
        "WINE_FULLSCREEN_FSR_STRENGTH",
        "FSR sharpening strength.",
    ),
    (
        EnvVarKey::WineFullscreenIntegerScaling,
        EnvRowKind::Flag,
        "WINE_FULLSCREEN_INTEGER_SCALING",
        "Integer scaling for fullscreen games.",
    ),
    (
        EnvVarKey::WineUseKwinHacks,
        EnvRowKind::Flag,
        "WINE_USE_KWIN_HACKS",
        "KDE-specific window-management workarounds.",
    ),
];

/// Rows of the DXVK section of the Proton/Wine variables page.
const DXVK_ROWS: &[(EnvVarKey, EnvRowKind, &str, &str)] = &[
    (
        EnvVarKey::DxvkHud,
        EnvRowKind::Flag,
        "DXVK_HUD",
        "Show a HUD with FPS and GPU information.",
    ),
    (
        EnvVarKey::DxvkHudCustom,
        EnvRowKind::Text("full or devinfo,fps,frametimes"),
        "DXVK_HUD value",
        "Custom HUD contents; overrides the simple checkbox when set.",
    ),
    (
        EnvVarKey::DxvkConfig,
        EnvRowKind::Text("e.g. dxgi.syncInterval = 0"),
        "DXVK_CONFIG",
        "Configure DXVK directly through the environment.",
    ),
    (
        EnvVarKey::DxvkShaderCacheDisabled,
        EnvRowKind::Flag,
        "DXVK_SHADER_CACHE off",
        "Disable DXVK's internal shader cache (sets the variable to 0).",
    ),
    (
        EnvVarKey::DxvkShaderCachePath,
        EnvRowKind::Text("/path/to/cache"),
        "DXVK_SHADER_CACHE_PATH",
        "Directory for the DXVK shader cache.",
    ),
];

/// Rows of the Gamemode section of the Proton/Wine variables page.
const GAMEMODE_ROWS: &[(EnvVarKey, EnvRowKind, &str, &str)] = &[(
    EnvVarKey::GamemodeAuto,
    EnvRowKind::Flag,
    "Enable",
    "Run the game through gamemoderun.",
)];

/// Rows of the Misc section of the Proton/Wine variables page.
const MISC_ROWS: &[(EnvVarKey, EnvRowKind, &str, &str)] = &[(
    EnvVarKey::MangoHud,
    EnvRowKind::Flag,
    "MANGOHUD",
    "Show the MangoHud performance overlay.",
)];
