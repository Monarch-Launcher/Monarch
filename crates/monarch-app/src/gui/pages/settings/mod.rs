use iced::{Element, Theme};
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use tracing::error;

use crate::gui::components::common::error_view;
use crate::gui::{self, show_confirm, show_error, AppMessage};
use monarch_core::monarch_utils::monarch_settings::Settings;
use monarch_core::{monarch_games, monarch_utils};

mod update;
mod view;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    Monarch,
    Steam,
    EpicGames,
    Gog,
    ItchIo,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    TabSelected(SettingsTab),
    ToggleQuickLaunch(bool),
    ToggleDownloadSpeedBits(bool),
    MaxDownloadSpeedChanged(String),
    MaxDownloadSpeedUnitSelected(SpeedPrefix),
    ToggleAutoUpdateCheck(bool),
    TogglePersistLibraryFilters(bool),
    LibraryFolderChanged(String),
    BrowseLibraryFolder,
    ClearCache,
    OpenLogs,
    RequestResetDefaults,
    ResetDefaults,
    ToggleSteam(bool),
    SteamUsernameChanged(String),
    SteamPasswordChanged(String),
    SaveSteamCredentials,
    DeleteSteamCredentials,
    SteamGuardSecretChanged(String),
    SaveSteamSecret,
    DeleteSteamSecret,
    ToggleEpic(bool),
    LoginEpic,
    DeleteEpicCredentials,
    EpicAuthCodeChanged(String),
    SaveEpicAuthCode,
    Refresh(()),
    OpenLink(&'static str),
    InstallSteamCMD,
    InstallSteamCMDLinuxWarning,
    InstallLegendary,
    InstallUmu,
    RemoveSteamCMD,
    RemoveLegendary,
    RemoveUmu,
    ToggleSteamHiddenPassword,
    ToggleHiddenSteamSecret,
    ToggleHiddenEpicToken,
    TestEpicFunctionality,
}

impl Message {
    fn requires_write_lock(&self) -> bool {
        match self {
            Message::ToggleQuickLaunch(_)
            | Message::ToggleDownloadSpeedBits(_)
            | Message::MaxDownloadSpeedChanged(_)
            | Message::MaxDownloadSpeedUnitSelected(_)
            | Message::ToggleAutoUpdateCheck(_)
            | Message::TogglePersistLibraryFilters(_)
            | Message::LibraryFolderChanged(_)
            | Message::ResetDefaults
            | Message::ToggleSteam(_)
            | Message::SaveSteamCredentials
            | Message::SaveSteamSecret
            | Message::DeleteSteamSecret
            | Message::DeleteSteamCredentials
            | Message::ToggleEpic(_)
            | Message::DeleteEpicCredentials
            | Message::InstallSteamCMD
            | Message::Refresh(_)
            | Message::ClearCache
            | Message::SaveEpicAuthCode => true,
            _ => false,
        }
    }
}

pub struct SettingsPage {
    current_tab: SettingsTab,
    shared_settings: Arc<RwLock<Settings>>,
    cache_size: u64,
    /// Text currently typed into the max download speed input.
    max_speed_tmp: String,
    /// Unit prefix picked next to the max speed input.
    max_speed_prefix: SpeedPrefix,
    steam_username_tmp: String,
    steam_password_tmp: String,
    view_steam_password: bool,
    view_steam_secret: bool,
    steam_secret_tmp: String,
    epic_auth_code_tmp: String,
    view_epic_token: bool,
}

impl Default for SettingsPage {
    fn default() -> Self {
        let shared_settings = monarch_utils::commands::get_settings().unwrap_or_default();
        let (steam_user, _epic_user, max_speed_tmp, max_speed_prefix) = match shared_settings.read()
        {
            Ok(s) => (
                s.steam.username.clone(),
                s.epic.username.clone(),
                format_speed_setting(s.monarch.max_download_speed_value),
                SpeedPrefix::from_setting(&s.monarch.max_download_speed_prefix),
            ),
            Err(_) => (
                String::new(),
                String::new(),
                String::new(),
                SpeedPrefix::Mega,
            ),
        };

        Self {
            current_tab: SettingsTab::Monarch,
            shared_settings,
            cache_size: monarch_utils::commands::get_cache_size().unwrap_or(0),
            max_speed_tmp,
            max_speed_prefix,
            steam_username_tmp: steam_user,
            steam_password_tmp: String::new(),
            view_steam_password: false,
            view_steam_secret: false,
            steam_secret_tmp: String::new(),
            epic_auth_code_tmp: String::new(),
            view_epic_token: false,
        }
    }
}

impl SettingsPage {
    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        let settings_ptr = self.shared_settings.clone();
        let mut write_guard: Option<RwLockWriteGuard<'_, Settings>> = None;

        if msg.requires_write_lock() {
            match settings_ptr.write() {
                Ok(settings) => write_guard = Some(settings),
                Err(e) => {
                    error!("gui::Settings::update() Failed to lock on shared_settings! | Err: {e}");
                    show_error("Failed to update settings!");
                    return iced::Task::none();
                }
            }
        }

        match msg {
            Message::TabSelected(tab) => self.current_tab = tab,
            Message::ToggleQuickLaunch(state) => {
                self.toggle_quicklaunch(&mut write_guard.unwrap(), state)
            }
            Message::ToggleDownloadSpeedBits(state) => {
                self.toggle_download_speed_bits(&mut write_guard.unwrap(), state)
            }
            Message::MaxDownloadSpeedChanged(value) => {
                self.change_max_download_speed(&mut write_guard.unwrap(), value)
            }
            Message::MaxDownloadSpeedUnitSelected(prefix) => {
                self.select_max_download_speed_unit(&mut write_guard.unwrap(), prefix)
            }
            Message::ToggleAutoUpdateCheck(state) => {
                self.toggle_auto_update_check(&mut write_guard.unwrap(), state)
            }
            Message::TogglePersistLibraryFilters(state) => {
                self.toggle_persist_library_filters(&mut write_guard.unwrap(), state)
            }
            Message::LibraryFolderChanged(folder) => {
                self.change_library_folder(&mut write_guard.unwrap(), &folder)
            }
            Message::ToggleSteam(state) => self.toggle_steam(&mut write_guard.unwrap(), state),
            Message::SteamUsernameChanged(u) => self.steam_username_tmp = u,
            Message::SteamPasswordChanged(p) => self.steam_password_tmp = p,
            Message::SaveSteamCredentials => {
                self.update_steam_credentials(&mut write_guard.unwrap())
            }
            Message::DeleteSteamCredentials => {
                self.delete_steam_credentials(&mut write_guard.unwrap())
            }
            Message::SteamGuardSecretChanged(s) => self.steam_secret_tmp = s,
            Message::SaveSteamSecret => self.update_steam_secret(&mut write_guard.unwrap()),
            Message::DeleteSteamSecret => self.remove_steam_secret(&mut write_guard.unwrap()),
            Message::ToggleEpic(state) => self.toggle_epic(&mut write_guard.unwrap(), state),
            Message::LoginEpic => self.login_epic(),
            Message::DeleteEpicCredentials => {
                self.delete_epic_credentials(&mut write_guard.unwrap())
            }
            Message::EpicAuthCodeChanged(code) => self.epic_auth_code_tmp = code,
            Message::SaveEpicAuthCode => self.login_epic_auth_code(&mut write_guard.unwrap()),
            Message::RequestResetDefaults => self.ask_reset_settings(),
            Message::ResetDefaults => self.reset_settings(&mut write_guard.unwrap()),
            Message::ClearCache => {
                monarch_utils::commands::clear_cached_images();
                self.refresh(&mut write_guard.unwrap());
            }
            Message::Refresh(_) => self.refresh(&mut write_guard.unwrap()),
            Message::OpenLogs => {
                let _ = monarch_utils::commands::open_logs();
            }
            Message::OpenLink(url) => self.open_link(url),
            Message::InstallUmu => self.install_umu_task(),
            Message::InstallSteamCMD => return self.install_steamcmd_task(&write_guard.unwrap()),
            Message::InstallSteamCMDLinuxWarning => show_confirm("Working with SteamCMD is a pain in the ass. That's why it's recommended to download and manage SteamCMD from you package manager.", AppMessage::Page(gui::pages::Message::Settings(Message::InstallSteamCMD))),
            Message::InstallLegendary => self.install_legendary_task(),
            Message::RemoveSteamCMD => self.remove_steamcmd(),
            Message::RemoveLegendary => self.remove_legendary(),
            Message::BrowseLibraryFolder => return self.pick_default_monarch_folder(),
            #[cfg(target_os = "linux")]
            Message::RemoveUmu => self.remove_umu(),
            #[cfg(not(target_os = "linux"))]
            Message::RemoveUmu => {}, // Do nothing
            Message::ToggleSteamHiddenPassword => {
                self.view_steam_password = !self.view_steam_password
            },
            Message::ToggleHiddenSteamSecret => {
                self.view_steam_secret = !self.view_steam_secret;
            },
            Message::ToggleHiddenEpicToken => {
                self.view_epic_token = !self.view_epic_token;
            }
            Message::TestEpicFunctionality => {
                let mut client = monarch_games::egs_client::EgsClient::new();
                futures::executor::block_on(client.load_existing_user()).unwrap();
                futures::executor::block_on(client.get_user_games());
            }
        }
        iced::Task::none()
    }

    pub fn view(&self) -> Element<'_, Message, Theme> {
        match self.shared_settings.read() {
            Ok(settings) => self.view_settings_page(&settings),
            Err(e) => {
                error!("gui::Settings::view() Failed to lock on shared_settings! | Err: {e}");
                error_view(
                    "Settings Unavailable",
                    "Failed to acquire a read lock on application settings. Try reloading this page.",
                    Some(Message::Refresh(())),
                )
            }
        }
    }
}

/// Unit prefix for the max download speed setting, stored as "k"/"m"/"g" in
/// settings.toml. Prefixes are decimal multiples of the unit base.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpeedPrefix {
    Kilo,
    Mega,
    Giga,
}

impl SpeedPrefix {
    fn as_str(self) -> &'static str {
        match self {
            SpeedPrefix::Kilo => "k",
            SpeedPrefix::Mega => "m",
            SpeedPrefix::Giga => "g",
        }
    }

    /// Parses a stored prefix string, falling back to Mega on unknown values.
    fn from_setting(value: &str) -> Self {
        match value {
            "k" => SpeedPrefix::Kilo,
            "g" => SpeedPrefix::Giga,
            _ => SpeedPrefix::Mega,
        }
    }
}

/// Pick-list entry pairing a unit prefix with the current bit/byte base, so
/// its label adapts ("kb/s", "mb/s", "gb/s" in bits mode vs "KB/s", "MB/s",
/// "GB/s" in bytes mode).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpeedUnitChoice {
    pub prefix: SpeedPrefix,
    pub bits: bool,
}

impl std::fmt::Display for SpeedUnitChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let scale = if self.bits {
            match self.prefix {
                SpeedPrefix::Kilo => "k",
                SpeedPrefix::Mega => "m",
                SpeedPrefix::Giga => "g",
            }
        } else {
            match self.prefix {
                SpeedPrefix::Kilo => "K",
                SpeedPrefix::Mega => "M",
                SpeedPrefix::Giga => "G",
            }
        };
        let base = if self.bits { "b" } else { "B" };
        write!(f, "{scale}{base}/s")
    }
}

/// Formats a speed value for the settings input: whole numbers without a
/// decimal point, everything else as-is.
fn format_speed_setting(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value.max(0.0) as u64)
    } else {
        format!("{value}")
    }
}
