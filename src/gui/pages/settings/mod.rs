use iced::{Element, Theme};
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use tracing::error;

use crate::gui::components::common::error_view;
use crate::gui::show_error;
use crate::monarch_utils;
use crate::monarch_utils::monarch_settings::Settings;

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
    SteamGuardSecretChanged(String),
    SaveSteamSecret,
    ToggleEpic(bool),
    EpicUsernameChanged(String),
    EpicPasswordChanged(String),
    SaveEpicCredentials,
    Refresh(()),
    OpenLink(&'static str),
    InstallSteamCMD,
    InstallLegendary,
    InstallUmu,
}

impl Message {
    fn requires_write_lock(&self) -> bool {
        match self {
            Message::ToggleQuickLaunch(_)
            | Message::LibraryFolderChanged(_)
            | Message::ResetDefaults
            | Message::ToggleSteam(_)
            | Message::SaveSteamCredentials
            | Message::SaveSteamSecret
            | Message::ToggleEpic(_)
            | Message::SaveEpicCredentials
            | Message::InstallSteamCMD => true,
            _ => false,
        }
    }
}

pub struct SettingsPage {
    current_tab: SettingsTab,
    shared_settings: Arc<RwLock<Settings>>,
    cache_size: u64,
    steam_username_tmp: String,
    steam_password_tmp: String,
    steam_secret_tmp: String,
    epic_username_tmp: String,
    epic_password_tmp: String,
}

impl Default for SettingsPage {
    fn default() -> Self {
        let shared_settings = monarch_utils::commands::get_settings().unwrap_or_default();
        let (steam_user, epic_user) = match shared_settings.read() {
            Ok(s) => (s.steam.username.clone(), s.epic.username.clone()),
            Err(_) => (String::new(), String::new()),
        };

        Self {
            current_tab: SettingsTab::Monarch,
            shared_settings,
            cache_size: monarch_utils::commands::get_cache_size().unwrap_or(0),
            steam_username_tmp: steam_user,
            steam_password_tmp: String::new(),
            steam_secret_tmp: String::new(),
            epic_username_tmp: epic_user,
            epic_password_tmp: String::new(),
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
            Message::LibraryFolderChanged(folder) => {
                self.change_library_folder(&mut write_guard.unwrap(), &folder)
            }
            Message::ToggleSteam(state) => self.toggle_steam(&mut write_guard.unwrap(), state),
            Message::SteamUsernameChanged(u) => self.steam_username_tmp = u,
            Message::SteamPasswordChanged(p) => self.steam_password_tmp = p,
            Message::SaveSteamCredentials => {
                self.update_steam_credentials(&mut write_guard.unwrap())
            }
            Message::SteamGuardSecretChanged(s) => self.steam_secret_tmp = s,
            Message::SaveSteamSecret => self.update_steam_secret(&mut write_guard.unwrap()),
            Message::ToggleEpic(state) => self.toggle_epic(&mut write_guard.unwrap(), state),
            Message::EpicUsernameChanged(u) => self.epic_username_tmp = u,
            Message::EpicPasswordChanged(p) => self.epic_password_tmp = p,
            Message::SaveEpicCredentials => self.update_epic_credentials(&mut write_guard.unwrap()),
            Message::RequestResetDefaults => self.ask_reset_settings(),
            Message::ResetDefaults => self.reset_settings(&mut write_guard.unwrap()),
            Message::ClearCache => {
                monarch_utils::commands::clear_cached_images();
                self.refresh();
            }
            Message::Refresh(_) => self.refresh(),
            Message::OpenLogs => {
                let _ = monarch_utils::commands::open_logs();
            }
            Message::OpenLink(url) => self.open_link(url),
            Message::InstallUmu => self.install_umu_task(),
            Message::InstallSteamCMD => return self.install_steamcmd_task(&write_guard.unwrap()),
            Message::InstallLegendary => self.install_legendary_task(),
            _ => {}
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
