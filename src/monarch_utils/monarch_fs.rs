use anyhow::{bail, Context, Result};
use regex::Regex;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::{fs, process::exit};
use tracing::{error, info, warn};

use crate::monarch_games::monarchgame::GameImageType;
use crate::monarch_utils::monarch_settings;
use crate::monarch_utils::monarch_state::MONARCH_STATE;

use super::monarch_settings::Settings;

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
    match MONARCH_STATE.try_read() {
        Ok(state) => {
            match state.get_settings_ptr().read() {
                Ok(settings) => return PathBuf::from(settings.monarch.monarch_home.clone()),
                Err(e) => {
                    error!("monarch_fs::get_monarch_home() Failed to get read lock on Settings | Err: {e}");
                }
            }
        }
        Err(e) => {
            error!("monarch_fs::get_monarch_home() Failed to get read lock on MONARCH_STATE | Err: {e}");
        }
    }

    // Fallback to manually reading settings file
    if let Ok(settings_table) = monarch_settings::read_settings() {
        if let Ok(settings) = settings_table.try_into::<Settings>() {
            println!("got settings: {:?}", settings.monarch.monarch_home);
            return PathBuf::from(settings.monarch.monarch_home.clone());
        }
    }

    // Else default
    generate_monarch_home().expect(
        "monarch_fs::get_monarch_home() Failed to generate monarch home path! Unrecoverable.",
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

/// This function returns where Monarch should place standalone binaries it downloads.
/// For Linux it'll follow XDG_BIN_HOME convention and for Windows it'll be under
/// %appdata%\Monarch\bin\
pub fn get_monarch_bins_path() -> Result<PathBuf> {
    if cfg!(target_os = "linux") {
        match std::env::var("XDG_BIN_HOME") {
            Ok(p) => return Ok(PathBuf::from(p)),
            Err(e) => {
                error!("monarch_fs::get_monarch_bins_path() $XDG_BIN_HOME not set! | Err: {e}");
                warn!("monarch_fs::get_monarch_bins_path() $XDG_BIN_HOME manually selecting ~/.local/bin/");

                let home_path: String = std::env::var("HOME").with_context(|| {
                    "monarch_fs::generate_monarch_bins_path() Could not find envoirment variable 'HOME' | Err: "
                })?;
                return Ok(PathBuf::from(home_path).join(".local").join("bin"));
            }
        }
    } else if cfg!(target_os = "windows") {
        let path =
            generate_monarch_home().with_context(|| "monarch_fs::get_monarch_bins_path() -> ")?;
        return Ok(path.join("bin"));
    }

    bail!("Failed to get location for standalone binaies! Unknown OS!")
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
    fn visit_dir(
        path: &Path,
        executables: &mut Vec<PathBuf>,
        executable_extensions: &[&str],
    ) -> Result<()> {
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

pub fn find_linux_binary(binary_name: &str) -> Option<PathBuf> {
    let path: String = match std::env::var("PATH") {
        Ok(p) => p,
        Err(e) => {
            error!("monarch_fs::linux_binary_installed() Failed to read $PATH! | Err: {e}");
            return None;
        }
    };

    let mut paths: Vec<&str> = path.split(":").collect();

    // Check locally installed binaries as well
    let xdg_bin_local: &str = &get_unix_home()
        .unwrap()
        .join(".local")
        .join("bin")
        .to_string_lossy()
        .to_string();
    if !paths.contains(&xdg_bin_local) {
        paths.push(xdg_bin_local);
    }

    for p in paths {
        match std::fs::read_dir(p) {
            Ok(rd) => {
                for entry in rd {
                    match entry {
                        Ok(de) => {
                            if de.file_name() == binary_name {
                                return Some(de.path());
                            }
                        }
                        Err(e) => {
                            error!("monarch_fs::linux_binary_installed() Failed to entry in: {p} | Err: {e}");
                            return None;
                        }
                    }
                }
            }
            Err(e) => {
                error!("monarch_fs::linux_binary_installed() Failed to read: {p} | Err: {e}");
                return None;
            }
        }
    }

    return None;
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
pub fn generate_cache_image_path(name: &str, t: GameImageType) -> PathBuf {
    let filename = match t {
        GameImageType::Cover => generate_image_filename(&format!("{name}_cover")),
        GameImageType::Artwork => generate_image_filename(&format!("{name}_artwork")),
    };
    let path: PathBuf = get_resources_cache();
    path.join(filename)
}

/// Create a name for image file in cache directory
pub fn generate_library_image_path(name: &str, t: GameImageType) -> PathBuf {
    let filename = match t {
        GameImageType::Cover => generate_image_filename(&format!("{name}_cover")),
        GameImageType::Artwork => generate_image_filename(&format!("{name}_artwork")),
    };
    let path: PathBuf = get_resources_library();
    path.join(filename)
}

/// Generates a filename without any special characters or spaces
fn generate_image_filename(name: &str) -> String {
    let mut filename: String = String::from(name);
    filename = filename.replace(' ', "_");

    let regex = Regex::new(r"[^a-zA-Z0-9_]").unwrap(); // Assume that regex will be created.

    filename = regex.replace_all(&filename, "").to_string();
    filename.push_str(".png");
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
