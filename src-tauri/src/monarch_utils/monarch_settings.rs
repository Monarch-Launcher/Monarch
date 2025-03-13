use anyhow::{bail, Context, Result};
use log::error;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use toml::Table;

use super::monarch_fs::{create_dir, generate_monarch_home, get_settings_path, path_exists};
use crate::monarch_games::monarch_client::generate_default_folder;

// Create a global variable containing the current state of settings according to Monarch backend.
// Allows for fewer reads of settings.toml by storing in program memory.
static mut SETTINGS_STATE: Lazy<Settings> = Lazy::<Settings>::new(Settings::default);

/*
* ----- Settings struct related ------
*/

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherSettings {
    pub game_folders: Vec<String>,
    pub manage: bool,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonarchSettings {
    pub game_folder: String,
    pub monarch_home: String,
    pub run_on_startup: bool,
    pub send_logs: bool,
    pub start_minimized: bool,
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
    pub monarch: MonarchSettings,
    pub quicklaunch: QuicklaunchSettings,
    pub steam: LauncherSettings,
    pub epic: LauncherSettings,
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

        let monarch: MonarchSettings = MonarchSettings {
            monarch_home: home_path_str,
            game_folder: default_game_folder_str,
            run_on_startup: false,
            send_logs: false,
            start_minimized: false,
        };

        let quicklaunch: QuicklaunchSettings = QuicklaunchSettings {
            close_shortcut: String::from("Esc"),
            open_shortcut: String::from("Super+Enter"),
            enabled: true,
            size: String::from("medium"),
        };

        let steam: LauncherSettings = LauncherSettings {
            game_folders: Vec::new(),
            manage: false,
            username: String::new(),
        };

        let epic: LauncherSettings = LauncherSettings {
            game_folders: Vec::new(),
            manage: false,
            username: String::new(),
        };

        Self {
            monarch,
            quicklaunch,
            steam,
            epic,
        }
    }
}

/// Function to do unsafe write of SETTINGS_STATE
pub fn set_settings_state(settings: Settings) {
    unsafe {
        *SETTINGS_STATE = settings;
    }
}

/// Function to do unsafe read of SETTINGS_STATE
pub fn get_settings_state() -> Settings {
    unsafe { SETTINGS_STATE.clone() }
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

    if let Ok(settings) = read_settings() {
        if !valid_settings(&settings) {
            println!("Invalid settings detected in settings.toml!");
            bail!("monarch_settings::init() Invalid settings detected in settings.toml!")
        }
        // Set SETTINGS_STATE to settings from settings.toml
        set_settings_state(settings.try_into().unwrap());
    }

    Ok(())
}

/// Writes default settings to settings.toml
pub fn set_default_settings() -> Result<Settings> {
    let settings: Settings = Settings::default();
    set_settings_state(settings.clone());

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
pub fn write_settings(settings: Settings) -> Result<Settings> {
    let path = get_settings_path().with_context(|| "monarch_settings::write_settings() -> {}")?;
    write_toml_content(&path, settings.clone().into())
        .with_context(|| "monarch_settings::write_settings() -> {}")?;
    Ok(settings)
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
* ----- Lots of stuff related to verifying that settings written to / read from settings.toml are
* valid. -----
*/

/// Main function for verifying that Monarch settings are valid.
/// TODO: Come back and implement tighter checks on settings.
fn valid_settings(settings: &Table) -> bool {
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
