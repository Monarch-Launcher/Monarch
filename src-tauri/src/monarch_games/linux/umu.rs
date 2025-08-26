use anyhow::{bail, Result};
use reqwest::Response;
use serde::Deserialize;
use std::path::PathBuf;

use crate::monarch_utils::monarch_fs::get_monarch_home;

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

/// Returns path where Monarch stores its copy of the umu-launcher binary.
fn get_umu_path() -> PathBuf {
    let path = get_monarch_home();
    path.join("umu")
}

/// For now a simple check to verify that umu-launcher exists.
fn umu_is_installed() -> bool {
    get_umu_path().exists()
}

/// Installs the umu-launcher by downloading the binary to $MONARCH_HOME/umu/umu-run
pub async fn install_umu() -> Result<()> {
    if umu_is_installed() {
        bail!("linux::umu::install_umu() Failed to install umu-launcher! | Err: Umu path already exists.")
    }

    let umu_release_url: &str =
        "https://api.github.com/repos/Open-Wine-Components/umu-launcher/releases/latest/";

    let release_response: String = reqwest::get(umu_release_url).await?.text().await?;
    let release_data: Release = serde_json::from_str(&release_response)?;

    let asset = release_data
        .assets
        .into_iter()
        .find(|a| a.name.contains("zipapp") && a.name.ends_with(".tar"))
        .ok_or("No matching asset found")
        .unwrap();

    let mut download_response: Response = reqwest::get(asset.browser_download_url).await?;
    let mut dest = std::fs::File::create(&get_umu_path()).unwrap();
    std::io::copy(&mut download_response, &mut dest).unwrap();

    Ok(())
}
