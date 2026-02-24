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
    Refresh,
}

impl Message {
    fn requires_write_lock(&self) -> bool {
        match self {
            Message::ToggleQuickLaunch(_)
            | Message::LibraryFolderChanged(_)
            | Message::ResetDefaults
            | Message::ToggleSteam(_)
            | Message::SteamUsernameChanged(_)
            | Message::SteamPasswordChanged(_)
            | Message::SaveSteamCredentials
            | Message::SteamGuardSecretChanged(_)
            | Message::SaveSteamSecret
            | Message::ToggleEpic(_)
            | Message::EpicUsernameChanged(_)
            | Message::EpicPasswordChanged(_)
            | Message::SaveEpicCredentials => true,
            _ => false,
        }
    }
}

pub struct SettingsPage {
    current_tab: SettingsTab,
    shared_settings: Arc<RwLock<Settings>>,
    cache_size: u64,
}

impl Default for SettingsPage {
    fn default() -> Self {
        Self {
            current_tab: SettingsTab::Monarch,
            shared_settings: monarch_utils::commands::get_settings().unwrap_or_default(),
            cache_size: monarch_utils::commands::get_cache_size().unwrap_or(0),
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
                self.toggle_quicklaunch(write_guard.unwrap(), state)
            }
            Message::LibraryFolderChanged(folder) => {
                self.change_library_folder(write_guard.unwrap(), &folder)
            }
            Message::ToggleSteam(state) => self.toggle_steam(write_guard.unwrap(), state),
            Message::SteamUsernameChanged(u) => {
                self.update_steam_username(write_guard.unwrap(), &u)
            }
            Message::SteamPasswordChanged(p) => {
                self.update_steam_password(write_guard.unwrap(), &p)
            }
            Message::SaveSteamCredentials => self.write_settings(&write_guard.unwrap()),
            Message::SteamGuardSecretChanged(s) => {
                self.update_steam_secret(write_guard.unwrap(), &s)
            }
            Message::SaveSteamSecret => self.write_settings(&write_guard.unwrap()),
            Message::ToggleEpic(state) => self.toggle_epic(write_guard.unwrap(), state),
            Message::EpicUsernameChanged(u) => self.update_epic_username(write_guard.unwrap(), &u),
            Message::EpicPasswordChanged(p) => self.update_epic_password(write_guard.unwrap(), &p),
            Message::SaveEpicCredentials => self.write_settings(&write_guard.unwrap()),
            Message::ResetDefaults => match monarch_utils::monarch_settings::set_default_settings()
            {
                Ok(settings) => {
                    let mut guard = write_guard.unwrap();
                    *guard = settings;
                }
                Err(e) => {
                    error!("Failed to reset settings: {e}");
                    show_error("Failed to reset settings!");
                }
            },
            Message::ClearCache => {
                monarch_utils::commands::clear_cached_images();
                self.refresh();
            }
            Message::Refresh => self.refresh(),
            Message::OpenLogs => {
                let _ = monarch_utils::commands::open_logs();
            }
            _ => {}
        }
        iced::Task::none()
    }

    pub fn view(&self) -> Element<'_, Message, Theme> {
        match self.shared_settings.read() {
            Ok(settings) => self.view_settings_page(settings),
            Err(e) => {
                error!("gui::Settings::view() Failed to lock on shared_settings! | Err: {e}");
                error_view(
                    "Settings Unavailable",
                    "Failed to acquire a read lock on application settings. This might be due to a background process holding a write lock.",
                    Some(Message::Refresh),
                )
            }
        }
    }
}
