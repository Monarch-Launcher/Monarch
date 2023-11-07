use core::result::Result;
use log::{debug, error};
use once_cell::sync::Lazy;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Table;

use super::monarch_fs::{get_app_data_path, get_settings_path, path_exists};

// Create a global variable containing the current state of settings according to Monarch backend
static mut SETTINGS_STATE: Lazy<Settings> = Lazy::<Settings>::new(|| Settings::new());

/// Struct for storing a persistent state of settings
struct Settings {
    settings: Table,
}

impl Settings {
    /// Returns new blank Settings struct
    fn new() -> Self {
        Self {
            settings: Table::new(),
        }
    }
}

/// Function to do unsafe write of SETTINGS_STATE
fn set_settings_state(settings: Table) {
    unsafe {
        SETTINGS_STATE.settings = settings;
    }
}

/// Function to do unsafe read of SETTINGS_STATE
fn get_settings_state() -> Table {
    unsafe { SETTINGS_STATE.settings.clone() }
}

/// Checks that a settings.toml file exists, otherwise attempts to create new file and populate
/// with default settings
pub fn init() -> Result<(), String> {
    match get_settings_path() {
        Ok(path) => {
            if !path_exists(&path) {
                // If settings.toml doesn't exist, create a new file and fill
                // with default settings
                if let Err(e) = set_default_settings() {
                    error!("monarch_settings::init() failed! | Error: {e}");
                    return Err("Failed to write default settings to settings.toml".to_string());
                }
            }
            Ok(())
        }
        Err(e) => {
            error!("monarch_settings::init() failed! Cannot get path to settings.toml! | Error: {e}");
            Err("Failed to get path to settings.toml!".to_string())
        }
    }
}

/// Writes default settings to settings.toml
pub fn set_default_settings() -> Result<Table, Table> {
    let path: PathBuf;
    let settings: Table = get_default_settings();
    set_settings_state(settings.clone());

    match get_settings_path() {
        Ok(settings_path) => path = settings_path,
        Err(e) => {
            error!("monarch_settings::set_default_settings() failed! Cannot get path to settings.toml! | Error: {e}");
            return Err(settings);
        }
    }

    if !path_exists(&path) {
        if let Err(e) = fs::File::create(&path) {
            error!(
                "monarch_settings::set_default_settings() failed! Something went wrong while trying to create new file: {dir} | Error: {e}",
                dir = path.display()
            );
            return Err(settings);
        }
    }

    write_toml_content(&path, settings)
}

/*
* ----- Frontend settings fuctionality -----
*/

/// Write settings to file where header is the "header" you want to change under,
/// key is the name of the setting and value is the new value the setting should have.
pub fn write_settings(settings: Table) -> Result<Table, Table> {
    match get_settings_path() {
        Ok(path) => write_toml_content(&path, settings),
        Err(e) => {
            error!("monarch_settings::write_settings() failed! Cannot get path to settings.toml! | Error: {e}");
            Err(get_settings_state())
        }
    }
}

/// Writes changes to settings.toml
fn write_toml_content(path: &Path, table: Table) -> Result<Table, Table> {
    if let Err(e) = fs::write(path, table.to_string()) {
        error!("monarch_settings::write_toml_content() failed! Something went wrong while writing settings to settings.toml | Error: {e}");
        return Err(get_settings_state());
    }
    set_settings_state(table);
    Ok(get_settings_state())
}

/// Read all settings from file
pub fn read_settings() -> Result<Table, String> {
    match get_settings_path() {
        Ok(path) => read_settings_content(&path),
        Err(e) => {
            error!("monarch_settings::read_settings() failed! Cannot get path to settings.toml! | Error: {e}");
            Err("Failed to get path to settings.toml!".to_string())
        }
    }
}

/*
* ----- Backend functionality -----
*
* This section is mostly helpful to read smaller parts of settings for some backend
* functionality when needed and not meant to be run a lot.
*/

/// Returns Table of settings under [monarch]
pub fn get_monarch_settings() -> Option<toml::Value> {
    get_settings_state().get("monarch").cloned()
}

/// Returns Table of settings under [quicklaunch]
pub fn get_quicklaunch_settings() -> Option<toml::Value> {
    get_settings_state().get("quicklaunch").cloned()
}

/// Returns Table of settings under [steam]
pub fn get_steam_settings() -> Option<toml::Value> {
    get_settings_state().get("steam").cloned()
}

/// Returns Table of settings under [epic]
pub fn get_epic_settings() -> Option<toml::Value> {
    get_settings_state().get("epic").cloned()
}

/*
* ----- settings.rs shit -----
*/

/// Parses content in settings.toml
fn read_settings_content(file: &PathBuf) -> Result<Table, String> {
    match fs::read_to_string(file) {
        Ok(content) => {
            if !content.is_empty() {
                return parse_table(content);
            }

            Ok(Table::new())
        }
        Err(e) => {
            error!("monarch_settings::read_settings_content() failed! Error reading: {path} | Error: {e}", path = file.display());
            Err("Failed to read settings.toml!".to_string())
        }
    }
}

/// Returns String content as TOML Table
fn parse_table(content: String) -> Result<Table, String> {
    match content.parse::<Table>() {
        Ok(settings) => Ok(settings),
        Err(e) => {
            error!("monarch_settings::parse_table() failed! Failed to parse content in settings.toml! | Error: {e}");
            Err("Failed to parse settings.toml content!".to_string())
        }
    }
}

/// Returns default Monarch settings in the form of a TOML Table.
/// .into() is used to avoid ugly syntax of e.g. Value::Boolean(true) - instead becomes true.into()
fn get_default_settings() -> Table {
    let mut settings: Table = Table::new();

    let mut monarch: Table = Table::new();
    let appdata_path = get_app_data_path().unwrap();
    let appdata_path_str = appdata_path.to_str().unwrap();
    let default_game_folder = appdata_path.join("games");
    let default_game_folder_str = default_game_folder.to_str().unwrap();
    monarch.insert("monarch_home".to_string(), appdata_path_str.into());
    monarch.insert("send_logs".to_string(), true.into());
    monarch.insert("run_on_startup".to_string(), false.into());
    monarch.insert("start_minimized".to_string(), false.into());
    monarch.insert(
        "game_folders".to_string(),
        vec![default_game_folder_str].into(),
    );

    let mut quicklaunch_settings: Table = Table::new();
    quicklaunch_settings.insert("enabled".to_string(), true.into());
    quicklaunch_settings.insert("open_shortcut".to_string(), "Super+Enter".into());
    quicklaunch_settings.insert("close_shortcut".to_string(), "Esc".into());
    quicklaunch_settings.insert("size".to_string(), "medium".into());

    let mut steam_settings: Table = Table::new();
    steam_settings.insert("game_folders".to_string(), Vec::<String>::new().into());
    steam_settings.insert("manage".to_string(), false.into());
    steam_settings.insert("username".to_string(), "".into());

    let mut epic_settings: Table = Table::new();
    epic_settings.insert("game_folders".to_string(), Vec::<String>::new().into());
    epic_settings.insert("manage".to_string(), false.into());
    epic_settings.insert("username".to_string(), "".into());

    settings.insert("monarch".to_string(), monarch.into());
    settings.insert("quicklaunch".to_string(), quicklaunch_settings.into());
    settings.insert("steam".to_string(), steam_settings.into());
    settings.insert("epic".to_string(), epic_settings.into());

    settings
}
