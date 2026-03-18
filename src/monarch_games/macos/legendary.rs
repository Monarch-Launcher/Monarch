use anyhow::{bail, Context, Result};
use reqwest;
use serde::Deserialize;
use std::path::PathBuf;
use tracing::info;

use crate::monarch_utils::monarch_fs::get_monarch_home;

/// Returns path to directory where Monarch stores its copy of the legendary launcher binary.
pub(crate) fn get_legendary_dir() -> PathBuf {
    todo!()
}

/// Returns path to umu-launcher binary.
pub fn get_legendary_exe() -> PathBuf {
    todo!()
}

/// For now a simple check to verify that umu-launcher exists.
pub fn legendary_is_installed() -> bool {
    todo!()
}

/// Installs the legendary launcher by downloading the binary to $MONARCH_HOME/legendary/legendary.exe
pub fn install_legendary() -> Result<()> {
    todo!()
}
