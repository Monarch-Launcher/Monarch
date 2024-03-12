use anyhow::{anyhow, Context, Result};
use log::{error, info, warn};
use regex::Regex;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::{fs, process::exit};

use super::monarch_settings::set_default_settings;

/*
---------- General functions for filesystem tasks ----------
*/

/// Create Monarch folder in users %appdata% directory
pub fn check_appdata_folder() {
    let appdata_path: Result<PathBuf> = get_home_path();

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
#[cfg(windows)]
pub fn get_home_path() -> Result<PathBuf> {
    let appdata_path = std::env::var("APPDATA").with_context(|| 
        -> String {format!("monarch_fs::get_home_path() failed! Could not find envoirment variable 'APPDATA' | Err:")})?;

    Ok(PathBuf::from(appdata_path).join("Monarch"))
}

#[cfg(not(windows))]
pub fn get_home_path() -> Result<PathBuf> {
    let home_path: String = std::env::var("HOME").with_context(|| -> String {
        format!(
            "monarch_fs::get_home_path() failed! Could not find envoirment variable 'HOME' | Err:"
        )
    })?;

    Ok(PathBuf::from(home_path).join(".monarch"))
}

/// Returns path to settings.json
pub fn get_settings_path() -> Result<PathBuf> {
    let path: PathBuf = get_home_path().with_context(|| 
        -> String {format!("monarch_fs::get_settings_path() failed! Something went wrong while getting %appdata%/$HOME path. | Err")})?;

    Ok(path.join("settings.toml"))
}

/// Returns path of games installed specifically by Monarch.
pub fn get_monarch_games_path() -> Result<PathBuf> {
    let path: PathBuf = get_home_path().with_context(|| 
        -> String {format!("monarch_fs::get_library_json_path() failed! Something went wrong while getting %appdata%/$HOME path. | Err")})?;

    Ok(path.join("monarch_games.json"))
}

/// Returns path to library.json
pub fn get_library_json_path() -> Result<PathBuf> {
    let path: PathBuf = get_home_path().with_context(|| 
        -> String {format!("monarch_fs::get_library_json_path() failed! Something went wrong while getting %appdata%/$HOME path. | Err")})?;

    Ok(path.join("library.json"))
}

/// Returns path to collections.json
pub fn get_collections_json_path() -> Result<PathBuf> {
    let path: PathBuf = get_home_path().with_context(|| 
        -> String {format!("monarch_fs::get_collections_json_path() failed! Something went wrong while getting %appdata%/$HOME path. | Err")})?;

    Ok(path.join("collections.json"))
}

/// Write JSON to file
pub fn write_json_content(content: Value, path: &Path) -> Result<()> {
    if let Err(e) = fs::write(path, content.to_string()) {
        return Err(anyhow!(
            "monarch_fs::write_json_content() failed! Something went wrong trying to write new library to: {file} | Err: {e}",
            file = path.display()));
    }
    Ok(())
}

/// Abstraction to check whether a given path exists already or not
pub fn path_exists(path: &Path) -> bool {
    Path::new(path).exists()
}

/// Attempts to create an empty directory and returns result
pub fn create_dir(path: &Path) -> Result<()> {
    if let Err(e) = fs::create_dir_all(path) {
        return Err(anyhow!(
            "monarch_fs::create_dir() failed! Something went wrong while creating new directory: {dir} | Error: {e}",
            dir = path.display()));
    }
    Ok(())
}

/*
---------- Functions related to storing in resources dir ----------
*/

/// Returns path to resources folder.
/// Should never fail during runtime because of init_monarch_fs,
/// but if it does it returns an empty string.
pub fn get_resources_path() -> Result<PathBuf> {
    let path: PathBuf = get_home_path().with_context(|| 
        -> String {format!("monarch_fs::get_resources_path() failed! Something went wrong while getting %appdata%/$HOME path. | Err")})?;

    Ok(path.join("resources"))
}

/// Returns path to store temporary images
pub fn get_resources_cache() -> Result<PathBuf> {
    let path: PathBuf = get_resources_path().with_context(|| 
        -> String {format!("monarch_fs::get_resources_cache() failed! Something went wrong while getting resources/ path. | Err")})?;

    Ok(path.join("cache"))
}

/// Returns path to store thumbnails for games in library
pub fn get_resources_library() -> Result<PathBuf> {
    let path: PathBuf = get_resources_path().with_context(|| 
        -> String {format!("monarch_fs::get_resources_library() failed! Something went wrong while getting resources/ path. | Err")})?;

    Ok(path.join("library"))
}

/// Create a name for image file in cache directory
/// Can be used to download image and check if an image already exists
pub fn generate_cache_image_path(name: &str) -> Result<PathBuf> {
    let filename = generate_image_filename(name).with_context(|| 
        -> String {format!("monarch_fs::generate_cache_image_name() failed! Failed to build name from {name} using regex. | Err")})?;

    let path: PathBuf = get_resources_cache().with_context(|| 
        -> String {format!("monarch_fs::generate_cache_image_name() failed! Something went wrong while trying to get resources/cache/ ! | Err")})?;

    Ok(path.join(&filename))
}

/// Create a name for image file in cache directory
pub fn generate_library_image_path(name: &str) -> Result<PathBuf> {
    let filename = generate_image_filename(name).with_context(|| 
        -> String {format!("monarch_fs::generate_library_image_name() failed! Failed to build name from {name} using regex. | Err")})?;

    let path: PathBuf = get_resources_library().with_context(|| 
        -> String {format!("monarch_fs::generate_cache_image_name() failed! Something went wrong while trying to get resources/library/ ! | Err")})?;

    Ok(path.join(&filename))
}

/// Generates a filename without any special characters or spaces
fn generate_image_filename(name: &str) -> Result<String> {
    let mut filename: String = String::from(name);
    filename = filename.replace(" ", "_");

    let regex = Regex::new(r"[^a-zA-Z0-9_]").with_context(|| -> String {
        format!("monarch_fs::generate_image_filename() failed! Failed to build new regex! | Err")
    })?;

    filename = regex.replace_all(&filename, "").to_string();
    filename.push_str(".jpg");
    Ok(filename)
}
