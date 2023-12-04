use core::result::Result;
use log::{error, info};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::monarch_games::monarchgame::MonarchGame;
use crate::monarch_utils::monarch_download::download_file;
use crate::monarch_utils::monarch_fs::{get_appdata_path, path_exists, create_dir};
use crate::monarch_utils::monarch_vdf;
use crate::monarch_utils::monarch_winreg::is_installed;
use crate::monarch_games::steam_client::parse_steam_ids;

/*
* SteamCMD related code.
*
* Monarchs way of handling steam games managed by Monarch itself.
*/

/// Returns path to Monarchs installed version of SteamCMD
fn get_steamcmd_dir() -> PathBuf {
    let mut path: PathBuf = get_appdata_path().unwrap();
    path.push("SteamCMD");
    path
}

/// Returns whether or not SteamCMD is installed
pub fn steamcmd_is_installed() -> bool {
    let path: PathBuf = get_steamcmd_dir();
    path_exists(&path)
}

/// Installs SteamCMD for user in .monarch
pub async fn install_steamcmd() -> Result<(), String> {
    let mut path: PathBuf = get_steamcmd_dir();

    if !path_exists(&path) {
        create_dir(&path).unwrap();
    }

    // Download steamcmd
    let download_path: PathBuf = download_file("https://steamcdn-a.akamaihd.net/client/installer/steamcmd.zip").await.unwrap();
    let mut cmd_path: PathBuf = download_path.clone();
    cmd_path.pop();
    cmd_path.push("steamcmd");
    
    // Unzip and copy steamcmd to correct directory
    if let Err(e) = Command::new("PowerShell").arg("Expand-Archive -LiteralPath").arg(&download_path).arg("-DestinationPath").arg(&cmd_path).output() {
        error!("Failed to unzip {} | Message: {e}", download_path.display());
        return Err("Failed to unzip steamcmd.zip!".to_string())
    }
    
    cmd_path.push("steamcmd.exe");
    path.push("steamcmd.exe");
    if let Err(e) = std::fs::copy(&cmd_path, &path) {
        error!("Failed to copy {} to Monarchs home directory! | Message: {e}", cmd_path.display());
        return Err("Failed to copy steamcmd to Monarchs home directory!".to_string())
    }
    Ok(())
}

/// Runs specified command via SteamCMD and waits for it to finish
/// before returning.
pub async fn steamcmd_command(args: Vec<&str>) -> Result<(), String> {
    let mut path: PathBuf = get_steamcmd_dir();
    path.push("steamcmd.exe");

    match Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(format!("Start-Process {:?} -ArgumentList {} -WindowStyle Normal -Wait", 
            &path, 
            args.iter()
                .map(|arg| format!("'{}'", arg))
                .collect::<Vec<_>>()
                .join(",")
            )).spawn() {
        Ok(mut child) => {
            if let Err(e) = child.wait() {
                error!("windows::steam::steamcmd_command() got an error from SteamCMD child process! | Error: {e}");
                return Err(String::from("Something went wrong while launching SteamCMD!"))
            }

            Ok(())
        }
        Err(e) => {
            // Anonymize login info in logs.
            let args_string: String = args.iter()
                .map(|arg| if arg.contains("login") { format!("+login username password ") } else {format!("{} ", arg) })
                .collect::<String>();

            error!("windows::steam::steamcmd_command() failed! Failed to run {steamcmd}{args_string} | Message: {e}", steamcmd = path.display());
            info!("The error above has replaced your login info for privacy reasons.");
            Err("Failed to run SteamCMD command!".to_string())
        }
    }
}

/*
 * Steam related code.
 *
 * Used to recognize and interact with preinstalled Steam games on users PC.
*/

/// Returns whether or not Steam launcher is installed
pub fn steam_is_installed() -> bool {
    return is_installed(r"Valve\Steam");
}

/// Finds local steam library installed on current system
pub async fn get_library() -> Vec<MonarchGame> {
    if !steam_is_installed() {
        info!("Steam not installed! Skipping...");
        return Vec::new()
    }

    let path = Path::new("C:\\Program Files (x86)\\Steam\\steamapps\\libraryfolders.vdf");
    match monarch_vdf::parse_library_file(&path) {
        Ok(found_games) => { return parse_steam_ids(found_games, false).await }
        Err(e) => { 
            error!("{:#}", e);
            vec![]
        }
    }
}

/// Runs specified command via Steam
pub fn run_command(args: &str) -> Result<(), String> {
    match Command::new("PowerShell").arg("start").arg(args).spawn() {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("steam::run_command() failed! Failed to run Steam command {args} | Message: {e}");
            Err("Failed to run steam command!".to_string())
        }
    }
}
