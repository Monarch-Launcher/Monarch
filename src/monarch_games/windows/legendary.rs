use anyhow::{bail, Context, Result};
use reqwest;
use serde::Deserialize;
use std::path::PathBuf;
use tracing::info;

use crate::monarch_utils::monarch_fs::get_monarch_home;

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    browser_download_url: String,
}

/// Returns path to directory where Monarch stores its copy of the legendary launcher binary.
pub(crate) fn get_legendary_dir() -> PathBuf {
    let path = get_monarch_home();
    path.join("legendary")
}

/// Returns path to umu-launcher binary.
pub fn get_legendary_exe() -> PathBuf {
    get_legendary_dir().join("legendary.exe")
}

/// For now a simple check to verify that umu-launcher exists.
pub fn legendary_is_installed() -> bool {
    let legendary_path = get_legendary_dir();
    if !legendary_path.exists() {
        return false;
    }
    get_legendary_exe().exists()
}

/// Installs the legendary launcher by downloading the binary to $MONARCH_HOME/legendary/legendary.exe
pub fn install_legendary() -> Result<()> {
    if legendary_is_installed() {
        bail!("windows::legendary::install_legendary() Failed to install legendary! | Err: Legendary path already exists.")
    }

    info!("Getting legendary releases...");
    let legendary_release_url: &str =
        "https://api.github.com/repos/derrod/legendary/releases/latest";

    let client = reqwest::blocking::Client::new();
    let release_response = client
        .get(legendary_release_url)
        .header("User-Agent", "Monarch/1.0")
        .send()
        .with_context(|| {
            "windows::legendary::install_legendary() Failed to get legendary release page! | Err: "
        })?;

    let release_text: String = release_response.text()
        .with_context(|| "windows::legendary::install_legendary() Failed to get response text from legendary release page! | Err: ")?;

    let release_data: Release = serde_json::from_str(&release_text).with_context(|| "windows::umu::install_legendary() Failed to parse response from legendary release page! | Err: ")?;

    if release_data.assets.len() == 0 {
        bail!("windows::legendary::install_legendary() Failed to install legendary! | Err: No assets found in release.");
    }

    info!("Using release: {}", release_data.tag_name);
    info!("Downloading legendary...");

    let download_url: &str = &release_data.assets[1].browser_download_url;
    let mut download_response = reqwest::blocking::get(download_url).with_context(|| {
        format!(
            "windows::legendary::install_legendary() Failed to get response from {} | Err: ",
            download_url
        )
    })?;

    if let Err(e) = std::fs::create_dir_all(get_legendary_dir()) {
        bail!(
            "windows::legendary::install_legendary() Failed to create legendary directory! | Err: {}",
            e
        );
    }

    let dest_path: PathBuf = get_legendary_exe();
    let mut dest = std::fs::File::create(&dest_path).with_context(|| {
        format!(
            "windows::legendary::install_legendary() Failed to create: {} | Err: ",
            get_legendary_dir().display()
        )
    })?;

    info!("Writing legendary to: {}...", dest_path.display());
    std::io::copy(&mut download_response, &mut dest).with_context(|| {
        "windows::legendary::install_legendary() Failed to copy response to file! | Err: "
    })?;

    info!("Finished downloading legendary.");

    Ok(())
}
