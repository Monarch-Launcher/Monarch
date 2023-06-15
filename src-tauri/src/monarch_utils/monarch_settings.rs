use serde::{Serialize, Deserialize};
use serde_json::{value::Value, json};
use log::{info, error};
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::collections::HashMap;

use super::monarch_fs::{get_settings_json_path, write_json_content};

#[derive(Serialize, Deserialize)]
pub struct MonarchSettings {
    platform_settings: HashMap<String, PlatformSettings> // Store settings with <name, settings>
}

impl MonarchSettings {
    pub fn new() -> Self {
        Self { platform_settings: HashMap::new() }
    }

    pub fn _get_launcher_settings(self, name: &str) -> PlatformSettings {
        self.platform_settings.get(name).unwrap().clone()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlatformSettings {
    library_file_path: String, // File location of a launchers library (ex. Steam: library.vdf)
    default_folder_path: String, // Default game folder location for launcher
    other_game_folders: Vec<String>, // Other folder locations to look for games
}

impl PlatformSettings {
    pub fn _get_library_file_path(self) -> String {
        return self.library_file_path
    }

    pub fn _set_library_file_path(mut self, path: &str) {
        self.library_file_path = path.to_string()
    }

    pub fn _get_default_library_path(self) -> String {
        return self.default_folder_path
    }

    pub fn _set_default_library_path(mut self, path: &str) {
        self.default_folder_path = path.to_string()
    }

    pub fn _get_other_game_folders(self) -> Vec<String> {
        return self.other_game_folders
    }

    pub fn _add_other_game_folder(mut self, path: &str) {
        self.other_game_folders.push(path.to_string())
    }

    pub fn _remove_other_game_folder(mut self, path: &str) {
        let index = self.other_game_folders.iter().position(|x| *x == path.to_string()).unwrap();
        self.other_game_folders.remove(index);
    } 


}

/// Read settings from JSON file
pub fn read_settings() -> Result<MonarchSettings, String> {
    match get_settings_json_path() {
        Ok(path) => {
            return parse_json_content(path)
        }
        Err(e) => {
            error!("Failed to get path to settings.json! | Message: {:?}", e);
            return Err("Failed to get path to settings.json!".to_string())
        }
    }
}

/// Converts json format to MonarchSettings struct
fn parse_json_content(path: PathBuf) -> Result<MonarchSettings, String> {
    match fs::File::open(path.clone()) {
        Ok(reader) => {
            match serde_json::from_reader::<File, MonarchSettings>(reader) {
                Ok(content) => {
                    return Ok(content)
                }
                Err(e) => {
                    error!("Failed to parse json to MonarchSettings! | Message: {:?}", e);
                    return Err("Failed to parse json content to settings!".to_string())
                }
            }
        }
        Err(e) => {
            error!("Failed to open file: {} | Message: {:?}", path.display(), e);
            return Err("Failed to open settings.json file!".to_string())
        }
    }
}

/// Writes settings (in json format) to settings.json which is where Monarch stores it's settings.
pub fn write_settings(content: Value) -> Result<(), String> {
    match get_settings_json_path() {
        Ok(path) => {
            if let Err(e) = write_json_content(content, path) {
                error!("Failed to write content to settings.json! (write_settings()) | Message: {:?}", e);
                return Err("Failed to write content to settings.json!".to_string())
            }
            return Ok(())
        }
        Err(e) => {
            error!("Failed to get path to settings.json! (write_settings()) | Message: {:?}", e);
            return Err("Failed to get path to settings.json!".to_string());
        }
    }
}

