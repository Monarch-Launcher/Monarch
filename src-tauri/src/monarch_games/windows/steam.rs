use anyhow::{bail, Context, Result};
use log::{error, info};
use reqwest::Response;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::monarch_games::monarchgame::MonarchGame;
use crate::monarch_games::steam_client::{get_steamcmd_dir, parse_steam_ids};
use crate::monarch_utils::monarch_fs::{create_dir, path_exists};
use crate::monarch_utils::monarch_vdf;
use crate::monarch_utils::monarch_winreg::is_installed;

/*
* SteamCMD related code.
*
* Monarchs way of handling steam games managed by Monarch itself.
*/

/// Installs SteamCMD for user in .monarch
pub async fn install_steamcmd() -> Result<()> {
    let steamcmd_path: PathBuf = get_steamcmd_dir();

    // Verify that steamcmd path has to be created
    if !path_exists(&steamcmd_path) {
        create_dir(&steamcmd_path).with_context(|| "windows::steam::install_steamcmd() -> ")?;
    }

    // Generate filenames
    let steamcmd_zip: PathBuf = steamcmd_path.join("steamcmd.zip");
    let steamcmd_exe: PathBuf = steamcmd_path.join("steamcmd.exe");

    // Download steamcmd
    let download_url: &str = "https://steamcdn-a.akamaihd.net/client/installer/steamcmd.zip";
    let response: Response = reqwest::get(download_url).await.with_context(|| format!("windows::steam::install_steamcmd() error occured running reqwest::get({}) | Err: ", download_url))?;
    let mut file: File = File::create(&steamcmd_zip)?;
    let content: String = response.text().await.with_context(|| "windows::steam::install_steamcmd() error while reading response.text()! | Err")?;
    file.write_all(content.as_bytes()).with_context(|| format!("windows::steam::install_steamcmd() error writing content to file: {} | Err", steamcmd_zip.display()))?;

    // Unzip and copy steamcmd to correct directory
    Command::new("powershell.exe")
        .arg("Expand-Archive -LiteralPath")
        .arg(&steamcmd_zip)
        .arg("-DestinationPath")
        .arg(&steamcmd_exe)
        .output()
        .with_context(|| format!(
            "windows::steam::install_steamcmd() failed! Failed to unzip {} | Err",
            steamcmd_zip.display()
        ))?;

    Ok(())
}

/// Runs specified command via SteamCMD and waits for it to finish
/// before returning.
pub fn steamcmd_command(args: Vec<&str>) -> Result<()> {
    let mut path: PathBuf = get_steamcmd_dir(); 
    path.push("steamcmd.exe");

    match Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(format!(
            "Start-Process {:?} -ArgumentList {} -WindowStyle Normal -Wait",
            &path,
            args.iter()
                .map(|arg| format!("'{}'", arg))
                .collect::<Vec<_>>()
                .join(",")
        ))
        .spawn()
    {
        Ok(mut child) => {
            // Wait for child process (SteamCMD) to finish and return Result
            child.wait().with_context(|| "windows::steam::steamcmd_command() failed! Error returned when running SteamCMD child process! | Err")?;
            Ok(())
        }
        Err(e) => {
            // Anonymize login info in logs.
            let args_string: String = args
                .iter()
                .map(|arg| {
                    if arg.contains("login") {
                        format!("+login username password ")
                    } else {
                        format!("{} ", arg)
                    }
                })
                .collect::<String>();

            info!("The error bellow has replaced your login info for privacy reasons.");

            let mut cmd_str: String = path.display().to_string();
            cmd_str.push_str(&args_string);
            bail!("windows::steam::steamcmd_command() Failed to run {cmd_str} | Err {e}", )
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
        return Vec::new();
    }

    let path = Path::new("C:\\Program Files (x86)\\Steam\\steamapps\\libraryfolders.vdf");
    match monarch_vdf::parse_library_file(&path) {
        Ok(found_games) => return parse_steam_ids(&found_games, false).await,
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
        .with_context(|| format!("windows::steam::run_command() failed! Failed to run Steam command {args} | Err"))?;

    Ok(())
}
