use log::error;
use crate::monarch_games::monarchgame::MonarchGame;
use anyhow::{bail, Result};

/*
* This file is currently only here to allow for MacOS compilation.
* Proper MacOS support is planned in the future.
*/

pub fn install_steamcmd() -> Result<()> {
    error!("monarch_games::macos::install_steamcmd() MacOS not currently supported!");
    bail!("monarch_games::macos::install_steamcmd() MacOS not currently supported!");
}

pub fn steamcmd_command(args: Vec<&str>) -> Result<()> {
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
