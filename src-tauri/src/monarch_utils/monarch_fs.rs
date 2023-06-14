use std::{process::exit, io, fs};
use serde_json::Value;
use log::{info, error};
use std::env::VarError;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use regex::Regex;
use core::result::Result;

use super::monarch_settings::{MonarchSettings, write_settings};

/*
---------- General functions for filesystem tasks ----------
*/

/// Create Monarch folder in users %appdata% directory
pub fn check_appdata_folder() {
    let appdata_path: Result<PathBuf, VarError> = get_app_data_path();

    match appdata_path {  
        Ok(path) =>  {
            if !path_exists(path.clone()) {
                if let Err(e) = create_dir(path) { // Returns result of creating directory
                    println!("Failed to create Monarch %appdata% folder! | Message: {:?}", e); // Only really useful rn for debugging
                    exit(1);
                }
            }
        }
        Err(e) => { // If Monarch fails to create its own %appdata% directory
            println!("Something went wrong looking for %appdata% folder! \nErr: {:?} \nExiting... ", e); // Only really useful rn for debugging
            exit(1); // Exit out of app!
        }
    }
}

/// Folder to store image resources for game thumbnails etc...
pub fn check_resources_folder() {
    // Can't be bothered rn, and honestly if it passes the appdata check this should pass
    let resources_dir: PathBuf = get_resources_path().unwrap(); 
    let cache_dir: PathBuf = get_resources_cache().unwrap();
    let lib_img_dir: PathBuf = get_resources_library().unwrap();
    let settings_path: PathBuf = get_settings_json_path().unwrap();
        
    if !path_exists(resources_dir.clone()) {
        info!("No resources folder detected! Creating new...");
        if let Err(e) = create_dir(resources_dir.clone()) {
            error!("Failed to create empty folder: {}! | Message: {:?}", resources_dir.display(), e);
            exit(1);
        }
    }

    if !path_exists(cache_dir.clone()) {
        info!("No cache folder detected for thumbnails! Creating new...");
        if let Err(e) = create_dir(cache_dir.clone()) {
            error!("Failed to create empty folder: {}! | Message: {:?}", cache_dir.display(), e);
            exit(1);
        }
    }

    if !path_exists(lib_img_dir.clone()) {
        info!("No library folder detected for thumbnails! Creating new...");
        if let Err(e) = create_dir(lib_img_dir.clone()) {
            error!("Failed to create empty folder: {}! | Message: {:?}", lib_img_dir.display(), e);
            exit(1);
        }
    }

    if !path_exists(settings_path.clone()) {
        info!("No settings.json detected! Creating new...");
        let settings: Value = serde_json::to_value(MonarchSettings::new()).unwrap();
        if let Err(e) = write_settings(settings) {
            error!("Failed to write default settings to: {}! | Message: {:?}", settings_path.display(), e);
            exit(1);
        }
    }
}

/// Gets the users %appdata% or $HOME directory and adds Monarch to the end of it to generate Monarch path
pub fn get_app_data_path() -> Result<PathBuf, VarError> {
    let appdata_path_res: Result<String, VarError>;
    let folder_name: &str;

    if cfg!(windows) {
        appdata_path_res = std::env::var("APPDATA");
        folder_name = "Monarch";
    } else { // Easiest way to avoid "possibly-uninitialized variables"
        appdata_path_res = std::env::var("HOME");
        folder_name= ".monarch";
    }

    match appdata_path_res {
        Ok(appdata_path) => {
            let mut path: PathBuf = PathBuf::from(appdata_path);
            path = path.join(folder_name);
            return Ok(path) 
        },
        Err(e) => Err(e)
    }
}

#[cfg(not(target_os = "windows"))]
// Returns $HOME on unix systems
pub fn get_home_path() -> Result<PathBuf, String> {
    match std::env::var("$HOME") {
        Ok(str_path) => { return Ok(PathBuf::from(str_path)) }
        Err(e) => {
            error!("Failed to get $HOME path! | Message: {:?}", e);
            return Err("Failed to get $HOME path!".to_string())
        }
    }
}

/// Returns path to settings.json
pub fn get_settings_json_path() -> Result<PathBuf, VarError> {
    match get_app_data_path() {
        Ok(mut path) => {
            path.push("settings.json");
            return Ok(path)
        }
        Err(e) => {
            error!("Failed to get %appdata% path! | Message: {:?}", e);
            return Err(e)
        }
    }
}

/// Returns path to library.json
pub fn get_library_json_path() -> Result<PathBuf, VarError> {
    match get_app_data_path() {
        Ok(mut path) => {
            path.push("library.json");
            return Ok(path)
        }
        Err(e) => {
            error!("Failed to get %appdata% path! | Message: {:?}", e);
            return Err(e)
        }
    }
}

/// Returns path to collections.json
pub fn get_collections_json_path() -> Result<PathBuf, VarError> {
    match get_app_data_path() {
        Ok(mut path) => {
            path.push("collections.json");
            return Ok(path)
        }
        Err(e) => {
            error!("Failed to get %appdata% path! | Message: {:?}", e);
            return Err(e)
        }
    }
}

/// Write JSON to file
pub fn write_json_content(content: Value, path: PathBuf) -> io::Result<()> {
    if let Err(e) = fs::write(path.clone(), content.to_string()) {
        error!("Failed to write new library to: {} | Message: {:?}", path.display(), e);
        return Err(e)
    }
    return Ok(())
}

/// Checks whether a given path exists already or not
pub fn path_exists(path: PathBuf) -> bool {
    if Path::new(path.as_path()).exists() {
        return true
    }
    return false
}

/// Attempts to create an empty directory and returns result
pub fn create_dir(path: PathBuf) -> io::Result<()> {
    if let Err(e) = fs::create_dir_all(path.as_path()) {
        error!("Failed to create new directory: {} | Message: {:?}", path.display(), e);
        return Err(e)
    }
    Ok(())
}

/*
---------- Functions related to storing in resources dir ----------
*/

/// Returns path to resources folder.
/// Should never fail during runtime because of init_monarch_fs,
/// but if it does it returns an empty string.
pub fn get_resources_path() -> Result<PathBuf, VarError> {
    match get_app_data_path() {
        Ok(mut path) => {
            path.push("resources");
            return Ok(path)
        }
        Err(e) => {
            error!("Failed to get %appdata% path! | Message: {:?}", e);
            return Err(e)
        }
    }
}

/// Returns path to store temporary images
pub fn get_resources_cache() -> Result<PathBuf, VarError> {
    match get_resources_path() {
        Ok(mut path) => {
            path.push("cache");
            return Ok(path)
        }
        Err(e) => {
            error!("Failed to get %appdata% path! | Message: {:?}", e);
            return Err(e)
        }
    }
}

/// Returns path to store thumbnails for games in library
pub fn get_resources_library() -> Result<PathBuf, VarError> {
    match get_resources_path() {
        Ok(mut path) => {
            path.push("library");
            return Ok(path)
        }
        Err(e) => {
            error!("Failed to get %appdata% path! | Message: {:?}", e);
            return Err(e)
        }
    }
}

/// Clears out old cached thumbnails (Don't like the indentaion level, will come back to rework later)
pub fn clear_cached_thumbnails() {
    match get_resources_cache() {
        Ok(cache) => {
            if let Ok(files) = fs::read_dir(cache) {
                for file in files {
                    if let Ok(file_path) = file {
                        remove_thumbnail(file_path);
                    }       
                }
            }
        }
        Err(e) => {
            error!("Failed to get cache directory! | Message: {}", e);
        }
    }
    
}

/// Removes old cache file if old enough
fn remove_thumbnail(file: DirEntry) {
    if time_to_remove(file.path()) {
        if let Err(e) = fs::remove_file(file.path()) {
            error!("Failed to remove file: {}! | Message: {:?}", file.path().display(), e);
        }
    }
}

/// Checks if it's time to remove cached thumbnail
fn time_to_remove(file: PathBuf) -> bool {
    if let Ok(metadata) = fs::metadata(file) {
        if let Ok(time) = metadata.modified() {
            if let Ok(age) = SystemTime::now().duration_since(time) {
                return age.as_secs() >= 1209600 // Return if file is older than 14 days
            }
        }
    }
    false
}

/// Removes all files in \resources\cache, meant for UI so that user can clear folder if wanted
pub fn clear_all_cache() {
    match get_resources_cache() {
        Ok(resources) => {
            match fs::read_dir(resources.clone()) {
                Ok(files) => {
                    for file in files {
                        match file {
                            Ok(f) => { remove_thumbnail(f); }
                            Err(e) => { error!("Failed to read file! | Message: {:?}", e); }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to get files from folder: {} | Message: {}", resources.display(), e);
                }
            }
        }
        Err(e) => {
            error!("Failed to get resources path! | Message: {}", e);
        }
    }
}

/// Create a name for image file in cache directory
/// Can be used to download image and check if an image already exists
pub fn generate_cache_image_name(name: &str) -> Result<PathBuf, String> {
    let filename: String;

    match generate_image_filename(name) {
        Ok(name) => { filename = name }
        Err(_) => { return Err("Failed to build name from regex!".to_string()) }
    }

    match get_resources_cache() {
        Ok(mut dir) => {
            dir.push(&filename);
            return Ok(dir)
        }
        Err(e) => {
            error!("Failed to get cached thumbnails folder! | Message: {}", e);
            return Err("Failed to get cached thumbnails folder!".to_string())
        }
    }
}

/// Create a name for image file in cache directory
pub fn generate_library_image_name(name: &str) -> Result<PathBuf, String> {
    let filename: String;

    match generate_image_filename(name) {
        Ok(name) => { filename = name }
        Err(_) => { return Err("Failed to build name from regex!".to_string()) }
    }

    match get_resources_library() {
        Ok(mut dir) => {
            dir.push(&filename);
            return Ok(dir)
        }
        Err(e) => {
            error!("Failed to get library thumbnails folder! (generate_library_image_name()) | Message: {}", e);
            return Err("Failed to get library thumbnails library!".to_string())
        }
    }
}

/// Generates a filename without any special characters or spaces
fn generate_image_filename(name: &str) -> Result<String, regex::Error> {
    let mut filename: String = String::from(name);
    filename = filename.replace(" ", "_");

    match Regex::new(r"[^a-zA-Z0-9_]") {
        Ok(re) => {
            filename = re.replace_all(&filename, "").to_string();
            filename.push_str(".jpg");
            return Ok(filename)
        }
        Err(e) => {
            error!("Failed to build new regex! (generate_image_filename()) | Message: {}", e);
            return Err(e)
        }
    }
}