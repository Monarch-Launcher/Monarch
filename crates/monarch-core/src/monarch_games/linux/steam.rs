use super::super::monarchgame::MonarchGame;
use crate::monarch_games::steam_client::parse_steam_ids;
use crate::monarch_utils::monarch_fs::{self, get_monarch_home};
use crate::monarch_utils::monarch_terminal::spawn_terminal;
use crate::monarch_utils::{
    monarch_fs::{create_dir, get_unix_home, path_exists},
    monarch_vdf,
};
use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tar::Archive;
use tracing::{error, info};

/*
* SteamCMD related code.
*
* Monarchs way of handling steam games managed by Monarch itself.
*/

/// Installs SteamCMD for user in .local/share/monarch/steamcmd
pub async fn install_steamcmd() -> Result<()> {
    let tar_dest: PathBuf = get_monarch_home().join("steamcmd.tar.gz");
    let dest_path: PathBuf = get_monarch_home().join("steamcmd");

    if !path_exists(&dest_path) {
        create_dir(&dest_path).with_context(|| "linux::steam::install_steamcmd() -> ")?;
    }

    // Download SteamCMD
    let steamcmd_url: &str =
        "https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz";
    info!("Downloading: {}", steamcmd_url);

    let response = crate::monarch_utils::monarch_http::download_client()
        .get(steamcmd_url)
        .send()
        .await
        .with_context(|| {
            "linux::steam::install_steamcmd() Failed to get response while downloading SteamCMD | Err: "
        })?;
    let content = response.bytes().await.with_context(|| "linux::steam::install_steamcmd() Failed to get response bytes while downloading SteamCMD | Err: ")?;

    info!("Writing: {}", tar_dest.display());
    let mut file = std::fs::File::create(&tar_dest).with_context(|| {
        "linux::steam::install_steamcmd() Failed to create empty steamcmd file. | Err: "
    })?;
    file.write_all(&content).with_context(|| {
        "linux::steam::install_steamcmd() Failed to copy response to file. | Err: "
    })?;

    // "Unzip" SteamCMD
    info!("Unpacking: {}", tar_dest.display());
    let tar_file = std::fs::File::open(&tar_dest).with_context(|| {
        format!(
            "linux::steam::install_steamcmd() Failed to open {} | Err: ",
            tar_dest.display()
        )
    })?;
    let tar = GzDecoder::new(tar_file);
    let mut archive = Archive::new(tar);
    archive.unpack(&dest_path).with_context(|| {
        format!(
            "linux::steam::install_steamcmd() Failed to unpack {} | Err: ",
            dest_path.display()
        )
    })?;

    // Remove tar file
    info!("Removing: {}", tar_dest.display());
    std::fs::remove_file(&tar_dest).with_context(|| {
        format!(
            "linux::steam::install_steamcmd() Failed to remove {} | Err: ",
            tar_dest.display()
        )
    })?;

    Ok(())

    /*
    // --------------- Failed attempt att putting steamcmd in .local/bin and .local/lib ---------------

    // Copy files to correct locations
    let bin_path: PathBuf = get_monarch_bins_path().expect("Don't expect to crash");
    let lib_path: PathBuf = get_unix_home().unwrap().join(".local").join("lib");

    if !(lib_path.join("steamcmd").join("linux32").exists()) {
        info!("Creating required directories.");
        std::fs::create_dir_all(lib_path.join("steamcmd").join("linux32"))?;
    }

    let mut from = dest_path.join("steamcmd.sh");
    let mut to = lib_path.join("steamcmd").join("steamcmd.sh");
    info!("Copying {} -> {}", from.display(), to.display());
    std::fs::copy(from, to)
        .with_context(|| "linux::steam::install_steamcmd() Failed to copy steamcmd.sh | Err: ")?;

    from = dest_path.join("linux32").join("crashhandler.so");
    to = lib_path
        .join("steamcmd")
        .join("linux32")
        .join("crashhandler.so");
    info!("Copying {} -> {}", from.display(), to.display());
    std::fs::copy(from, to).with_context(|| {
        "linux::steam::install_steamcmd() Failed to copy crashhandler.so | Err: "
    })?;

    from = dest_path.join("linux32").join("libstdc++.so.6");
    to = lib_path
        .join("steamcmd")
        .join("linux32")
        .join("libstdc++.so.6");
    info!("Copying {} -> {}", from.display(), to.display());
    std::fs::copy(from, to).with_context(|| {
        "linux::steam::install_steamcmd() Failed to copy libstdc++.so.6 | Err: "
    })?;

    from = dest_path.join("linux32").join("steamcmd");
    to = lib_path.join("steamcmd").join("linux32").join("steamcmd");
    info!("Copying {} -> {}", from.display(), to.display());
    std::fs::copy(from, to).with_context(|| {
        "linux::steam::install_steamcmd() Failed to copy linux32/steamcmd| Err: "
    })?;

    from = dest_path.join("linux32").join("steamerrorreporter");
    to = lib_path
        .join("steamcmd")
        .join("linux32")
        .join("steamerrorreporter");
    info!("Copying {} -> {}", from.display(), to.display());
    std::fs::copy(from, to).with_context(|| {
        "linux::steam::install_steamcmd() Failed to copy steamerrorreporter | Err: "
    })?;

    from = dest_path.join("linux32").join("steamcmd");
    to = bin_path.join("steamcmd");
    info!("Symlinking {} -> {}", from.display(), to.display());

    std::os::unix::fs::symlink(&from, &to).with_context(|| {
        format!(
            "linux::steam::install_steamcmd() Failed to symlink: {} -> {} | Err: ",
            from.display(),
            to.display()
        )
    })?;

    */
}

/// Returns path to the SteamCMD binary used in SteamCMD commands
pub fn get_steamcmd_exe() -> PathBuf {
    if let Some(p) = monarch_fs::find_linux_binary("steamcmd") {
        return p;
    }

    // Fallback to .local/share/monarch/steamcmd/steamcmd.sh
    let fallback_path: PathBuf = get_monarch_home().join("steamcmd").join("steamcmd.sh");
    if fallback_path.exists() {
        return fallback_path;
    }

    PathBuf::new()
}

/// Returns whether or not SteamCMD is installed
pub fn steamcmd_is_installed() -> bool {
    get_steamcmd_exe().exists()
}

/// Runs specified command via SteamCMD
/// Is currently async to work with Windows version
/// TODO: Come back and add a way of showing the output of SteamCMD
pub async fn steamcmd_command(args: Vec<&str>) -> Result<()> {
    let steamcmd_path: String = get_steamcmd_exe().to_string_lossy().to_string();
    let workdir: String = get_unix_home()
        .unwrap()
        .join(".local")
        .join("lib")
        .join("steamcmd")
        .to_string_lossy()
        .to_string();
    let args_string: String = args.iter().map(|arg| format!("{arg} ")).collect::<String>();

    let rx = spawn_terminal(
        format!("{steamcmd_path} {args_string}; sleep 3;"),
        HashMap::new(),
        Some(workdir),
    );

    if let Err(e) = rx.await {
        error!("linux::steam::steamcmd_command() Terminal command failed! | Err: {e}");
    }

    Ok(())
}

/*
 * Steam related code.
 *
 * Used to recognize and interact with preinstalled Steam games on users PC.
 */

/// Returns whether or not Steam launcher is installed
pub fn steam_is_installed() -> bool {
    monarch_fs::find_linux_binary("steam").is_some()
}

/// Finds local steam library installed on current system
pub async fn get_library() -> Vec<MonarchGame> {
    if !steam_is_installed() {
        info!("Steam not installed! Skipping...");
        return Vec::new();
    }

    let mut games: Vec<MonarchGame> = Vec::new();

    let found_games: Vec<String> = match get_default_libraryfolders_location() {
        Ok(path) => match monarch_vdf::parse_library_file(&path) {
            Ok(g) => g,
            Err(e) => {
                error!("linux::steam::get_library() -> {e}");
                Vec::new()
            }
        },
        Err(e) => {
            error!(
                "linux::steam::get_library() Failed to get default path to Steam library.vdf! | Err: {e}",
            );
            Vec::new()
        }
    };

    if !found_games.is_empty() {
        games = parse_steam_ids(&found_games, false, true).await;
    }

    games
}

/// Returns default path used by steam on Linux systems ($HOME/.steam)
pub fn get_default_location() -> Result<PathBuf> {
    let path: PathBuf =
        get_unix_home().with_context(|| "linux::steam::get_default_location() -> ".to_string())?;

    Ok(path.join(".steam/steam/")) // Add path to libraryfolders.vdf
}

/// Returns default path to libraryfolders.vdf used by steam on Linux systems
pub fn get_default_libraryfolders_location() -> Result<PathBuf> {
    let path: PathBuf = get_default_location()
        .with_context(|| "linux::steam::get_default_libraryfolders_location() -> ".to_string())?;

    Ok(path.join("steamapps/libraryfolders.vdf")) // Add path to libraryfolders.vdf
}

/// Runs specified command via Steam
pub fn run_command(args: &str) -> Result<()> {
    Command::new("steam").arg(args).spawn().with_context(|| {
        format!("linux::steam::run_command() Failed to run Steam command {args} | Err")
    })?;

    Ok(())
}
