use core::result::Result;
use log::{error, info, warn};
use regex::Regex;
use serde_json::Value;
use std::env::VarError;
use std::path::{Path, PathBuf};
use std::{fs, io, process::exit};

use super::monarch_settings::set_default_settings;

/*
---------- General functions for filesystem tasks ----------
*/

/// Create Monarch folder in users %appdata% directory
pub fn check_appdata_folder() {
    let appdata_path: Result<PathBuf, VarError> = get_appdata_path();

    match appdata_path {
        Ok(path) => {
            if !path_exists(&path) {
                if let Err(e) = create_dir(&path) {
                    // Returns result of creating directory
                    println!("monarch_fs::check_appdata_folder() failed! Failed to create Monarchs %appdata% / $HOME folder! | Error: {e}"); // Only really useful rn for debugging
                    exit(1);
                }
            }
        }
        Err(e) => {
            // If Monarch fails to create its own %appdata% directory
            println!("monarch_fs::get_app_data_path() failed! Something went wrong looking for %appdata% / $HOME folder! \nErr: {e} \nExiting... "); // Only really useful rn for debugging
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
    let settings_path: PathBuf = get_settings_path().unwrap();

    if !path_exists(&resources_dir) {
        warn!("No resources folder detected!");
        info!("Creating folder...");
        if let Err(e) = create_dir(&resources_dir) {
            error!(
                "monarch_fs::check_resources_folder() failed! Something went wrong trying to create empty folder: {dir}! | Error: {e}",
                dir = resources_dir.display()
            );
            exit(1);
        }
    }

    if !path_exists(&cache_dir) {
        warn!("No cache folder detected in resources/ !");
        info!("Creating folder...");
        if let Err(e) = create_dir(&cache_dir) {
            error!(
                "monarch_fs::check_resources_folder() failed! Something went wrong trying to create empty folder: {dir}! | Error: {e}",
                dir = cache_dir.display()
            );
            exit(1);
        }
    }

    if !path_exists(&lib_img_dir) {
        warn!("No library folder detected in resources/ !");
        info!("Creating folder...");
        if let Err(e) = create_dir(&lib_img_dir) {
            error!(
                "monarch_fs::check_resources_folder() failed! Something went wrong trying to create empty folder: {dir}! | Error: {e}",
                dir = lib_img_dir.display()
            );
            exit(1);
        }
    }

    if !path_exists(&settings_path) {
        warn!("No settings.toml detected!");
        info!("Creating new settings.toml with default settings...");
        if let Err(e) = set_default_settings() {
            error!(
                "monarch_fs::check_resources_folder() failed! Something went wrong trying to write default settings to: {dir}! | Error: {e}",
                dir = settings_path.display()
            );
            exit(1);
        }
    }
}

/// Gets the users %appdata% or $HOME directory and adds Monarch to the end of it to generate Monarch path
/// returns either $HOME/.monarch or %appdata%/Monarch
pub fn get_appdata_path() -> Result<PathBuf, VarError> {
    let appdata_path_res: Result<String, VarError>;
    let folder_name: &str;

    if cfg!(windows) {
        // If on windows
        appdata_path_res = std::env::var("APPDATA");
        folder_name = "Monarch";
    } else {
        // If on other OS
        appdata_path_res = std::env::var("HOME");
        folder_name = ".monarch";
    }

    match appdata_path_res {
        Ok(appdata_path) => {
            let mut path: PathBuf = PathBuf::from(appdata_path);
            path = path.join(folder_name);
            return Ok(path);
        }
        Err(e) => Err(e),
    }
}

#[cfg(not(target_os = "windows"))]
// Returns $HOME on unix systems. Useful for looking for directories like .Steam
pub fn get_home_path() -> Result<PathBuf, String> {
    match std::env::var("$HOME") {
        Ok(str_path) => return Ok(PathBuf::from(str_path)),
        Err(e) => {
            error!("monarch_fs::get_home_path() failed! | Error: {e}");
            return Err("Failed to get $HOME path!".to_string());
        }
    }
}

/// Returns path to settings.json
pub fn get_settings_path() -> Result<PathBuf, VarError> {
    match get_appdata_path() {
        Ok(mut path) => {
            path.push("settings.toml");
            return Ok(path);
        }
        Err(e) => {
            error!("monarch_fs::get_settings_path() failed! Something went wrong while getting %appdata%/$HOME path. | Error: {e}");
            return Err(e);
        }
    }
}

/// Returns path to library.json
pub fn get_library_json_path() -> Result<PathBuf, VarError> {
    match get_appdata_path() {
        Ok(mut path) => {
            path.push("library.json");
            return Ok(path);
        }
        Err(e) => {
            error!("monarch_fs::get_library_json_path() failed! Something went wrong while getting %appdata%/$HOME path. | Error: {e}");
            return Err(e);
        }
    }
}

/// Returns path to collections.json
pub fn get_collections_json_path() -> Result<PathBuf, VarError> {
    match get_appdata_path() {
        Ok(mut path) => {
            path.push("collections.json");
            return Ok(path);
        }
        Err(e) => {
            error!("monarch_fs::get_collections_json_path() failed! Something went wrong while getting %appdata%/$HOME path. | Error: {e}");
            return Err(e);
        }
    }
}

/// Write JSON to file
pub fn write_json_content(content: Value, path: &Path) -> io::Result<()> {
    if let Err(e) = fs::write(path, content.to_string()) {
        error!(
            "monarch_fs::write_json_content() failed! Something went wrong trying to write new library to: {file} | Error: {e}",
            file = path.display()
        );
        return Err(e);
    }
    return Ok(());
}

/// Abstraction to check whether a given path exists already or not
pub fn path_exists(path: &Path) -> bool {
    Path::new(path).exists()
}

/// Attempts to create an empty directory and returns result
pub fn create_dir(path: &Path) -> io::Result<()> {
    if let Err(e) = fs::create_dir_all(path) {
        error!(
            "monarch_fs::create_dir() failed! Something went wrong while creating new directory: {dir} | Error: {e}",
            dir = path.display()
        );
        return Err(e);
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
    match get_appdata_path() {
        Ok(mut path) => {
            path.push("resources");
            return Ok(path);
        }
        Err(e) => {
            error!("monarch_fs::get_resources_path() failed! Something went wrong while getting %appdata%/$HOME path. | Error: {e}");
            return Err(e);
        }
    }
}

/// Returns path to store temporary images
pub fn get_resources_cache() -> Result<PathBuf, VarError> {
    match get_resources_path() {
        Ok(mut path) => {
            path.push("cache");
            return Ok(path);
        }
        Err(e) => {
            error!("monarch_fs::get_resources_cache() failed! Something went wrong while getting resources/ path. | Error: {e}");
            return Err(e);
        }
    }
}

/// Returns path to store thumbnails for games in library
pub fn get_resources_library() -> Result<PathBuf, VarError> {
    match get_resources_path() {
        Ok(mut path) => {
            path.push("library");
            return Ok(path);
        }
        Err(e) => {
            error!("monarch_fs::get_resources_library() failed! Something went wrong while getting resources/ path. | Error: {e}");
            return Err(e);
        }
    }
}

/// Create a name for image file in cache directory
/// Can be used to download image and check if an image already exists
pub fn generate_cache_image_name(name: &str) -> Result<PathBuf, String> {
    let filename: String;

    match generate_image_filename(name) {
        Ok(name) => filename = name,
        Err(e) => {
            error!("monarch_fs::generate_cache_image_name() failed! Failed to build name from regex and {name}. | Error: {e}");
            return Err("Failed to build name from regex!".to_string());
        }
    }

    match get_resources_cache() {
        Ok(mut dir) => {
            dir.push(&filename);
            return Ok(dir);
        }
        Err(e) => {
            error!("monarch_fs::generate_cache_image_name() failed! Something went wrong while trying to get resources/cache/ ! | Error: {e}");
            return Err("Failed to get cached thumbnails folder!".to_string());
        }
    }
}

/// Create a name for image file in cache directory
pub fn generate_library_image_name(name: &str) -> Result<PathBuf, String> {
    let filename: String;

    match generate_image_filename(name) {
        Ok(name) => filename = name,
        Err(e) => {
            error!("monarch_fs::generate_library_image_name() failed! Failed to build name from regex and {name}. | Error: {e}");
            return Err("Failed to build name from regex!".to_string());
        }
    }

    match get_resources_library() {
        Ok(mut dir) => {
            dir.push(&filename);
            return Ok(dir);
        }
        Err(e) => {
            error!("monarch_fs::generate_cache_image_name() failed! Something went wrong while trying to get resources/library/ ! | Error: {e}");
            return Err("Failed to get library thumbnails library!".to_string());
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
            return Ok(filename);
        }
        Err(e) => {
            error!(
                "monarch_fs::generate_image_filename() failed! Failed to build new regex! | Error: {}",
                e
            );
            return Err(e);
        }
    }
}

