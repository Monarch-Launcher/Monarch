use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::path::PathBuf;
use reqwest;
use tracing::info;

use crate::monarch_utils::monarch_fs::get_monarch_home;

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
}

/// Returns path to directory where Monarch stores its copy of the umu-launcher binary.
fn get_legendary_dir() -> PathBuf {
    let path = get_monarch_home();
    path.join("legendary")
}

/// Returns path to umu-launcher binary.
pub fn get_legendary_exe() -> PathBuf {
    get_legendary_dir().join("legendary")
}

/// For now a simple check to verify that umu-launcher exists.
pub fn legendary_is_installed() -> bool {
    let umu_path = get_legendary_dir();
    if !umu_path.exists() {
        return false
    }
    get_legendary_exe().exists()
}

/// Installs the umu-launcher by downloading the binary to $MONARCH_HOME/umu/umu-run
pub fn install_legendary() -> Result<()> {
    if legendary_is_installed() {
        bail!("linux::legendary::install_legendary() Failed to install legendary! | Err: Legendary path already exists.")
    }

    info!("Getting legendary releases...");
    let legendary_release_url: &str =
        "https://api.github.com/repos/derrod/legendary/releases/latest";

    let client = reqwest::blocking::Client::new();
    let release_response = client
        .get(legendary_release_url)
        .header("User-Agent", "Monarch/1.0")
        .send()?; 

    let release_text: String = release_response.text()
        .with_context(|| "linux::umu::install_legendary() Failed to get response text from legendary release page! | Err: ")?;

    let release_data: Release = serde_json::from_str(&release_text).with_context(|| "linux::umu::install_legendary() Failed to parse response from legendary release page! | Err: ")?;

    info!("Using release: {}", release_data.tag_name);
    info!("Downloading legendary...");

    let download_url: String = format!("https://github.com/derrod/legendary/releases/download/{}/legendary", release_data.tag_name);
    let mut download_response = reqwest::blocking::get(&download_url).with_context(|| format!("linux::legendary::install_legendary() Failed to get response from {} | Err: ", &download_url))?;
    let dest_path: PathBuf = get_monarch_home().join("legendary");
    let mut dest = std::fs::File::create(&dest_path).with_context(|| format!("linux::legendary::install_legendary() Failed to create: {} | Err: ", get_legendary_dir().display()))?;

    info!("Writing legendary to: {}...", dest_path.display());
    std::io::copy(&mut download_response, &mut dest).with_context(|| "linux::legendary::install_legendary() Failed to copy response to file! | Err: ")?;

    info!("Finished downloading legendary.");

    info!("Removing: {}...", dest_path.display());
    std::fs::remove_file(&dest_path).with_context(|| format!("linux::legendary::install_legendary() Failed to remove: {} | Err: ", dest_path.display()))?;

    Ok(())
}