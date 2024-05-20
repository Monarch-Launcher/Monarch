use anyhow::{Context, Result};
use log::{error, info, warn};
use regex::Regex;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::{fs, process::exit};

use super::monarch_settings::get_monarch_settings;

/*
---------- General functions for filesystem tasks ----------
*/

/// Folder to store image resources for game thumbnails etc...
pub fn verify_monarch_folders() {
    let paths: [PathBuf; 4] = [
        get_monarch_home(),
        get_resources_path(),
        get_resources_cache(),
        get_resources_library(),
    ];

    for path in paths {
        if !path_exists(&path) {
            warn!("{} not found!", path.display());
            info!("Creating folder: {}", path.display());

            if let Err(e) = create_dir(&path) {
                error!("monarch_fs::verify_monarch_folders() -> {e}",);
                exit(1);
            }
        }
    }
}

/// Returns Unix $HOME.
/// DO NOT USE THIS ON WINDOWS!
pub fn get_unix_home() -> Result<PathBuf> {
    let home_path: String = std::env::var("HOME").with_context(|| {
        "monarch_fs::get_home_path() Could not find envoirment variable 'HOME' | Err: "
    })?;

    Ok(PathBuf::from(home_path))
}

/// Returns the monarch data folder from settings.toml
pub fn get_monarch_home() -> PathBuf {
    let settings = get_monarch_settings().unwrap();

    // Remove " " and ' ' that are still in Strings parsed from toml
    PathBuf::from(
        settings["monarch_home"]
            .to_string()
            .trim_matches('"')
            .trim_matches('\''),
    )
}

/// Gets the users %appdata% or $HOME directory and adds Monarch to the end of it to generate Monarch path
/// returns either $HOME/.monarch or %appdata%/Monarch
#[cfg(windows)]
pub fn generate_monarch_home() -> Result<PathBuf> {
    let appdata_path = std::env::var("APPDATA").with_context(|| {
        "monarch_fs::generate_monarch_home() Could not find envoirment variable 'APPDATA' | Err: "
    })?;

    Ok(PathBuf::from(appdata_path).join("Monarch"))
}

#[cfg(windows)]
pub fn get_settings_path() -> Result<PathBuf> {
    let path = generate_monarch_home().with_context(|| "monarch_fs::get_settings_path() failed! Something went wrong while getting %appdata%/$HOME path. | Err: ")?;

    Ok(PathBuf::from(path).join("settings.toml"))
}

#[cfg(not(windows))]
/// Returns Monarch home according to XDG (.local/share/monarch)
pub fn generate_monarch_home() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("XDG_DATA_HOME") {
        return Ok(PathBuf::from(path).join("monarch")); // Return early with data home according to XDG env var.
    }

    warn!(
        "monarch_fs::generate_monarch_home() No XDG_DATA_HOME set! Falling back to ~/.local/share/"
    );

    let home_path: String = std::env::var("HOME").with_context(|| {
        "monarch_fs::generate_monarch_home() Could not find envoirment variable 'HOME' | Err: "
    })?;

    Ok(PathBuf::from(home_path)
        .join(".local")
        .join("share")
        .join("monarch"))
}

#[cfg(not(windows))]
/// Returns path to settings.json
pub fn get_settings_path() -> Result<PathBuf> {
    if cfg!(not(windows)) {
        if let Ok(path) = std::env::var("XDG_CONFIG_HOME") {
            return Ok(PathBuf::from(path).join("monarch").join("settings.toml"));
            // Return early with data home according to XDG env var.
        }
    }

    warn!("monarch_fs::get_settings_path() No XDG_CONFIG_HOME set! Falling back to ~/.config/");

    let path: String = std::env::var("HOME").with_context(|| {
        "monarch_fs::get_settings_path() Something went wrong while getting $HOME path. | Err: "
    })?;

    Ok(PathBuf::from(path)
        .join(".config")
        .join("monarch")
        .join("settings.toml"))
}

/// Returns path of games installed specifically by Monarch.
pub fn get_monarch_games_path() -> PathBuf {
    let path: PathBuf = get_monarch_home();
    path.join("monarch_games.json")
}

/// Returns path to library.json
pub fn get_library_json_path() -> PathBuf {
    let path: PathBuf = get_monarch_home();
    path.join("library.json")
}

/// Returns path to collections.json
pub fn get_collections_json_path() -> PathBuf {
    let path: PathBuf = get_monarch_home();
    path.join("collections.json")
}

/// Write JSON to file
pub fn write_json_content(content: Value, path: &Path) -> Result<()> {
    fs::write(path, content.to_string()).with_context(|| format!("monarch_fs::write_json_content() Something went wrong trying to write new library to: {file} | Err: ", file = path.display()))?;
    Ok(())
}

/// Abstraction to check whether a given path exists already or not
pub fn path_exists(path: &Path) -> bool {
    Path::new(path).exists()
}

/// Attempts to create an empty directory and returns result
pub fn create_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("monarch_fs::write_json_content() Something went wrong trying to write new library to: {dir} | Err: ", dir = path.display()))?;
    Ok(())
}

/*
---------- Functions related to storing in resources dir ----------
*/

/// Returns path to resources folder.
/// Should never fail during runtime because of init_monarch_fs,
/// but if it does it returns an empty string.
pub fn get_resources_path() -> PathBuf {
    let path: PathBuf = get_monarch_home();
    path.join("resources")
}

/// Returns path to store temporary images
pub fn get_resources_cache() -> PathBuf {
    let path: PathBuf = get_resources_path();
    path.join("cache")
}

/// Returns path to store thumbnails for games in library
pub fn get_resources_library() -> PathBuf {
    let path: PathBuf = get_resources_path();
    path.join("library")
}

/// Create a name for image file in cache directory
/// Can be used to download image and check if an image already exists
pub fn generate_cache_image_path(name: &str) -> PathBuf {
    let filename = generate_image_filename(name);
    let path: PathBuf = get_resources_cache();
    path.join(filename)
}

/// Create a name for image file in cache directory
pub fn generate_library_image_path(name: &str) -> PathBuf {
    let filename = generate_image_filename(name);
    let path: PathBuf = get_resources_library();
    path.join(filename)
}

/// Generates a filename without any special characters or spaces
fn generate_image_filename(name: &str) -> String {
    let mut filename: String = String::from(name);
    filename = filename.replace(' ', "_");

    let regex = Regex::new(r"[^a-zA-Z0-9_]").unwrap(); // Assume that regex will be created.

    filename = regex.replace_all(&filename, "").to_string();
    filename.push_str(".jpg");
    filename
}
