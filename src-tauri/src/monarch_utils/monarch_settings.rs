use core::result::Result;
use std::fs;
use std::path::PathBuf;
use log::error;
use toml::{map::Map, Table, Value};

use super::monarch_fs::{get_settings_path, path_exists};

/// Writes default settings to settings.ini
pub fn set_default_settings() -> Result<(), String> {
    let path: PathBuf = get_settings_path().unwrap();

    if !path_exists(path.clone()) {
        if let Err(e) = fs::File::create(path.clone()) {
            error!("Failed to create new file: {} | Message: {:?}", path.display(), e);
            return Err("Failed to create new settings.ini!".to_string())
        }
    }

    return Ok(())
}

/// Write settings to file where header is the "header" you want to change under,
/// key is the name of the setting and value is the new value the setting should have.
pub fn write_settings(header: &str, key: &str, value: &str) -> Result<(), String> {
    match get_settings_path() {
        Ok(path) => {
            return write_settings_content(path, header, key, value)
        }
        Err(e) => {
            error!("Failed to get path to settings.ini! | Message: {:?}", e);
            return Err("Failed to get path to settings.ini!".to_string())
        }
    }
}

/// Read all settings from file
pub fn read_settings() -> Result<Table, String> {
    match get_settings_path() {
        Ok(path) => {
            return read_settings_content(path)        
        }
        Err(e) => {
            error!("Failed to get path to settings.ini! | Message: {:?}", e);
            return Err("Failed to get path to settings.ini!".to_string())
        }
    }
}

/// Writes setting to settings.toml
fn write_settings_content(file: PathBuf, header: &str, key: &str, value: &str) -> Result<(), String> {
    match read_settings_content(file) {
        Ok(mut settings) => {
            let mut settings_sec = read_settings_section(header, settings);
            settings_sec.insert(key.into(), value.into());
            settings.insert(header.into(), settings_sec.into());
            
            Ok(())
        }
        Err(e) => {
            error!("Failed to read settings from settings.toml! | Message: {:?}", e);
            return Err("Failed to read settings.toml!".to_string())
        }
    }
}

/// Parses content in settings.toml
fn read_settings_content(file: PathBuf) -> Result<Map<String, Value>, String> {
    match fs::read_to_string(&file) {
        Ok(content) => {
            match Table::try_from(content) {
                Ok(settings) => {
                    return Ok(settings)
                }
                Err(e) => {
                    error!("Failed to parse content in settings.toml! | Message: {:?}", e);
                    return Err("Failed to parse settings.toml content!".to_string())
                }
            }
        }
        Err(e) => {
            error!("Failed to read content from: {} | Message: {:?}", file.display(), e);
            return Err("Failed to read settings.toml!".to_string())
        }
    }
}

/// Returns a speicif section in settings TOML Table
fn read_settings_section(header: &str, settings: Map<String, Value>) -> Result<Map<String, Value>, String> {
    match settings.get_mut(header) {
        Ok(settings_sec) => {
            match settings_sec.as_table() {
                Some(settings_table) => {
                    return Ok(settings_table.clone())
                }
                None => {
                    error!("Failed to parse section as TOML Table!");
                    return Err("Failed to parse section as table!".to_string())
                }
            }
        }
        Err(e) => {
            error!("Failed to get section in settings: {} | Message: {:?}", header, e);
            return Err("Failed to get section in settings!".to_string())
        }
    }
}