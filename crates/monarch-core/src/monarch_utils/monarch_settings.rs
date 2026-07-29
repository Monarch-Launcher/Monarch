use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use toml::Table;
use tracing::{error, info};

use super::monarch_fs::{create_dir, generate_monarch_home, get_settings_path, path_exists};
use crate::monarch_games::monarch_client::generate_default_folder;
use crate::monarch_games::{legendary_client, steam_client};
use crate::monarch_utils::monarch_state::MONARCH_STATE;

#[cfg(target_os = "linux")]
use crate::monarch_games::linux::umu;

/*
* ----- Settings related structs ------
*/

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherSettings {
    pub game_folders: Vec<String>,
    pub manage: bool,
    pub username: String,
    pub twofa: bool,

    #[serde(default)]
    pub custom_data: String, // Arbitrary data - used to store Epic Games creds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonarchSettings {
    pub game_folder: String,
    pub monarch_home: String,
    pub run_on_startup: bool,
    pub send_logs: bool,
    pub start_minimized: bool,

    #[serde(default)]
    pub umu_bin: String,

    #[serde(default)]
    pub steamcmd_bin: String,

    #[serde(default)]
    pub legendary_bin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuicklaunchSettings {
    pub close_shortcut: String,
    pub open_shortcut: String,
    pub enabled: bool,
    pub size: String,
}

/// Struct for storing a persistent state of settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub settings_path: String,
    pub monarch: MonarchSettings,
    pub quicklaunch: QuicklaunchSettings,
    pub steam: LauncherSettings,
    pub epic: LauncherSettings,
}

impl Settings {
    pub const fn new() -> Self {
        Self {
            settings_path: String::new(),
            monarch: MonarchSettings {
                game_folder: String::new(),
                monarch_home: String::new(),
                run_on_startup: false,
                send_logs: false,
                start_minimized: false,
                umu_bin: String::new(),
                steamcmd_bin: String::new(),
                legendary_bin: String::new(),
            },
            quicklaunch: QuicklaunchSettings {
                close_shortcut: String::new(),
                open_shortcut: String::new(),
                enabled: false,
                size: String::new(),
            },
            steam: LauncherSettings {
                game_folders: Vec::new(),
                manage: false,
                username: String::new(),
                twofa: false,
                custom_data: String::new(),
            },
            epic: LauncherSettings {
                game_folders: Vec::new(),
                manage: false,
                username: String::new(),
                twofa: false,
                custom_data: String::new(),
            },
        }
    }

    /// Verifies some settings and returns whether or not settings were changed
    pub fn fix_settings(&mut self) {
        if self.settings_path.is_empty() {
            self.settings_path = get_settings_path().unwrap().to_string_lossy().to_string();
        }
        if self.monarch.game_folder.is_empty() {
            self.monarch.game_folder = generate_default_folder()
                .unwrap()
                .to_string_lossy()
                .to_string();
        }
        #[cfg(target_os = "linux")]
        {
            self.monarch.umu_bin = umu::get_umu_exe().to_string_lossy().to_string();
        }
        self.monarch.steamcmd_bin = steam_client::get_steamcmd_exe()
            .to_string_lossy()
            .to_string();
        self.monarch.legendary_bin = legendary_client::get_legendary_exe()
            .to_string_lossy()
            .to_string();

        if let Err(e) = write_settings(&self) {
            error!("monarch_settings::fix_settings() Failed to write settings! | Err: {e}")
        }
    }
}

// TODO: Redo this implementation to make sure it doesn't panic
impl From<Settings> for Table {
    fn from(src: Settings) -> Table {
        let toml = toml::to_string_pretty(&src).unwrap();
        Table::from_str(&toml).unwrap()
    }
}

impl Default for Settings {
    fn default() -> Self {
        let home_path = generate_monarch_home().unwrap();
        let home_path_str = home_path.to_str().unwrap().to_string();
        let default_game_folder = generate_default_folder().unwrap();
        let default_game_folder_str = default_game_folder.to_str().unwrap().to_string();
        let settings_path = get_settings_path().unwrap().to_str().unwrap().to_string();

        #[cfg(target_os = "linux")]
        let umu_bin: String = umu::get_umu_exe().to_string_lossy().to_string();

        #[cfg(not(target_os = "linux"))]
        let umu_bin: String = String::new();

        let steamcmd_bin: String = steam_client::get_steamcmd_exe()
            .to_string_lossy()
            .to_string();
        let legendary_bin: String = legendary_client::get_legendary_exe()
            .to_string_lossy()
            .to_string();

        let monarch: MonarchSettings = MonarchSettings {
            monarch_home: home_path_str,
            game_folder: default_game_folder_str,
            run_on_startup: false,
            send_logs: false,
            start_minimized: false,
            umu_bin,
            steamcmd_bin,
            legendary_bin,
        };

        let quicklaunch: QuicklaunchSettings = QuicklaunchSettings {
            close_shortcut: String::from("Esc"),
            open_shortcut: String::from("Control+Enter"),
            enabled: true,
            size: String::from("medium"),
        };

        let steam: LauncherSettings = LauncherSettings {
            game_folders: Vec::new(),
            manage: false,
            username: String::new(),
            twofa: false,
            custom_data: String::new(),
        };

        let epic: LauncherSettings = LauncherSettings {
            game_folders: Vec::new(),
            manage: false,
            username: String::new(),
            twofa: false,
            custom_data: String::new(),
        };

        Self {
            settings_path,
            monarch,
            quicklaunch,
            steam,
            epic,
        }
    }
}

/// Function to do unsafe read of SETTINGS_STATE
pub fn get_settings() -> Result<Arc<RwLock<Settings>>> {
    match MONARCH_STATE.read() {
        Ok(state) => Ok(state.get_settings_ptr()),
        Err(e) => {
            bail!("monarch_settings::get_settings() Failed to aqcuire read lock on MONARCH_STATE | Err: {e}")
        }
    }
}

/*
* ----- Misc functions related to managing settings in Monarch -----
*/

/// Checks that a settings.toml file exists, otherwise attempts to create new file and populate
/// with default settings
pub fn init() -> Result<()> {
    let path: PathBuf = get_settings_path().with_context(|| "monarch_settings::init() -> ")?;

    if !path_exists(&path) {
        // If settings.toml doesn't exist, create a new file and write default settings
        if let Err(e) = set_default_settings() {
            bail!("monarch_settings::init() -> {:?}", e);
        }
    }

    if let Ok(mut settings) = read_settings() {
        if !valid_settings(&mut settings) {
            println!("Invalid settings detected in settings.toml!");
            bail!("monarch_settings::init() Invalid settings detected in settings.toml!")
        }
    }

    Ok(())
}

/// Writes default settings to settings.toml
pub fn set_default_settings() -> Result<Settings> {
    let settings: Settings = Settings::default();
    //set_settings_state(settings.clone());

    let path: PathBuf =
        get_settings_path().with_context(|| "monarch_settings::set_default_settings() -> {}")?;

    if !path_exists(&path) {
        create_dir(path.parent().unwrap())
            .with_context(|| "monarch_settings::set_default_settings() -> {}")?;
    }

    write_toml_content(&path, settings.clone().into())
        .with_context(|| "monarch_settings::set_default_settings() -> {}")?;

    Ok(settings)
}

/*
* ----- Frontend settings fuctionality -----
*/

/// Write settings to file where header is the "header" you want to change under,
/// key is the name of the setting and value is the new value the setting should have.
pub fn write_settings(settings: &Settings) -> Result<()> {
    write_toml_content(
        &PathBuf::from(&settings.settings_path),
        settings.clone().into(),
    )
    .with_context(|| "monarch_settings::write_settings() -> {}")?;
    Ok(())
}

/// Writes changes to settings.toml
fn write_toml_content(path: &Path, table: Table) -> Result<()> {
    if let Err(e) = fs::write(path, table.to_string()) {
        bail!("monarch_settings::write_toml_content() Something went wrong while writing settings to settings.toml | Err: {e}");
    }
    Ok(())
}

/*
* ----- settings.rs shit -----
*/

/// Read all settings from file
pub fn read_settings() -> Result<Table> {
    let path: PathBuf =
        get_settings_path().with_context(|| "monarch_settings::read_settings() -> ")?;
    read_settings_content(&path).with_context(|| "monarch_settings::read_settings() -> ")
}

/// Parses content in settings.toml
fn read_settings_content(file: &PathBuf) -> Result<Table> {
    let content: String = fs::read_to_string(file).with_context(|| {
        format!(
            "monarch_settings::read_settings_content() Error reading: {path} | Err: ",
            path = file.as_path().display()
        )
    })?;

    if !content.is_empty() {
        return parse_table(content)
            .with_context(|| "monarch_settings::read_settings_content() -> ");
    }

    Ok(Table::new())
}

/// Returns String content as TOML Table
fn parse_table(content: String) -> Result<Table> {
    content.parse::<Table>().with_context(|| {
        "monarch_settings::parse_table() Failed to parse content in settings.toml! | Err"
    })
}

/*
* ----- Lots of stuff related to verifying that settings written to / read from settings.toml are valid. -----
*/

/// Main function for verifying that Monarch settings are valid.
/// TODO: Come back and implement tighter checks on settings.
fn valid_settings(settings: &mut Table) -> bool {
    match settings.get("settings_path") {
        Some(settings_path) => {
            let correct_path = get_settings_path().unwrap().to_str().unwrap().to_string();

            if settings_path.as_str().unwrap() != correct_path {
                error!("monarch_settings::valid_settings() Error while validating settings! | Err: settings_path does not match!");
                info!(
                    "monarch_settings::valid_settings() Setting correct path: {}",
                    correct_path
                );
                settings.insert(
                    "settings_path".to_string(),
                    toml::Value::String(correct_path),
                );
                return false;
            }
        }
        None => {
            error!("monarch_settings::valid_settings() Error while validating settings! | Err: Missing settings_path!");
            info!("monarch_settings::valid_settings() Attempting to add settings_path");

            let correct_path = get_settings_path().unwrap().to_str().unwrap().to_string();
            settings.insert(
                "settings_path".to_string(),
                toml::Value::String(correct_path.clone()),
            );

            if let Err(e) = write_toml_content(&PathBuf::from(correct_path), settings.clone()) {
                error!("monarch_settings::valid_settings() Failed to write correct settings to settings.toml! | Err: {e}");
                return false;
            }
        }
    }
    // Validate one section of the settings at the time
    match settings.get("monarch") {
        Some(_monarch_settings) => {}
        None => {
            error!("monarch_settings::valid_settings() Error while validating settings! | Err: Missing [monarch] header!");
            return false;
        }
    }
    match settings.get("quicklaunch") {
        Some(_quicklaunch_settings) => {}
        None => {
            error!("monarch_settings::valid_settings() Error while validating settings! | Err: Missing [quicklaunch] header!");
            return false;
        }
    }
    match settings.get("steam") {
        Some(_steam_settings) => {}
        None => {
            error!("monarch_settings::valid_settings() Error while validating settings! | Err: Missing [steam] header!");
            return false;
        }
    }
    match settings.get("epic") {
        Some(_epic_settings) => {}
        None => {
            error!("monarch_settings::valid_settings() Error while validating settings! | Err: Missing [epic] header!");
            return false;
        }
    }
    true
}
