use anyhow::{bail, Context, Result};
use reqwest;
use serde::Deserialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tar::Archive;
use tracing::{error, info, warn};

use crate::{
    monarch_games::{games::GameType, monarchgame::MonarchGame},
    monarch_library::library,
    monarch_utils::{
        monarch_fs::{self, get_monarch_bins_path, get_monarch_home},
        monarch_terminal,
    },
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

/// Resolve the on-disk install directory for a game.
fn resolve_install_dir(game: &MonarchGame, exe: &Path) -> PathBuf {
    let recorded = game.properties.install_dir.trim();
    if !recorded.is_empty() && recorded != "Error" {
        let path = PathBuf::from(recorded);
        if path.is_dir() {
            return path;
        }
    }
    exe.parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}

/// Rename the install folder when needed for Wine, then update the game record.
async fn ensure_wine_safe_game_paths(
    game: &mut MonarchGame,
    install_dir: PathBuf,
    exe: PathBuf,
) -> Result<(PathBuf, PathBuf)> {
    let Some(safe_dir) = monarch_fs::ensure_wine_safe_install_dir(&install_dir)? else {
        return Ok((install_dir, exe));
    };

    let rel_exe = exe
        .strip_prefix(&install_dir)
        .unwrap_or(Path::new(exe.file_name().unwrap_or_default()));
    let safe_exe = safe_dir.join(rel_exe);

    game.properties.install_dir = safe_dir.to_string_lossy().to_string();
    game.executable_path = Some(safe_exe.to_string_lossy().to_string());

    if let Err(e) = library::update_game_properties(game).await {
        error!("linux::umu:: Failed to persist renamed install paths | Err: {e}");
    }

    Ok((safe_dir, safe_exe))
}

/// Executes the game using umu-launcher to run in proton.
pub async fn umu_run(game: &MonarchGame) -> Result<()> {
    let mut game = game.clone();

    let compatibility = game
        .compatibility
        .clone()
        .ok_or_else(|| anyhow::anyhow!("linux::umu::umu_run() No compatibility layer set for {}", game.name))?;
    info!("Compatibility layer set: {compatibility}");

    let exe = PathBuf::from(
        game.executable_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("linux::umu::umu_run() No executable path set for {}", game.name))?,
    );
    if !exe.is_file() {
        bail!("Executable not found: {}", exe.display());
    }

    let install_dir = resolve_install_dir(&game, &exe);
    let (install_dir, exe) = ensure_wine_safe_game_paths(&mut game, install_dir, exe).await?;

    let store_arg = match game.get_store_name().as_str() {
        "epic" | "epicgames" => "egs".to_string(),
        _ => "none".to_string(),
    };

    let gameid_arg = format!("umu-{}", game.get_store_id());
    let prefix = monarch_fs::wine_prefix_dir(&gameid_arg);
    std::fs::create_dir_all(&prefix).with_context(|| {
        format!(
            "linux::umu::umu_run() Failed to create wine prefix {} | Err: ",
            prefix.display()
        )
    })?;

    // Mount the game directory inside the Steam Runtime container and give
    let mut env_vars: HashMap<String, String> = HashMap::from([
        ("PROTONPATH".to_string(), compatibility),
        ("GAMEID".to_string(), gameid_arg.clone()),
        ("STORE".to_string(), store_arg),
        ("WINEPREFIX".to_string(), prefix.to_string_lossy().to_string()),
        (
            "STEAM_COMPAT_DATA_PATH".to_string(),
            prefix.to_string_lossy().to_string(),
        ),
        (
            "STEAM_COMPAT_INSTALL_PATH".to_string(),
            install_dir.to_string_lossy().to_string(),
        ),
        (
            "WINEDLLOVERRIDES".to_string(),
            "winemenubuilder.exe=d".to_string(),
        ),
    ]);

    let exe_arg = monarch_fs::relative_launch_exe_arg(&exe, &install_dir);

    let umu: PathBuf = get_umu_exe();
    let launch_command: String = format!("{} {}", umu.display(), exe_arg);

    let launch_args = game.launch_args.as_deref().unwrap_or_default().trim();
    info!("Launch args: {launch_args}");
    let full_command: String = if launch_args.contains("%command%") {
        warn!("Using Steam %command% style launch arguments!");
        launch_args.replace("%command%", &launch_command)
    } else if launch_args.is_empty() {
        launch_command
    } else {
        format!("{launch_command} {launch_args}")
    };

    info!("Install dir: {}", install_dir.display());
    info!("Env vars: {:?}", env_vars);
    info!("Launch command: {}", &full_command);

    // Avoid leaking a host LD_PRELOAD into the Steam Runtime container.
    env_vars
        .entry("LD_PRELOAD".to_string())
        .or_insert_with(String::new);

    let rx = monarch_terminal::spawn_terminal(
        full_command,
        env_vars,
        Some(install_dir.to_string_lossy().to_string()),
    );
    if let Err(e) = rx.await {
        error!("linux::umu::umu_run() Terminal command failed! | Err: {e}");
    }
    Ok(())
}
