use std::sync::RwLockWriteGuard;

use tracing::error;

use crate::{
    gui::{pages::settings::SettingsPage, show_error},
    monarch_utils::{self, monarch_settings::Settings},
};

impl SettingsPage {
    pub fn toggle_quicklaunch(
        &mut self,
        mut settings: RwLockWriteGuard<'_, Settings>,
        state: bool,
    ) {
        settings.quicklaunch.enabled = state;
        self.write_settings(&settings);
    }

    pub fn change_library_folder(
        &mut self,
        mut settings: RwLockWriteGuard<'_, Settings>,
        folder: &str,
    ) {
        settings.monarch.game_folder = folder.to_string();
    }

    pub fn toggle_steam(&mut self, mut settings: RwLockWriteGuard<'_, Settings>, state: bool) {
        settings.steam.manage = state;
        self.write_settings(&settings);
    }

    pub fn update_steam_username(
        &mut self,
        mut settings: RwLockWriteGuard<'_, Settings>,
        username: &str,
    ) {
        settings.steam.username = username.to_string();
    }

    pub fn update_steam_password(
        &mut self,
        mut settings: RwLockWriteGuard<'_, Settings>,
        password: &str,
    ) {
        let username: String = settings.steam.username.clone();
        let _ = monarch_utils::commands::set_password(
            "steam",
            &mut settings.steam,
            &username,
            password,
        );
    }

    pub fn update_steam_secret(
        &mut self,
        mut settings: RwLockWriteGuard<'_, Settings>,
        secret: &str,
    ) {
        let _ = monarch_utils::commands::set_secret("steam", &mut settings.steam, secret);
    }

    pub fn toggle_epic(&mut self, mut settings: RwLockWriteGuard<'_, Settings>, state: bool) {
        settings.epic.manage = state;
        self.write_settings(&settings);
    }

    pub fn update_epic_username(
        &mut self,
        mut settings: RwLockWriteGuard<'_, Settings>,
        username: &str,
    ) {
        settings.epic.username = username.to_string();
    }

    pub fn update_epic_password(
        &mut self,
        mut settings: RwLockWriteGuard<'_, Settings>,
        password: &str,
    ) {
        let username: String = settings.epic.username.clone();
        let _ =
            monarch_utils::commands::set_password("epic", &mut settings.epic, &username, password);
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

    pub fn write_settings(&self, settings: &Settings) {
        if let Err(e) = monarch_utils::commands::write_settings(settings) {
            error!("Failed to write settings: {}", e);
            show_error("Failed to write settings! The change will be reverted on next launch.");
        }
    }
}
