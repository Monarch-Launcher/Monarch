use core::result::Result;
use std::fs;
use std::{path::PathBuf, collections::HashMap};
use ini::Ini;
use log::error;
use serde::{Serialize, Deserialize};

use super::monarch_fs::{get_settings_path, path_exists};

// This struct is currently only meant as a parsing tool for ini to make it easier 
// to serialize and deserialize
#[derive(Serialize, Deserialize)]
pub struct MonarchSettings {
    settings: HashMap<String, HashMap<String, String>>
}

impl MonarchSettings {
    /// Parses Ini into MonarchSettings struct
    pub fn from(ini: Ini) -> Self {
        let mut map: HashMap<String, HashMap<String, String>> = HashMap::new();
        
        for (sec, prop) in ini.iter() {
            if sec.is_some() {
                let mut inner_map: HashMap<String, String> = HashMap::new();

                for (k, v) in prop.iter() {
                    inner_map.insert(k.to_string(), v.to_string());
                }
                map.insert(sec.unwrap().to_string(), inner_map);
            }
        }
        return Self { settings: map }
    }
}

/// Writes default settings to settings.ini
pub fn set_default_settings() -> Result<(), String> {
    let settings: Ini = Ini::new();
    let path: PathBuf = get_settings_path().unwrap();

    if !path_exists(path.clone()) {
        if let Err(e) = fs::File::create(path.clone()) {
            error!("Failed to create new file: {} | Message: {:?}", path.display(), e);
            return Err("Failed to create new settings.ini!".to_string())
        }
    }
    
    if let Err(e) = settings.write_to_file(path) {
        error!("Failed to write default settings to settings.ini! | Message: {:?}", e);
        return Err("Failed to write default settings to settings.ini!".to_string())
    }

    return Ok(())
}

/// Write settings to file where header is the "header" you want to change under,
/// key is the name of the setting and value is the new value the setting should have.
pub fn write_settings(header: &str, key: &str, value: &str) -> Result<(), String> {
    match get_settings_path() {
        Ok(path) => {
            match Ini::load_from_file(path) {
                Ok(mut settings) => {
                    settings.with_section(Some(header))
                            .set(key, value);
                    return Ok(())
                }
                Err(e) => {
                    error!("Failed to load settings from settings.ini! | Message: {:?}", e);
                    return Err("Failed to load settings from settings.ini!".to_string())
                }
            }
        }
        Err(e) => {
            error!("Failed to get path to settings.ini! | Message: {:?}", e);
            return Err("Failed to get path to settings.ini!".to_string())
        }
    }
}

/// Read all settings from file
pub fn read_settings() -> Result<MonarchSettings, String> {
    match get_settings_path() {
        Ok(path) => {
            match Ini::load_from_file(path) {
                Ok(settings) => { return Ok(MonarchSettings::from(settings)) }
                Err(e) => {
                    error!("Failed to load settings from settings.ini! | Message: {:?}", e);
                    return Err("Failed to load settings from settings.ini!".to_string())
                }
            }
        }
        Err(e) => {
            error!("Failed to get path to settings.ini! | Message: {:?}", e);
            return Err("Failed to get path to settings.ini!".to_string())
        }
    }
}