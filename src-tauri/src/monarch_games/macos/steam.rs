use crate::monarch_games::monarchgame::MonarchGame;
use anyhow::{bail, Result};
use tauri::AppHandle;
use tracing::error;
use std::path::PathBuf;

/*
* This file is currently only here to allow for MacOS compilation.
* Proper MacOS support is planned in the future.
*/

pub async fn install_steamcmd(handle: &AppHandle) -> Result<()> {
    error!("monarch_games::macos::install_steamcmd() MacOS not currently supported!");
    bail!("monarch_games::macos::install_steamcmd() MacOS not currently supported!");
}

pub async fn steamcmd_command(handle: &AppHandle, args: Vec<&str>) -> Result<()> {
    error!("monarch_games::macos::steamcmd_command() MacOS not currently supported!");
    bail!("monarch_games::macos::steamcmd_command() MacOS not currently supported!");
}

pub fn steam_is_installed() -> bool {
    error!("monarch_games::macos::steam_is_installed() MacOS not currently supported!");
    false
}

pub async fn get_library() -> Vec<MonarchGame> {
    error!("monarch_games::macos::get_library() MacOS not currently supported!");
    vec![]
}

pub fn run_command(args: &str) -> Result<()> {
    error!("monarch_games::macos::run_command() MacOS not currently supported!");
    bail!("monarch_games::macos::run_command() MacOS not currently supported!");
}

/// Returns default path used by steam on Linux systems ($HOME/.steam)
pub fn get_default_location() -> Result<PathBuf> {
    error!("monarch_games::macos::get_default_location() MacOS not currently supported!");
    bail!("monarch_games::macos::get_default_location() MacOS not currently supported!");
}

/// Returns default path to libraryfolders.vdf used by steam on Linux systems
pub fn get_default_libraryfolders_location() -> Result<PathBuf> {
    error!("monarch_games::macos::get_libraryfolders_location() MacOS not currently supported!");
    bail!("monarch_games::macos::get_libraryfolders_location() MacOS not currently supported!");
}