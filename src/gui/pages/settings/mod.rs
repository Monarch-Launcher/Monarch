use iced::{Element, Theme};
use std::sync::{Arc, RwLock};
use tracing::error;

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

#[derive(Clone, Debug)]
pub enum Message {
    TabSelected(SettingsTab),
    ToggleQuickLaunch,
    LibraryFolderChanged(String),
    SaveLibraryFolder,
    BrowseLibraryFolder,
    ClearCache,
    OpenLogs,
    ResetDefaults,
    ToggleSteam,
    SteamUsernameChanged(String),
    SteamPasswordChanged(String),
    SaveSteamCredentials,
    SteamGuardSecretChanged(String),
    SaveSteamSecret,
    ToggleEpic,
    EpicUsernameChanged(String),
    EpicPasswordChanged(String),
    SaveEpicCredentials,
    Refresh,
    RequestTestModal,
}

pub struct SettingsPage {
    current_tab: SettingsTab,
    settings: Arc<RwLock<Settings>>,
    cache_size: u64,
}

impl Default for SettingsPage {
    fn default() -> Self {
        Self {
            current_tab: SettingsTab::Monarch,
            settings: monarch_utils::commands::get_settings().unwrap(),
            cache_size: monarch_utils::commands::get_cache_size().unwrap_or(0),
        }
    }
}

impl SettingsPage {
    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::TabSelected(tab) => self.current_tab = tab,
            Message::ToggleQuickLaunch => self.settings.quicklaunch.enabled = enabled,
            Message::LibraryFolderChanged(folder) => self.settings.monarch.game_folder = folder,
            Message::ToggleSteam => self.settings.steam.manage = enabled,
            Message::SteamUsernameChanged(u) => self.settings.steam.username = u,
            Message::SteamPasswordChanged(p) => {
                let _ = monarch_utils::commands::set_password(
                    "steam".to_string(),
                    self.settings.steam.username.clone(),
                    p,
                );
            }
            Message::SteamGuardSecretChanged(s) => {
                monarch_utils::commands::set_secret("steam".to_string(), s.clone()).unwrap();
            }
            Message::ToggleEpic => self.settings.epic.manage = enabled,
            Message::EpicUsernameChanged(u) => self.settings.epic.username = u,
            Message::EpicPasswordChanged(p) => {
                let _ = monarch_utils::commands::set_password(
                    "epic".to_string(),
                    self.settings.epic.username.clone(),
                    p,
                );
            }
            Message::ResetDefaults => match monarch_utils::commands::default_settings() {
                Ok(settings) => self.settings = settings,
                Err(settings) => {
                    error!("Failed to reset settings");
                    show_error("Failed to reset settings!");
                    self.settings = settings;
                }
            },
            Message::ClearCache => {
                monarch_utils::commands::clear_cached_images();
                self.refresh();
            }
            Message::Refresh => self.refresh(),
            Message::RequestTestModal => {
                crate::gui::show_error("Easy error display works!");
            }
            Message::OpenLogs => {
                let _ = monarch_utils::commands::open_logs();
            }
            _ => {}
        }
        iced::Task::none()
    }

    pub fn view(&self) -> Element<'_, Message, Theme> {
        self.view_settings_page()
    }
}
