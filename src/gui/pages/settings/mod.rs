use iced::widget::{button, column, container, row, scrollable, svg, text, toggler, Space};
use iced::{alignment, Color, Element, Length, Theme};

use crate::gui::components::common::{input_field, primary_button, secondary_button};
use crate::gui::styles;
use crate::monarch_utils;
use crate::monarch_utils::monarch_settings::Settings;

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
    ToggleQuickLaunch(bool),
    LibraryFolderChanged(String),
    SaveLibraryFolder,
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

pub struct SettingsPage {
    current_tab: SettingsTab,
    settings: Settings,
    cache_size: u64,
}

impl Default for SettingsPage {
    fn default() -> Self {
        Self {
            current_tab: SettingsTab::Monarch,
            settings: monarch_utils::commands::get_settings(),
            cache_size: monarch_utils::commands::get_cache_size().unwrap_or(0),
        }
    }
}

impl SettingsPage {
    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::TabSelected(tab) => self.current_tab = tab,
            Message::ToggleQuickLaunch(enabled) => self.settings.quicklaunch.enabled = enabled,
            Message::LibraryFolderChanged(folder) => self.settings.monarch.game_folder = folder,
            Message::ToggleSteam(enabled) => self.settings.steam.manage = enabled,
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
            Message::ToggleEpic(enabled) => self.settings.epic.manage = enabled,
            Message::EpicUsernameChanged(u) => self.settings.epic.username = u,
            Message::EpicPasswordChanged(p) => {
                let _ = monarch_utils::commands::set_password(
                    "epic".to_string(),
                    self.settings.epic.username.clone(),
                    p,
                );
            }
            Message::ResetDefaults => {
                self.settings = monarch_utils::commands::default_settings().unwrap()
            }
            Message::ClearCache => {
                monarch_utils::commands::clear_cached_images();
                self.refresh();
            }
            Message::Refresh => self.refresh(),
            _ => {}
        }
        iced::Task::none()
    }

    pub fn refresh(&mut self) {
        self.cache_size = monarch_utils::commands::get_cache_size().unwrap_or(0);
    }

    pub fn view(&self) -> Element<'_, Message, Theme> {
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
            SettingsTab::Monarch => self.view_monarch(),
            SettingsTab::Steam => self.view_steam(),
            SettingsTab::EpicGames => self.view_epic(),
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

    fn view_monarch(&self) -> Element<'_, Message, Theme> {
        column![
            self.section_header("General"),
            text("Configure general behavior and preferences for the Monarch launcher.")
                .size(14)
                .color([0.7, 0.7, 0.7]),
            Space::new().height(20),
            row![
                text("Quicklaunch (Requires restart. Shortcut: Ctrl+Enter)")
                    .size(16)
                    .width(Length::Shrink),
                Space::new().width(10),
                toggler(self.settings.quicklaunch.enabled).on_toggle(Message::ToggleQuickLaunch),
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
                    &self.settings.monarch.game_folder,
                    Message::LibraryFolderChanged
                ),
                Space::new().width(10),
                secondary_button("Browse", Some(Message::BrowseLibraryFolder)),
            ]
            .align_y(alignment::Vertical::Center),
            Space::new().height(10),
            row![
                Space::new().width(Length::Fill),
                primary_button("Save", Some(Message::SaveLibraryFolder)),
            ],
            Space::new().height(10),
            text(format!("Current: {}", self.settings.monarch.game_folder))
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
                button(text("Reset to Defaults").size(14))
                    .on_press(Message::ResetDefaults)
                    .padding(10)
                    .style(styles::button::destructive),
            ]
            .spacing(10),
        ]
        .spacing(10)
        .into()
    }

    fn view_steam(&self) -> Element<'_, Message, Theme> {
        let _tmp: &str = "";
        column![
            self.section_header("Steam Integration"),
            row![
                text("Allow Monarch to manage Steam games")
                    .size(16)
                    .width(Length::Shrink),
                Space::new().width(10),
                toggler(self.settings.steam.manage).on_toggle(Message::ToggleSteam),
            ]
            .align_y(alignment::Vertical::Center),
            Space::new().height(25),
            text("Enter your Steam credentials to enable library synchronization and game downloads.")
                .size(14)
                .color([0.7, 0.7, 0.7]),
            text("Read about authentication security")
                .size(12)
                .color([1.0, 0.4, 0.0]), // Orange-ish
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
            text("How to find your shared secret")
                .size(12)
                .color([1.0, 0.4, 0.0]),
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

    fn view_epic(&self) -> Element<'_, Message, Theme> {
        let _tmp: &str = "";
        column![
            self.section_header("Epic Games Integration"),
            row![
                text("Allow Monarch to manage Epic Games")
                    .size(16)
                    .width(Length::Shrink),
                Space::new().width(10),
                toggler(self.settings.epic.manage).on_toggle(Message::ToggleEpic),
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
        let i = ((self.cache_size as f64).log2() / k.log2()).round();
        let size = self.cache_size as f64 / (k.powi(i as i32));

        if size.is_nan() {
            return "0 B".to_string();
        }
        format!("{:.2} {}", size, prefixes[i as usize])
    }
}
