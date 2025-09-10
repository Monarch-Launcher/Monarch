use super::super::monarchgame::MonarchGame;
use crate::monarch_games::steam_client::{get_steamcmd_dir, parse_steam_ids};
use crate::monarch_utils::monarch_fs::get_monarch_home;
use crate::monarch_utils::monarch_terminal::run_in_terminal;
use crate::monarch_utils::{
    monarch_fs::{create_dir, get_unix_home, path_exists},
    monarch_vdf,
};
use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use tar::Archive;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tauri::AppHandle;
use tracing::{error, info};

/*
* SteamCMD related code.
*
* Monarchs way of handling steam games managed by Monarch itself.
*/

/// Installs SteamCMD for user in .monarch
pub async fn install_steamcmd() -> Result<()> {
    let tar_dest: PathBuf = get_monarch_home().join("steamcmd.tar.gz");
    let dest_path: PathBuf = get_steamcmd_dir();

    if !path_exists(&dest_path) {
        create_dir(&dest_path).with_context(|| "linux::steam::install_steamcmd() -> ")?;
    }

    // Download SteamCMD
    let steamcmd_url: &str = "https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz";
    info!("Downloading: {}", steamcmd_url);

    let response = reqwest::get(steamcmd_url).await.with_context(|| "linux::steam::install_steamcmd() Failed to get response while downloading SteamCMD | Err: ")?;
    let content = response.bytes().await.with_context(|| "linux::steam::install_steamcmd() Failed to get response bytes while downloading SteamCMD | Err: ")?;

    info!("Writing: {}", tar_dest.display());
    let mut file = std::fs::File::create(&tar_dest).with_context(|| "linux::steam::install_steamcmd() Failed to create empty steamcmd file. | Err: ")?;
    file.write_all(&content).with_context(|| "linux::steam::install_steamcmd() Failed to copy response to file. | Err: ")?;
    
    // "Unzip" SteamCMD
    info!("Unpacking: {}", tar_dest.display());
    let tar_file = std::fs::File::open(&tar_dest).with_context(|| format!("linux::steam::install_steamcmd() Failed to open {} | Err: ", tar_dest.display()))?;
    let tar = GzDecoder::new(tar_file);
    let mut archive = Archive::new(tar);
    archive.unpack(&dest_path).with_context(|| format!("linux::steam::install_steamcmd() Failed to unpack {} | Err: ", dest_path.display()))?;

    // Remove tar file
    info!("Removing: {}", tar_dest.display());
    std::fs::remove_file(&tar_dest).with_context(|| format!("linux::steam::install_steamcmd() Failed to remove {} | Err: ", tar_dest.display()))?;

    Ok(())
}

/// Runs specified command via SteamCMD
/// Is currently async to work with Windows version
/// TODO: Come back and add a way of showing the output of SteamCMD
pub async fn steamcmd_command(handle: &AppHandle, args: Vec<&str>) -> Result<()> {
    let work_dir: PathBuf = get_steamcmd_dir();
    let args_string: String = args.iter().map(|arg| format!("{arg} ")).collect::<String>();

    run_in_terminal(
        handle,
        &format!("./steamcmd.sh {}; sleep 3;", args_string),
        None,
        Some(&work_dir)
    )
    .await
    .with_context(|| "linux::steam::steamcmd_command() -> ")?;

    //info!("linux::steam::steamcmd_command() Result from steamcmd command {}: {}", format!("\"sh -c {} {}\"", path.display(), args_string), cmd_output);
    Ok(())
}

/*
 * Steam related code.
 *
 * Used to recognize and interact with preinstalled Steam games on users PC.
 */

/// Returns whether or not Steam launcher is installed
pub fn steam_is_installed() -> bool {
    if let Ok(result) = Command::new("find")
        .arg("/usr/bin")
        .arg("-name")
        .arg("steam")
        .output()
    {
        // (for now) Assume that non-empty result means Steam is installed on System
        if !result.stdout.is_empty() {
            return true;
        }
    }

    false
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
