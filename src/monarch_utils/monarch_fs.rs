use anyhow::{Context, Result};
use regex::Regex;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::{fs, process::exit};
use tracing::{error, info, warn};

use super::monarch_settings::{get_settings_state, Settings};

/*
---------- General functions for filesystem tasks ----------
*/

/// Folder to store image resources for game thumbnails etc...
pub fn verify_monarch_folders() {
    let paths: [PathBuf; 5] = [
        get_monarch_home(),
        get_resources_path(),
        get_resources_cache(),
        get_resources_library(),
        get_settings_path().expect("Panic while getting settings.toml path!"),
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
    let settings: Settings = get_settings_state();
    PathBuf::from(settings.monarch.monarch_home)
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
/// Currently assuming MacOS is fine being treated the same as Linux
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
/// Just like with getting home path, this function assumes MacOS is fine
/// with behaving like Linux
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
    fs::write(path, serde_json::to_string_pretty(&content).unwrap()) // TODO: Remove unwrap for better error handling
        .with_context(|| format!("monarch_fs::write_json_content() Something went wrong trying to write new library to: {file} | Err: ", file = path.display()))?;
    Ok(())
}

/// Abstraction to check whether a given path exists already or not
pub fn path_exists(path: &Path) -> bool {
    Path::new(path).exists()
}

/// Attempts to create an empty directory and returns result
pub fn create_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("monarch_fs::create_dir() Something went wrong trying to create directory: {dir} | Err: ", dir = path.display()))?;
    Ok(())
}

/// Returns all found executables in a given directory
pub fn get_executables(path: &Path) -> Result<Vec<PathBuf>> {
    let mut executables: Vec<PathBuf> = Vec::new();
    let executable_extensions: [&'static str; 6] = ["exe", "app", "sh", "bin", "run", "x86_64"];

    visit_dir(&path, &mut executables, &executable_extensions).unwrap();

    // Recursively visits all directories and subdirectories to find executables
    fn visit_dir(path: &Path, executables: &mut Vec<PathBuf>, executable_extensions: &[&str]) -> Result<()> {
        for entry in fs::read_dir(path).with_context(|| format!("monarch_fs::get_executables() Something went wrong trying to read directory: {dir} | Err: ", dir = path.display()))? {
            let entry = entry.with_context(|| format!("monarch_fs::get_executables() Something went wrong trying to read directory entry: {dir} | Err: ", dir = path.display()))?;
            let inner_path = entry.path();

            if inner_path.is_file() {
                if executable_extensions.contains(&inner_path.extension().unwrap_or("".as_ref()).to_str().unwrap_or("")) {
                    executables.push(inner_path.clone());
                }
            }
            if inner_path.is_dir() {
                visit_dir(&inner_path, executables, executable_extensions)?;
            }
        }
        Ok(())
    }

    Ok(executables)
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

pub fn is_in_cache_dir(path: &Path) -> bool {
    let cache_path: PathBuf = get_resources_cache();
    path.starts_with(cache_path)
}

/// Copies image from cache to resources
/// Returns path to new image in resources directory
pub fn copy_cache_to_library(cache_path: &Path) -> Result<PathBuf> {
    let resources_path: PathBuf = get_resources_library();
    let filename = cache_path.file_name().with_context(|| {
        format!(
            "monarch_fs::copy_cache_to_resources() Failed to get filename of path: {} | Err: ",
            cache_path.display()
        )
    })?;
    let destination_path = resources_path.join(&filename);
    fs::copy(cache_path, &destination_path)
        .with_context(|| format!("monarch_fs::copy_cache_to_resources() Something went wrong trying to copy image from cache to resources: {} | Err: {}", cache_path.display(), destination_path.display()))?;
    Ok(destination_path)
}
