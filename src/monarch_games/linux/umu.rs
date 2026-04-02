use anyhow::{bail, Context, Result};
use reqwest;
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};
use tar::Archive;
use tracing::{error, info, warn};

use crate::{
    monarch_games::{games::GameType, monarchgame::MonarchGame},
    monarch_utils::{
        monarch_fs::{self, get_monarch_bins_path, get_monarch_home},
        monarch_terminal,
    },
    monarch_utils::{monarch_fs::get_monarch_home, monarch_terminal},
};

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

/// Returns path to directory where Monarch stores its copy of the umu-launcher binary.
fn get_umu_dir() -> PathBuf {
    get_monarch_bins_path()
        .expect("Don't expect to crash")
        .join("umu")
}

/// Returns path to umu-launcher binary.
pub fn get_umu_exe() -> PathBuf {
    if let Some(p) = monarch_fs::find_linux_binary("umu-run") {
        return p;
    }
    get_umu_dir().join("umu-run")
}

/// For now a simple check to verify that umu-launcher exists.
pub fn umu_is_installed() -> bool {
    if monarch_fs::find_linux_binary("umu-run").is_some() {
        return true;
    }

    let umu_path = get_umu_dir();
    if !umu_path.exists() {
        return false;
    }
    get_umu_exe().exists()
}

pub fn remove_umu() -> Result<()> {
    if !umu_is_installed() {
        warn!("linux::remove_umu() Umu not found!");
        bail!("Umu not found!")
    }

    std::fs::remove_dir_all(&get_umu_dir())
        .with_context(|| "linux::remove_umu() Failed to remove_dir_all() | Err: ")
}

/// Installs the umu-launcher by downloading the binary to $MONARCH_HOME/umu/umu-run
pub fn install_umu() -> Result<()> {
    if umu_is_installed() {
        bail!("linux::umu::install_umu() Failed to install umu-launcher! | Err: Umu path already exists.")
    }

    info!("Getting umu-launcher releases...");
    let umu_release_url: &str =
        "https://api.github.com/repos/Open-Wine-Components/umu-launcher/releases/latest";

    let client = reqwest::blocking::Client::new();
    let release_response = client
        .get(umu_release_url)
        .header("User-Agent", "Monarch/1.0")
        .send()?;

    let release_text: String = release_response.text()
        .with_context(|| "linux::umu::install_umu() Failed to get response text from umu-launcher release page! | Err: ")?;

    let release_data: Release = serde_json::from_str(&release_text).with_context(|| {
        "linux::umu::install_umu() Failed to parse response from umu-launcher release page! | Err: "
    })?;

    info!("Using release: {}", release_data.tag_name);

    let asset = release_data
        .assets
        .into_iter()
        .find(|a| a.name.contains("zipapp") && a.name.ends_with(".tar"))
        .ok_or("No matching asset found")
        .unwrap();

    info!("Downloading : {}", &asset.browser_download_url);

    let mut download_response =
        reqwest::blocking::get(&asset.browser_download_url).with_context(|| {
            format!(
                "linux::umu::install_umu() Failed to get response from {} | Err: ",
                &asset.browser_download_url
            )
        })?;
    let dest_path: PathBuf = get_monarch_home().join(asset.name);
    let mut dest = std::fs::File::create(&dest_path).with_context(|| {
        format!(
            "linux::umu::install_umu() Failed to create: {} | Err: ",
            get_umu_dir().display()
        )
    })?;

    info!("Writing umu-launcher to: {}...", dest_path.display());
    std::io::copy(&mut download_response, &mut dest)
        .with_context(|| "linux::umu::install_umu() Failed to copy response to file! | Err: ")?;

    info!("Unpacking: {}...", dest_path.display());
    let mut archive = Archive::new(std::fs::File::open(&dest_path).with_context(|| {
        format!(
            "linux::umu::install_umu() Failed to open {} | Err: ",
            dest_path.display()
        )
    })?);
    archive.unpack(get_monarch_home()).with_context(|| {
        format!(
            "linux::umu::install_umu() Failed to unpack {}! | Err: ",
            dest_path.display()
        )
    })?;

    info!("Finished downloading umu-launcher.");

    info!("Removing: {}...", dest_path.display());
    std::fs::remove_file(&dest_path).with_context(|| {
        format!(
            "linux::umu::install_umu() Failed to remove: {} | Err: ",
            dest_path.display()
        )
    })?;

    Ok(())
}

/// Executes the game using umu-launcher to run in proton.
pub async fn umu_run(game: &MonarchGame) -> Result<()> {
    info!(
        "Compatibility layer set: {}",
        game.compatibility.as_ref().unwrap()
    );

    let store_arg = match game.get_store_name().as_str() {
        "epic" => "egs".to_string(),
        _ => "none".to_string(),
    };

    let gameid_arg = format!("umu-{}", game.get_store_id());

    let env_vars: HashMap<String, String> = HashMap::from([
        (
            "PROTON_PATH".to_string(),
            game.compatibility.as_ref().unwrap().clone(),
        ),
        ("GAMEID".to_string(), gameid_arg),
        ("STORE".to_string(), store_arg),
    ]);

    let umu: PathBuf = get_umu_exe();
    let launch_command: String = format!(
        "{} '{}'",
        umu.display(),
        game.executable_path.as_ref().unwrap()
    );

    // Order launch args and command in proper order
    info!(
        "Launch args: {}",
        game.launch_args.as_deref().unwrap_or_default()
    );
    let full_command: String = if game
        .launch_args
        .as_deref()
        .unwrap_or_default()
        .contains("%command%")
    {
        warn!("Using Steam %command% style launch arguments!");
        game.launch_args
            .as_deref()
            .unwrap()
            .replace("%command%", &launch_command)
    } else {
        format!(
            "{} {}",
            launch_command,
            game.launch_args.as_deref().unwrap()
        )
    };

    info!("Env vars: {:?}", env_vars);
    info!("Launch command: {}", &full_command);
    let rx = monarch_terminal::spawn_terminal(full_command, env_vars, None);
    if let Err(e) = rx.await {
        error!("linux::umu::umu_run() Terminal command failed! | Err: {e}");
    }
    Ok(())
}
