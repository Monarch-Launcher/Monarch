use core::result::Result;
use std::fs;
use std::path::PathBuf;
use log::{error, info};
use toml::{map::Map, Table, Value};

use super::monarch_fs::{get_settings_path, path_exists};

/// Writes default settings to settings.ini
pub fn set_default_settings() -> Result<(), String> {
    let path: PathBuf = get_settings_path().unwrap();
    let settings: Table = Table::new();

    if !path_exists(path.clone()) {
        if let Err(e) = fs::File::create(path.clone()) {
            error!("Failed to create new file: {} | Message: {:?}", path.display(), e);
            return Err("Failed to create new settings.toml!".to_string())
        }
    }

    if let Err(e) = fs::write(path, settings.to_string()) {
        error!("Failed to write default settings to settings.toml! | Message: {:?}", e);
        return Err("Failed to write default settings to settings.toml!".to_string())
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
            error!("Failed to get path to settings.toml! | Message: {:?}", e);
            return Err("Failed to get path to settings.toml!".to_string())
        }
    }
}

/// Writes setting to settings.toml
fn write_settings_content(file: PathBuf, header: &str, key: &str, value: &str) -> Result<(), String> {
    match read_settings_content(&file) {
        Ok(mut settings) => {
            if let Ok(mut settings_sec) = read_settings_section(header, &settings) {
                settings_sec.insert(key.into(), value.into());
                settings.insert(header.into(), settings_sec.into());
                return write_toml_content(file, settings)
            }

            // If no section exists, create a new one
            let mut settings_sec: Table = Table::new();
            settings_sec.insert(key.into(), value.into());
            settings.insert(header.into(), settings_sec.into());
            return write_toml_content(file, settings)
        }
        Err(e) => {
            error!("Failed to read settings from settings.toml! | Message: {:?}", e);
            return Err("Failed to read settings.toml!".to_string())
        }
    }
}

/// Writes changes to 
fn write_toml_content(path: PathBuf, content: Table) -> Result<(), String> {
    if let Err(e) = fs::write(path, content.to_string()) {
        error!("Failed to write settings to settings.toml | Message: {:?}",e );
        return Err("Failed to write changes to settings.toml!".to_string())
    }
    Ok(())
}

/// Read all settings from file
pub fn read_settings() -> Result<Table, String> {
    match get_settings_path() {
        Ok(path) => {
            return read_settings_content(&path)        
        }
        Err(e) => {
            error!("Failed to get path to settings.ini! | Message: {:?}", e);
            return Err("Failed to get path to settings.toml!".to_string())
        }
    }
}

/// Parses content in settings.toml
fn read_settings_content(file: &PathBuf) -> Result<Table, String> {
    match fs::read_to_string(&file) {
        Ok(content) => {
            if !content.is_empty() {
                return parse_table(content)
            }

            return Ok(Table::new())
        }
        Err(e) => {
            error!("Failed to read content from: {} | Message: {:?}", file.display(), e);
            return Err("Failed to read settings.toml!".to_string())
        }
    }
}

/// Returns String content as TOML Table
fn parse_table(content: String) -> Result<Table, String> {
    match Table::try_from(content.clone()) {
        Ok(settings) => {
            return Ok(settings)
        }
        Err(e) => {
            error!("Failed to parse content in settings.toml! | Message: {:?}", e);
            info!("CONTENT: {:?}", content);
            return Err("Failed to parse settings.toml content!".to_string())
        }
    }
}

/// Returns a speicif section in settings TOML Table
fn read_settings_section(header: &str, settings: &Map<String, Value>) -> Result<Table, String> {
    match settings.get(header) {
        Some(settings_sec) => {
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
        None => {
            error!("Failed to get section in settings: {} ", header);
            return Err("Failed to get section in settings!".to_string())
        }
    }
}