use std::sync::RwLockReadGuard;

use iced::{
    alignment,
    widget::{
        button, column, container, rich_text, row, scrollable, span, svg, text, toggler, Space,
    },
    Color, Element, Length, Theme,
};

use crate::{
    gui::{
        components::common::{danger_button, input_field, primary_button, secondary_button},
        pages::settings::{Message, SettingsPage, SettingsTab},
        styles,
    }, monarch_games, monarch_utils::monarch_settings::Settings
};

impl SettingsPage {
    pub fn view_settings_page(
        &self,
        settings: RwLockReadGuard<'_, Settings>,
    ) -> Element<'_, Message, Theme> {
        let sidebar = column![
            self.tab_button(
                "Monarch",
                SettingsTab::Monarch,
                crate::gui::resources::MONARCH.clone()
            ),
            self.tab_button(
                "Steam",
                SettingsTab::Steam,
                crate::gui::resources::STEAM.clone()
            ),
            self.tab_button(
                "Epic Games",
                SettingsTab::EpicGames,
                crate::gui::resources::EPIC.clone()
            ),
            self.tab_button("GOG", SettingsTab::Gog, crate::gui::resources::GOG.clone()),
            self.tab_button(
                "Itch.io",
                SettingsTab::ItchIo,
                crate::gui::resources::ITCH.clone()
            ),
        ]
        .width(Length::Fixed(200.0))
        .spacing(10)
        .padding(20);

        let content = container(scrollable(match self.current_tab {
            SettingsTab::Monarch => self.view_monarch(settings),
            SettingsTab::Steam => self.view_steam(settings),
            SettingsTab::EpicGames => self.view_epic(settings),
            SettingsTab::Gog | SettingsTab::ItchIo => self.view_coming_soon(),
        }))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(40);

        row![sidebar, content].into()
    }

    fn tab_button<'a>(
        &self,
        label: &'a str,
        tab: SettingsTab,
        icon: svg::Handle,
    ) -> Element<'a, Message, Theme> {
        let is_active = self.current_tab == tab;

        let style = if is_active {
            styles::button::primary
        } else {
            styles::button::text
        };

        button(
            row![
                svg(icon)
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0))
                    .style(move |_theme: &Theme, _status| {
                        iced::widget::svg::Style {
                            color: Some(if is_active {
                                Color::BLACK
                            } else {
                                Color::WHITE
                            }),
                        }
                    }),
                text(label)
                    .size(16)
                    .width(Length::Fill)
                    .align_x(alignment::Horizontal::Left),
            ]
            .spacing(10)
            .align_y(alignment::Vertical::Center),
        )
        .on_press(Message::TabSelected(tab))
        .padding(12)
        .width(Length::Fill)
        .style(style)
        .into()
    }

    fn view_monarch(&self, settings: RwLockReadGuard<'_, Settings>) -> Element<'_, Message, Theme> {
        column![
            self.section_header("General"),
            Space::new().height(20),
            text("Configure general behavior and preferences for the Monarch launcher.")
                .size(20)
                .color([0.7, 0.7, 0.7]),
            Space::new().height(20),
            row![
                text("Quicklaunch (Requires restart. Shortcut: Ctrl+Enter)")
                    .size(16)
                    .width(Length::Shrink),
                Space::new().width(10),
                toggler(settings.quicklaunch.enabled).on_toggle(Message::ToggleQuickLaunch),
            ]
            .align_y(alignment::Vertical::Center),
            Space::new().height(40),
            self.section_header("Game Library Folder"),
            text("Set the default folder where Monarch will download new games.")
                .size(14)
                .color([0.7, 0.7, 0.7]),
            Space::new().height(15),
            row![
                input_field(
                    "Path to game folder",
                    &settings.monarch.game_folder,
                    Message::LibraryFolderChanged
                ),
                Space::new().width(10),
                secondary_button("Browse", Some(Message::BrowseLibraryFolder)),
            ]
            .align_y(alignment::Vertical::Center),
            Space::new().height(10),
            text(format!("Current: {}", settings.monarch.game_folder))
                .size(12)
                .color([0.5, 0.5, 0.5]),
            Space::new().height(40),
            self.section_header("Storage & Cache"),
            row![
                text(format!("Cached images: {}", self.format_cache())).size(14),
                Space::new().width(Length::Fill),
                secondary_button("Clear cache", Some(Message::ClearCache)),
            ]
            .align_y(alignment::Vertical::Center),
            Space::new().height(40),
            self.section_header("System"),
            row![
                secondary_button("Open Logs", Some(Message::OpenLogs)),
                Space::new().width(10),
                danger_button("Reset to Defaults", Some(Message::RequestResetDefaults)),
            ]
            .spacing(10),
        ]
        .spacing(10)
        .into()
    }

    fn view_steam(&self, settings: RwLockReadGuard<'_, Settings>) -> Element<'_, Message, Theme> {
        let _tmp: &str = "";
        let steamcmd_installed: &str = if monarch_games::commands::steamcmd_is_installed() {
            "Installed"
        } else {
            "Not installed"
        };

        column![
            self.section_header("Steam Integration"),
            row![
                text("Allow Monarch to manage Steam games")
                    .size(16)
                    .width(Length::Shrink),
                Space::new().width(10),
                toggler(settings.steam.manage).on_toggle(Message::ToggleSteam),
            ]
            .align_y(alignment::Vertical::Center),
            Space::new().height(25),
            
            row![
                text(format!("SteamCMD: {}", steamcmd_installed)).size(16).width(Length::Shrink),
                Space::new().width(Length::Fill),
                primary_button("Install SteamCMD", Some(Message::InstallSteamCMD)),
            ]
            .align_y(alignment::Vertical::Center),
            Space::new().height(25),

            text("Enter your Steam credentials to enable library synchronization and game downloads.")
                .size(14)
                .color([0.7, 0.7, 0.7]),
            rich_text([
                span("Read about how Monarch handles Steam account security")
                    .size(14)
                    .color([1.0, 0.4, 0.0])
                    .link("https://github.com/Monarch-Launcher/Monarch/blob/main/docs/steam_login.md")])
                .on_link_click(Message::OpenLink)
                .size(14),
            Space::new().height(15),

            input_field(
                "Steam Username",
                _tmp,
                Message::SteamUsernameChanged
            ),
            Space::new().height(10),

            input_field(
                "Steam Password",
                _tmp,
                Message::SteamPasswordChanged
            ),
            Space::new().height(10),

            row![
                Space::new().width(Length::Fill),
                primary_button("Save Credentials", Some(Message::SaveSteamCredentials)),
            ],
            Space::new().height(10),
            
            text("Status: Not logged in").size(14).color([0.5, 0.5, 0.5]),
            Space::new().height(40),

            container(column![])
                .width(Length::Fill)
                .height(Length::Fixed(1.0))
                .style(|_| container::Style {
                    background: Some(iced::Color::from_rgba8(255, 255, 255, 0.1).into()),
                    ..Default::default()
                }),
            Space::new().height(40),

            self.section_header("Steam Guard (2FA)"),
            text("If you use Steam Guard Mobile Authenticator, you can provide your shared secret here.")
                .size(14)
                .color([0.7, 0.7, 0.7]),
            Space::new().height(15),

            input_field(
                "Shared Secret",
                _tmp,
                Message::SteamGuardSecretChanged
            ),
            Space::new().height(10),

            row![
                Space::new().width(Length::Fill),
                primary_button("Save Secret", Some(Message::SaveSteamSecret)),
            ],
            Space::new().height(10),

            text("Status: Not configured").size(14).color([0.5, 0.5, 0.5]),
        ]
        .spacing(10)
        .into()
    }

    fn view_epic(&self, settings: RwLockReadGuard<'_, Settings>) -> Element<'_, Message, Theme> {
        let _tmp: &str = "";
        column![
            self.section_header("Epic Games Integration"),
            row![
                text("Allow Monarch to manage Epic Games")
                    .size(16)
                    .width(Length::Shrink),
                Space::new().width(10),
                toggler(settings.epic.manage).on_toggle(Message::ToggleEpic),
            ]
            .align_y(alignment::Vertical::Center),
            Space::new().height(25),
            text("Enter your Epic Games credentials to enable library synchronization and game downloads.")
                .size(14)
                .color([0.7, 0.7, 0.7]),
            Space::new().height(15),
            input_field(
                "Epic Username / Email",
                _tmp,
                Message::EpicUsernameChanged
            ),
            Space::new().height(10),
            input_field(
                "Password",
                _tmp,
                Message::EpicPasswordChanged
            ),
            Space::new().height(10),
            row![
                Space::new().width(Length::Fill),
                primary_button("Save Credentials", Some(Message::SaveEpicCredentials)),
            ],
            Space::new().height(10),
            text("Status: Not logged in").size(14).color([0.5, 0.5, 0.5]),
        ]
        .spacing(10)
        .into()
    }

    fn view_coming_soon(&self) -> Element<'_, Message, Theme> {
        container(
            text("Coming soon")
                .size(30)
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }

    fn section_header<'a>(&self, label: &'a str) -> Element<'a, Message, Theme> {
        text(label).size(24).width(Length::Fill).into()
    }

    fn format_cache(&self) -> String {
        let prefixes: [&str; 4] = ["B", "KiB", "MiB", "GiB"];
        let k = 1024 as f64;
        let i = ((self.cache_size as f64).log2() / k.log2()).floor();
        let size = self.cache_size as f64 / (k.powi(i as i32));

        if size.is_nan() {
            return "0 B".to_string();
        }
        format!("{:.2} {}", size, prefixes[i as usize])
    }
}
