use iced::Task;
use tracing::error;

use crate::gui::{
    components::common::open_folder_dialog,
    pages::{
        self,
        settings::{Message, SettingsPage},
    },
    show_confirm, show_error, AppMessage,
};
use monarch_core::monarch_games::{self, egs_client::EgsClient};
use monarch_core::monarch_utils::{self, monarch_settings::Settings};

use super::{format_speed_setting, SpeedPrefix};

impl SettingsPage {
    pub fn toggle_quicklaunch(&mut self, settings: &mut Settings, state: bool) {
        settings.quicklaunch.enabled = state;
        self.write_settings(settings);
    }

    pub fn toggle_download_speed_bits(&mut self, settings: &mut Settings, state: bool) {
        // Switching the unit base must not silently change the effective
        // limit eightfold, so re-express the entered number in the new base.
        let effective_bps = settings.monarch.max_download_speed_bps();
        settings.monarch.show_download_speed_in_bits = state;

        if settings.monarch.max_download_speed_value > 0.0 {
            let multiplier = settings.monarch.max_download_speed_multiplier();
            settings.monarch.max_download_speed_value = if state {
                effective_bps as f64 * 8.0 / multiplier
            } else {
                effective_bps as f64 / multiplier
            };
            // Keep the input showing what is now stored.
            self.max_speed_tmp = format_speed_setting(settings.monarch.max_download_speed_value);
        }
        self.write_settings(settings);
        self.apply_max_download_speed(settings);
    }

    /// Handles edits to the max download speed input. The raw text is always
    /// kept in the temp field so typing isn't fought; only valid, finite,
    /// non-negative values are persisted and pushed to the downloader.
    pub fn change_max_download_speed(&mut self, settings: &mut Settings, value: String) {
        self.max_speed_tmp = value;

        let trimmed = self.max_speed_tmp.trim();
        let parsed = if trimmed.is_empty() {
            Some(0.0)
        } else {
            trimmed.parse::<f64>().ok()
        };

        let Some(speed_value) = parsed.filter(|speed| speed.is_finite()) else {
            // Intermediate/invalid input (e.g. "1." or "-"): keep the last
            // applied limit until the text becomes a valid number again.
            return;
        };
        let speed_value = speed_value.max(0.0);

        settings.monarch.max_download_speed_value = speed_value;
        self.write_settings(settings);
        self.apply_max_download_speed(settings);
    }

    /// Persists the chosen unit prefix and re-applies the limit.
    pub fn select_max_download_speed_unit(&mut self, settings: &mut Settings, prefix: SpeedPrefix) {
        self.max_speed_prefix = prefix;
        settings.monarch.max_download_speed_prefix = prefix.as_str().to_string();
        self.write_settings(settings);
        self.apply_max_download_speed(settings);
    }

    /// Pushes the persisted speed limit into the live downloader so running
    /// and queued downloads pick it up immediately.
    fn apply_max_download_speed(&self, settings: &Settings) {
        let bps = settings.monarch.max_download_speed_bps();
        if let Err(e) = monarch_utils::commands::set_max_download_speed_bps(bps) {
            error!("Failed to apply max download speed | Err: {e}");
            show_error("Failed to apply the download speed limit!");
        }
    }

    pub fn toggle_auto_update_check(&mut self, settings: &mut Settings, state: bool) {
        settings.monarch.check_updates_on_startup = state;
        self.write_settings(settings);
    }

    pub fn change_library_folder(&mut self, settings: &mut Settings, folder: &str) {
        settings.monarch.game_folder = folder.to_string();
        self.write_settings(settings);
    }

    pub fn toggle_steam(&mut self, settings: &mut Settings, state: bool) {
        settings.steam.manage = state;
        self.write_settings(settings);
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

        self.steam_password_tmp = "".to_string();
        self.write_settings(settings);
    }

    pub fn delete_steam_credentials(&mut self, settings: &mut Settings) {
        settings.steam.username = "".to_string();
        if let Err(e) = monarch_utils::commands::delete_password("steam", &mut settings.steam) {
            error!("Failed to delete Steam credentials: {}", e);
            show_error("Failed to delete Steam credentials!");
        }
        if let Err(e) = monarch_utils::commands::delete_secret("steam", &mut settings.steam) {
            error!("Failed to delete Steam secret: {}", e);
            show_error("Failed to delete Steam secret!");
        }
        self.write_settings(settings);
    }

    pub fn update_steam_secret(&mut self, settings: &mut Settings) {
        let _ = monarch_utils::commands::set_secret(
            "steam",
            &mut settings.steam,
            &self.steam_secret_tmp,
        );
        settings.steam.twofa = true;
        self.steam_secret_tmp = "".to_string();
        self.write_settings(settings);
    }

    pub fn remove_steam_secret(&mut self, settings: &mut Settings) {
        if let Err(e) = monarch_utils::commands::delete_secret("steam", &mut settings.steam) {
            show_error(e);
        }
        settings.steam.twofa = false;
        self.write_settings(settings);
    }

    pub fn toggle_epic(&mut self, settings: &mut Settings, state: bool) {
        settings.epic.manage = state;
        self.write_settings(settings);
    }

    pub fn login_epic(&self) {
        let client: EgsClient = monarch_games::egs_client::EgsClient::new();
        if client.credentials_exist() {
            show_error("Epic Games credentials detected on system! Please delete them before attempting to log in.");
            return;
        }
        client.open_epic_login();
    }

    pub fn login_epic_auth_code(&mut self, settings: &mut Settings) {
        let mut client: EgsClient = monarch_games::egs_client::EgsClient::new();
        let trimmed_code: &str = self.epic_auth_code_tmp.trim().trim_matches('"');
        if let Err(e) = futures::executor::block_on(client.save_epic_auth_code(trimmed_code)) {
            error!("Failed to login to epic games using auth code! -> {:?}", e);
            show_error("Failed to login to Epic Games using authorization code!");
            return;
        }
        self.epic_auth_code_tmp = "".to_string();
        settings.epic.username = client.display_name();
        self.write_settings(settings);
    }

    pub fn delete_epic_credentials(&mut self, settings: &mut Settings) {
        settings.epic.username = "".to_string();
        if let Err(e) = monarch_utils::commands::delete_password("epic", &mut settings.epic) {
            error!("Failed to delete Epic credentials: {}", e);
            show_error("Failed to delete Epic credentials!");
            return;
        }
        self.write_settings(settings);
    }

    pub fn refresh(&mut self, settings: &mut Settings) {
        // Get updated paths and shit
        settings.fix_settings();

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
        if monarch_games::commands::umu_is_installed() {
            show_error("umu-run already detected on system! Cannot install another version.");
            return;
        }

        if let Err(e) = monarch_games::commands::install_umu() {
            show_error(&e);
        }
    }

    pub fn install_steamcmd_task(&self, settings: &Settings) -> iced::Task<Message> {
        if monarch_games::commands::steamcmd_is_installed() {
            show_error("steamcmd already detected on system! Cannot install another version.");
            return iced::Task::none();
        }

        if settings.steam.username.is_empty() {
            show_error("No Steam username set. Please set your username.");
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
        if monarch_games::commands::legendary_is_installed() {
            show_error("legendary already detected on system! Cannot install another version.");
            return;
        }

        if let Err(e) = monarch_games::commands::install_legendary() {
            show_error(&e);
        }
    }

    #[cfg(target_os = "linux")]
    pub fn remove_umu(&self) {
        if let Err(e) = monarch_games::commands::remove_umu() {
            show_error(e);
        }
    }

    pub fn remove_steamcmd(&self) {
        if let Err(e) = monarch_games::commands::remove_steamcmd() {
            show_error(e);
        }
    }

    pub fn remove_legendary(&self) {
        if let Err(e) = monarch_games::commands::remove_legendary() {
            show_error(e);
        }
    }

    pub fn pick_default_monarch_folder(&self) -> iced::Task<Message> {
        Task::future(open_folder_dialog()).then(|handle| match handle {
            Some(file_handle) => Task::done(Message::LibraryFolderChanged(
                file_handle.path().to_string_lossy().to_string(),
            )),
            None => iced::Task::none(),
        })
    }
}
