use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::path::PathBuf;
use reqwest;
use tracing::info;
use tar::Archive;

use crate::monarch_utils::monarch_fs::get_monarch_home;

#[derive(Debug, Deserialize, Clone)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize, Clone)]
struct Asset {
    name: String,
    browser_download_url: String,
}

/// Returns path where Monarch stores its copy of the umu-launcher binary.
fn get_umu_dir() -> PathBuf {
    let path = get_monarch_home();
    path.join("umu")
}

pub fn get_umu_exe() -> PathBuf {
    get_umu_dir().join("umu-run")
}

/// For now a simple check to verify that umu-launcher exists.
pub fn umu_is_installed() -> bool {
    let umu_path = get_umu_dir();
    if !umu_path.exists() {
        return false
    }
    umu_path.join("umu-run").exists()
}

/// Installs the umu-launcher by downloading the binary to $MONARCH_HOME/umu/umu-run
pub fn install_umu() -> Result<()> {
    if umu_is_installed() {
        bail!("linux::umu::install_umu() Failed to install umu-launcher! | Err: Umu path already exists.")
    }

    info!("Getting umu-launcher releases...");
    let umu_release_url: &str =
        "https://api.github.com/repos/Open-Wine-Components/umu-launcher/releases";

    let client = reqwest::blocking::Client::new();
    let release_response = client
        .get(umu_release_url)
        .header("User-Agent", "Monarch/1.0")
        .send()?; 

    let release_text: String = release_response.text()
        .with_context(|| "linux::umu::install_umu() Failed to get response text from umu-launcher release page! | Err: ")?;

    let release_data: Vec<Release> = serde_json::from_str(&release_text).with_context(|| "linux::umu::install_umu() Failed to parse response from umu-launcher release page! | Err: ")?;

    info!("Using release: {}", release_data[0].tag_name);

    let asset = release_data[0]
        .clone()
        .assets
        .into_iter()
        .find(|a| a.name.contains("zipapp") && a.name.ends_with(".tar"))
        .ok_or("No matching asset found")
        .unwrap();

    info!("Downloading asset: {}...", asset.name);

    let mut download_response = reqwest::blocking::get(&asset.browser_download_url).with_context(|| format!("linux::umu::install_umu() Failed to get response from {} | Err: ", &asset.browser_download_url))?;
    let dest_path: PathBuf = get_monarch_home().join(asset.name);
    let mut dest = std::fs::File::create(&dest_path).with_context(|| format!("linux::umu::install_umu() Failed to create: {} | Err: ", get_umu_dir().display()))?;

    info!("Writing umu-launcher to: {}...", dest_path.display());
    std::io::copy(&mut download_response, &mut dest).with_context(|| "linux::umu::install_umu() Failed to copy response to file! | Err: ")?;

    info!("Unpacking: {}...", dest_path.display());
    let mut archive = Archive::new(std::fs::File::open(&dest_path).with_context(|| format!("linux::umu::install_umu() Failed to open {} | Err: ", dest_path.display()))?);
    archive.unpack(get_monarch_home()).with_context(|| format!("linux::umu::install_umu() Failed to unpack {}! | Err: ", dest_path.display()))?;

    info!("Finished downloading umu-launcher.");

    info!("Removing: {}...", dest_path.display());
    std::fs::remove_file(&dest_path).with_context(|| format!("linux::umu::install_umu() Failed to remove: {} | Err: ", dest_path.display()))?;

    Ok(())
}