use iced::widget::{button, column, container, row, scrollable, text, toggler, Space};
use iced::{alignment, Element, Length, Theme};

use crate::gui::components::common::{input_field, primary_button, secondary_button};
use crate::gui::styles;

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
}

pub struct SettingsPage {
    current_tab: SettingsTab,
    quicklaunch_enabled: bool,
    library_folder: String,
    steam_enabled: bool,
    steam_username: String,
    steam_password: String,
    steam_guard_secret: String,
    epic_enabled: bool,
    epic_username: String,
    epic_password: String,
}

impl Default for SettingsPage {
    fn default() -> Self {
        Self {
            current_tab: SettingsTab::Monarch,
            quicklaunch_enabled: false,
            library_folder: "/home/mooze/MonarchGames".to_string(),
            steam_enabled: false,
            steam_username: "".to_string(),
            steam_password: "".to_string(),
            steam_guard_secret: "".to_string(),
            epic_enabled: false,
            epic_username: "".to_string(),
            epic_password: "".to_string(),
        }
    }
}

impl SettingsPage {
    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::TabSelected(tab) => self.current_tab = tab,
            Message::ToggleQuickLaunch(enabled) => self.quicklaunch_enabled = enabled,
            Message::LibraryFolderChanged(folder) => self.library_folder = folder,
            Message::ToggleSteam(enabled) => self.steam_enabled = enabled,
            Message::SteamUsernameChanged(u) => self.steam_username = u,
            Message::SteamPasswordChanged(p) => self.steam_password = p,
            Message::SteamGuardSecretChanged(s) => self.steam_guard_secret = s,
            Message::ToggleEpic(enabled) => self.epic_enabled = enabled,
            Message::EpicUsernameChanged(u) => self.epic_username = u,
            Message::EpicPasswordChanged(p) => self.epic_password = p,
            _ => {}
        }
        iced::Task::none()
    }

    pub fn view(&self) -> Element<'_, Message, Theme> {
        let sidebar = column![
            self.tab_button("Monarch", SettingsTab::Monarch),
            self.tab_button("Steam", SettingsTab::Steam),
            self.tab_button("Epic Games", SettingsTab::EpicGames),
            self.tab_button("GOG", SettingsTab::Gog),
            self.tab_button("Itch.io", SettingsTab::ItchIo),
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

    fn tab_button<'a>(&self, label: &'a str, tab: SettingsTab) -> Element<'a, Message, Theme> {
        let is_active = self.current_tab == tab;

        let style = if is_active {
            styles::button::primary
        } else {
            styles::button::text
        };

        button(
            text(label)
                .size(16)
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Left),
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
                toggler(self.quicklaunch_enabled).on_toggle(Message::ToggleQuickLaunch),
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
                    &self.library_folder,
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
            text(format!("Current: {}", self.library_folder))
                .size(12)
                .color([0.5, 0.5, 0.5]),
            Space::new().height(40),
            self.section_header("Storage & Cache"),
            row![
                text("Cached images: 1.22 MB").size(14),
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
        column![
            self.section_header("Steam Integration"),
            row![
                text("Allow Monarch to manage Steam games")
                    .size(16)
                    .width(Length::Shrink),
                Space::new().width(10),
                toggler(self.steam_enabled).on_toggle(Message::ToggleSteam),
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
                &self.steam_username,
                Message::SteamUsernameChanged
            ),
            Space::new().height(10),
            input_field(
                "Steam Password",
                &self.steam_password,
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
                &self.steam_guard_secret,
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
        column![
            self.section_header("Epic Games Integration"),
            row![
                text("Allow Monarch to manage Epic Games")
                    .size(16)
                    .width(Length::Shrink),
                Space::new().width(10),
                toggler(self.epic_enabled).on_toggle(Message::ToggleEpic),
            ]
            .align_y(alignment::Vertical::Center),
            Space::new().height(25),
            text("Enter your Epic Games credentials to enable library synchronization and game downloads.")
                .size(14)
                .color([0.7, 0.7, 0.7]),
            Space::new().height(15),
            input_field(
                "Epic Username / Email",
                &self.epic_username,
                Message::EpicUsernameChanged
            ),
            Space::new().height(10),
            input_field(
                "Password",
                &self.epic_password,
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
}
