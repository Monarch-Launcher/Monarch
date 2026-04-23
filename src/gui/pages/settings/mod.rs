use iced::{Element, Theme};
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use tracing::error;

use crate::gui::components::common::error_view;
use crate::gui::{self, show_confirm, show_error, AppMessage};
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
    DeleteSteamCredentials,
    SteamGuardSecretChanged(String),
    SaveSteamSecret,
    DeleteSteamSecret,
    ToggleEpic(bool),
    EpicUsernameChanged(String),
    EpicPasswordChanged(String),
    SaveEpicCredentials,
    DeleteEpicCredentials,
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
    ToggleEpicHiddenPassword,
    ToggleHiddenSteamSecret,
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
            | Message::DeleteSteamSecret
            | Message::DeleteSteamCredentials
            | Message::ToggleEpic(_)
            | Message::SaveEpicCredentials
            | Message::DeleteEpicCredentials
            | Message::InstallSteamCMD
            | Message::Refresh(_)
            | Message::ClearCache => true,
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
    view_steam_password: bool,
    view_steam_secret: bool,
    steam_secret_tmp: String,
    epic_username_tmp: String,
    epic_password_tmp: String,
    view_epic_password: bool,
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
            view_steam_password: false,
            view_steam_secret: false,
            steam_secret_tmp: String::new(),
            epic_username_tmp: epic_user,
            epic_password_tmp: String::new(),
            view_epic_password: false,
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
            Message::DeleteSteamCredentials => {
                self.delete_steam_credentials(&mut write_guard.unwrap())
            }
            Message::SteamGuardSecretChanged(s) => self.steam_secret_tmp = s,
            Message::SaveSteamSecret => self.update_steam_secret(&mut write_guard.unwrap()),
            Message::DeleteSteamSecret => self.remove_steam_secret(&mut write_guard.unwrap()),
            Message::ToggleEpic(state) => self.toggle_epic(&mut write_guard.unwrap(), state),
            Message::EpicUsernameChanged(u) => self.epic_username_tmp = u,
            Message::EpicPasswordChanged(p) => self.epic_password_tmp = p,
            Message::SaveEpicCredentials => self.update_epic_credentials(&mut write_guard.unwrap()),
            Message::DeleteEpicCredentials => {
                self.delete_epic_credentials(&mut write_guard.unwrap())
            }
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
            Message::ToggleEpicHiddenPassword => {
                self.view_epic_password = !self.view_epic_password
            },
            Message::ToggleHiddenSteamSecret => {
                self.view_steam_secret = !self.view_steam_secret;
            },
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
