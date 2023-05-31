use serde::{Serialize, Deserialize};
use serde_json::{value::Value, json};
use log::{info, error};
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::collections::HashMap;

use super::monarch_fs::get_settings_json_path;

#[derive(Serialize, Deserialize)]
pub struct MonarchSettings {
    platform_settings: HashMap<String, PlatformSettings> // Store settings with <name, settings>
}

impl MonarchSettings {
    /// Settings for specific game
    pub fn get_game_settings(&self, platform: &str, game: &str) -> Option<&GameSettings> {
        match &self.platform_settings.get(platform) {
            Some(platform_settings) => {
                return platform_settings.game_settings.get(game)
            }
            None => return None
        }
    }
    pub fn set_game_settings(mut self, platform: &str, game: &str, new_settings: GameSettings) {
        self.platform_settings.entry(platform.to_string())
                              .and_modify(|ps| { 
                                           ps.game_settings.entry(game.to_string())
                                                           .and_modify(|gs| *gs = new_settings); });
        }
    }

#[derive(Serialize, Deserialize)]
struct PlatformSettings {
    game_settings: HashMap<String, GameSettings>, // Store settings with <name, settings>
    library_file_path: String,
    default_folder_path: String
}

#[derive(Serialize, Deserialize)]
struct GameSettings {
    launch_args: Vec<String>,
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

pub fn write_setting() {

}

