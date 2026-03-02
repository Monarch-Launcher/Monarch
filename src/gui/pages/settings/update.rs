use tracing::error;

use crate::{
    gui::{
        pages::{
            self,
            settings::{Message, SettingsPage},
        },
        show_confirm, show_error, AppMessage,
    },
    monarch_games,
    monarch_utils::{self, monarch_settings::Settings},
};

impl SettingsPage {
    pub fn toggle_quicklaunch(&mut self, settings: &mut Settings, state: bool) {
        settings.quicklaunch.enabled = state;
        self.write_settings(&settings);
    }

    pub fn change_library_folder(&mut self, settings: &mut Settings, folder: &str) {
        settings.monarch.game_folder = folder.to_string();
        self.write_settings(&settings);
    }

    pub fn toggle_steam(&mut self, settings: &mut Settings, state: bool) {
        settings.steam.manage = state;
        self.write_settings(&settings);
    }

    pub fn update_steam_credentials(&mut self, settings: &mut Settings) {
        settings.steam.username = self.steam_username_tmp.clone();

        if !self.steam_password_tmp.is_empty() {
            let _ = monarch_utils::commands::set_password(
                "steam",
                &mut settings.steam,
                &self.steam_username_tmp,
                &self.steam_password_tmp,
            );
        }

        self.write_settings(settings);
    }

    pub fn update_steam_secret(&mut self, settings: &mut Settings) {
        let _ = monarch_utils::commands::set_secret(
            "steam",
            &mut settings.steam,
            &self.steam_secret_tmp,
        );
        self.write_settings(settings);
    }

    pub fn toggle_epic(&mut self, settings: &mut Settings, state: bool) {
        settings.epic.manage = state;
        self.write_settings(&settings);
    }

    pub fn update_epic_credentials(&mut self, settings: &mut Settings) {
        settings.epic.username = self.epic_username_tmp.clone();

        if !self.epic_password_tmp.is_empty() {
            let _ = monarch_utils::commands::set_password(
                "epic",
                &mut settings.epic,
                &self.epic_username_tmp,
                &self.epic_password_tmp,
            );
        }

        self.write_settings(settings);
    }

    pub fn refresh(&mut self) {
        match monarch_utils::commands::get_cache_size() {
            Ok(size) => self.cache_size = size,
            Err(e) => {
                self.cache_size = 0;
                error!("Failed to get cache size: {}", e);
                show_error("Failed to get cache size!");
            }
        }
    }

    pub fn ask_reset_settings(&self) {
        show_confirm(
            "Are you sure you want to reset to default settings?",
            AppMessage::Page(pages::Message::Settings(Message::ResetDefaults)),
        );
    }

    pub fn reset_settings(&mut self, settings: &mut Settings) {
        *settings = monarch_utils::monarch_settings::Settings::default();
        self.write_settings(settings);
    }

    pub fn write_settings(&self, settings: &Settings) {
        if let Err(e) = monarch_utils::commands::write_settings(settings) {
            error!("Failed to write settings: {}", e);
            show_error("Failed to write settings! The change will be reverted on next launch.");
        }
    }

    pub fn open_link(&self, url: &str) {
        monarch_utils::commands::open_external_link(url);
    }

    pub fn install_umu_task(&self) {
        if let Err(e) = monarch_games::commands::install_umu() {
            show_error(&e);
        }
    }

    pub fn install_steamcmd_task(&self, settings: &Settings) -> iced::Task<Message> {
        if settings.steam.username.is_empty() {
            show_error("No Steam username set. Please set your username at least.");
            return iced::Task::none();
        }

        iced::Task::perform(
            async move {
                let _ = monarch_games::commands::install_steamcmd().await;
            },
            Message::Refresh,
        )
    }

    pub fn install_legendary_task(&self) {
        if let Err(e) = monarch_games::commands::install_legendary() {
            show_error(&e);
        }
    }
}
