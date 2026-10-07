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
    ProtonLog,
    ProtonLogDir,
    ProtonCrashReportDir,
    ProtonWaitAttach,
    ProtonUseWined3d,
    ProtonNoD3D11,
    ProtonNoD3D10,
    ProtonDxvkD3D8,
    ProtonNoFsync,
    ProtonNoNtsync,
    ProtonDisableNvapi,
    ProtonForceLargeAddressAware,
    ProtonHeapDelayFree,
    ProtonUseXalia,
    HostLcAll,
    Fna3dForceDriver,

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
    DxvkLogLevel,
    DxvkLogPath,
    DxvkConfigFile,
    DxvkConfig,
    DxvkFilterDeviceName,
    DxvkFilterDeviceUuid,
    DxvkDebug,
    DxvkShaderCacheDisabled,
    DxvkShaderCachePath,

    // Gamemode / Gamescope
    GamemodeAuto,
    GamescopeWsi,
    GamescopeFsrStrength,
    MangoHud,
}

/// The widget used to edit one variable on the Proton/Wine variables page:
/// a checkbox for variables that are either 1 or 0, or a text input for
/// everything else.
#[derive(Clone, Copy, Debug)]
pub enum EnvRowKind {
    Flag,
    /// Text input with a placeholder shown while the field is empty.
    Text(&'static str),
}#[derive(Clone, Debug)]
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
        let intro = concat!(
            "These environment variables are passed to the compatibility layer when launching ",
            "the game. Checked options are set to 1; every other field is passed as entered. ",
            "Leave a field empty to leave the variable unset."
        );

        let content = column![
            text("Proton/Wine Variables")
                .size(18)
                .font(styles::fonts::BOLD),
            text(intro).size(12).color(Color::from_rgb8(140, 140, 140)),
            env_section_header("Proton"),
            self.env_rows_section(&PROTON_ROWS),
            env_section_header("Wine"),
            self.env_rows_section(&WINE_ROWS),
            env_section_header("DXVK"),
            self.env_rows_section(&DXVK_ROWS),
            env_section_header("Gamemode/Gamescope"),
            self.env_rows_section(&GAMEMODE_GAMESCOPE_ROWS),
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
                EnvRowKind::Flag => section = section.push(
                    column![
                        checkbox(self.env_flag(*key))
                            .label(name)
                            .on_toggle(|checked| Message::EnvVarToggled(*key, checked))
                            .text_size(14),
                        text(description).size(12).color(Color::from_rgb8(140, 140, 140)),
                    ]
                    .spacing(2),
                ),
                EnvRowKind::Text(placeholder) => section = section.push(
                    column![
                        text(name).size(14),
                        text(description).size(12).color(Color::from_rgb8(140, 140, 140)),
                        text_input(placeholder, self.env_text(*key))
                            .on_input(|value| Message::EnvVarChanged(*key, value))
                            .size(14)
                            .padding(8),
                    ]
                    .spacing(4),
                ),
            }
        }

        section.into()
    }
}

/// Section label for the environment variables page, matching the gray
/// headers used across the modals.
fn env_section_header(label: &str) -> Element<'_, Message> {
    text(label)
        .size(13)
        .color(Color::from_rgb8(140, 140, 140))
        .into()
}

/// Rows of the Proton section of the Proton/Wine variables page. Each row is
/// (variable key, widget, displayed name, description).
const PROTON_ROWS: &[(EnvVarKey, EnvRowKind, &str, &str)] = &[
    (
        EnvVarKey::ProtonLog,
        EnvRowKind::Flag,
        "PROTON_LOG",
        "Create a Proton/Wine diagnostic log.",
    ),
    (
        EnvVarKey::ProtonLogDir,
        EnvRowKind::Text("/path/to/log/dir"),
        "PROTON_LOG_DIR",
        "Change where Proton writes its log.",
    ),
    (
        EnvVarKey::ProtonCrashReportDir,
        EnvRowKind::Text("/path/to/crash/reports"),
        "PROTON_CRASH_REPORT_DIR",
        "Write Proton crash reports to this directory.",
    ),
    (
        EnvVarKey::ProtonWaitAttach,
        EnvRowKind::Flag,
        "PROTON_WAIT_ATTACH",
        "Wait for a debugger to attach before launching (debugging).",
    ),
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
        EnvVarKey::ProtonForceLargeAddressAware,
        EnvRowKind::Flag,
        "PROTON_FORCE_LARGE_ADDRESS_AWARE",
        "Force LARGE_ADDRESS_AWARE (enabled by default).",
    ),
    (
        EnvVarKey::ProtonHeapDelayFree,
        EnvRowKind::Flag,
        "PROTON_HEAP_DELAY_FREE",
        "Delay freeing heap memory to work around certain memory/use-after-free bugs.",
    ),
    (
        EnvVarKey::ProtonUseXalia,
        EnvRowKind::Flag,
        "PROTON_USE_XALIA",
        "Enable the Xalia gamepad UI.",
    ),
    (
        EnvVarKey::HostLcAll,
        EnvRowKind::Text("e.g. en_US.UTF-8"),
        "HOST_LC_ALL",
        "Override the locale used for the game.",
    ),
    (
        EnvVarKey::Fna3dForceDriver,
        EnvRowKind::Text("D3D11 or OpenGL"),
        "FNA3D_FORCE_DRIVER",
        "Force the FNA3D renderer (game-specific).",
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
        EnvVarKey::DxvkLogLevel,
        EnvRowKind::Text("none, error, warn or info"),
        "DXVK_LOG_LEVEL",
        "DXVK logging level.",
    ),
    (
        EnvVarKey::DxvkLogPath,
        EnvRowKind::Text("/path/to/logs"),
        "DXVK_LOG_PATH",
        "Directory DXVK log files are written to.",
    ),
    (
        EnvVarKey::DxvkConfigFile,
        EnvRowKind::Text("/path/to/dxvk.conf"),
        "DXVK_CONFIG_FILE",
        "Configuration file for DXVK.",
    ),
    (
        EnvVarKey::DxvkConfig,
        EnvRowKind::Text("e.g. dxgi.syncInterval = 0"),
        "DXVK_CONFIG",
        "Configure DXVK directly through the environment.",
    ),
    (
        EnvVarKey::DxvkFilterDeviceName,
        EnvRowKind::Text("e.g. NVIDIA GeForce RTX 3080"),
        "DXVK_FILTER_DEVICE_NAME",
        "Select a graphics card by name.",
    ),
    (
        EnvVarKey::DxvkFilterDeviceUuid,
        EnvRowKind::Text("Device UUID"),
        "DXVK_FILTER_DEVICE_UUID",
        "Select a graphics card by UUID.",
    ),
    (
        EnvVarKey::DxvkDebug,
        EnvRowKind::Text("e.g. marker"),
        "DXVK_DEBUG",
        "DXVK debug switches (development).",
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

/// Rows of the Gamemode/Gamescope section of the Proton/Wine variables page.
const GAMEMODE_GAMESCOPE_ROWS: &[(EnvVarKey, EnvRowKind, &str, &str)] = &[
    (
        EnvVarKey::GamemodeAuto,
        EnvRowKind::Flag,
        "GAMEMODEAUTO",
        "Enable Feral GameMode while the game runs.",
    ),
    (
        EnvVarKey::GamescopeWsi,
        EnvRowKind::Flag,
        "ENABLE_GAMESCOPE_WSI",
        "Enable the Gamescope Vulkan WSI layer (for Gamescope sessions).",
    ),
    (
        EnvVarKey::GamescopeFsrStrength,
        EnvRowKind::Text("0-5"),
        "GAMESCOPE_FSR_STRENGTH",
        "FSR sharpening strength used by Gamescope.",
    ),
    (
        EnvVarKey::MangoHud,
        EnvRowKind::Flag,
        "MANGOHUD",
        "Show the MangoHud performance overlay.",
    ),
];
