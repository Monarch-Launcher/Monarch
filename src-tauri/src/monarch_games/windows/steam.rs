use log::{error, info};
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Context, Result, anyhow};

use crate::monarch_games::monarchgame::MonarchGame;
use crate::monarch_utils::monarch_download::download_file;
use crate::monarch_utils::monarch_fs::{get_home_path, path_exists, create_dir};
use crate::monarch_utils::monarch_vdf;
use crate::monarch_utils::monarch_winreg::is_installed;
use crate::monarch_games::steam_client::parse_steam_ids;

/*
* SteamCMD related code.
*
* Monarchs way of handling steam games managed by Monarch itself.
*/

/// Returns path to Monarchs installed version of SteamCMD
fn get_steamcmd_dir() -> Result<PathBuf> {
    let path: PathBuf = get_home_path().with_context(|| 
        -> String {format!("windows::steam::get_steamcmd_dir() failed! Error returned when getting home path! | Err")})?;
    Ok(path.join("SteamCMD"))
}

/// Returns whether or not SteamCMD is installed
pub fn steamcmd_is_installed() -> Result<bool> {
    let path: PathBuf = get_steamcmd_dir().with_context(|| 
        -> String {format!("windows::steam::steamcmd_is_installed() failed! Error returned when getting SteamCMD directory! | Err")})?;
    Ok(path_exists(&path))
}

/// Installs SteamCMD for user in .monarch
pub async fn install_steamcmd() -> Result<()> {
    let mut dest_path: PathBuf = get_steamcmd_dir().with_context(||
        -> String {format!("windows::steam::install_steamcmd() failed! Error returned when getting SteamCMD directory! | Err")})?;

    if !path_exists(&dest_path) {
        create_dir(&dest_path).context("windows::steam::install_steamcmd() failed! Error creating SteamCMD directory! | Err")?;
    }

    // Download steamcmd
    let download_path: PathBuf = download_file("https://steamcdn-a.akamaihd.net/client/installer/steamcmd.zip").await.with_context(|| 
        -> String {format!("windows::steam::install_steamcmd() failed! Downloading SteamCMD returned error! | Err")})?;
    
    // Change from steamcmd.zip to steamcmd/
    let mut cmd_path: PathBuf = download_path.clone();
    cmd_path.pop();
    cmd_path.push("steamcmd");
    
    // Unzip and copy steamcmd to correct directory
    Command::new("powershell.exe")
        .arg("Expand-Archive -LiteralPath")
        .arg(&download_path).arg("-DestinationPath")
        .arg(&cmd_path)
        .output()
        .context(format!("windows::steam::install_steamcmd() failed! Failed to unzip {} | Err", download_path.display()))?;
    
    cmd_path.push("steamcmd.exe");
    dest_path.push("steamcmd.exe");
    std::fs::copy(&cmd_path, &dest_path).context(format!("windows::steam::install_steamcmd() failed! Error copying {} to {} | Err", cmd_path.display(), dest_path.display()))?;

    Ok(())
}

/// Runs specified command via SteamCMD and waits for it to finish
/// before returning.
pub fn steamcmd_command(args: Vec<&str>) -> Result<()> {
    let mut path: PathBuf = get_steamcmd_dir().with_context(|| 
        -> String {format!("windows::steam::steamcmd_command() failed! Error returned when getting SteamCMD directory! | Err")})?;
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
            // Wait for child process (SteamCMD) to finish and return Result
            child.wait().context("windows::steam::steamcmd_command() failed! Error returned when running SteamCMD child process! | Err")?;
            Ok(())
        }
        Err(e) => {
            // Anonymize login info in logs.
            let args_string: String = args.iter()
                .map(|arg| if arg.contains("login") { format!("+login username password ") } else {format!("{} ", arg) })
                .collect::<String>();

            error!("windows::steam::steamcmd_command() failed! Failed to run {steamcmd}{args_string} | Message: {e}", steamcmd = path.display());
            info!("The error above has replaced your login info for privacy reasons.");
            Err(anyhow!("windows::steam::steamcmd_command() failed!"))
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
pub fn run_command(args: &str) -> Result<()> {
    Command::new("powershell.exe")
        .arg("start")
        .arg(args)
        .spawn()
        .context(format!("steam::run_command() failed! Failed to run Steam command {args} | Err"))?;
        
    Ok(())
}
