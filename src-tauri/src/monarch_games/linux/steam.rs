use super::super::monarchgame::MonarchGame;
use crate::monarch_games::steam_client::{get_steamcmd_dir, parse_steam_ids};
use crate::monarch_utils::{
    monarch_fs::{create_dir, get_home_path, path_exists},
    monarch_vdf,
};
use anyhow::{Context, Result};
use log::{error, info};
use std::path::PathBuf;
use std::process::Command;

/*
* SteamCMD related code.
*
* Monarchs way of handling steam games managed by Monarch itself.
*/

/// Installs SteamCMD for user in .monarch
pub fn install_steamcmd() -> Result<()> {
    let dest_path: PathBuf = get_steamcmd_dir().with_context(||
        -> String {format!("linux::steam::install_steamcmd() failed! Error returned when getting SteamCMD directory! | Err")})?;

    if !path_exists(&dest_path) {
        create_dir(&dest_path).context(
            "linux::steam::install_steamcmd() failed! Error creating SteamCMD directory! | Err",
        )?;
    }

    let mut download_arg: String = String::from("curl -sqL ");
    download_arg.push_str(
        r#""https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz" -o "#,
    );
    download_arg.push_str(dest_path.to_str().unwrap());
    download_arg.push_str("/steamcmd_linux.tar.gz");

    let mut tar_arg: String = String::from("tar zxvf ");
    tar_arg.push_str(dest_path.to_str().unwrap());
    tar_arg.push_str("/steamcmd_linux.tar.gz");
    tar_arg.push_str(" -C ");
    tar_arg.push_str(dest_path.to_str().unwrap());

    info!("Running: {download_arg} && {tar_arg}");

    Command::new("sh")
        .arg("-c")
        .arg(&download_arg)
        .output()
        .context(format!(
            "linux::steam::install_steamcmd() failed! Failed to run: {download_arg} | Err"
        ))?;

    Command::new("sh")
        .arg("-c")
        .arg(&tar_arg)
        .output()
        .context(format!(
            "linux::steam::install_steamcmd() failed! Failed to run: {tar_arg} | Err"
        ))?;

    Ok(())
}

/// Runs specified command via SteamCMD
/// Is currently async to work with Windows version
/// TODO: Come back and add a way of showing the output of SteamCMD
pub fn steamcmd_command(args: Vec<&str>) -> Result<()> {
    let mut path: PathBuf = get_steamcmd_dir().with_context(|| 
        -> String {format!("linux::steam::steamcmd_command() failed! Error returned when getting SteamCMD directory! | Err")})?;
    path.push("steamcmd.sh");

    Command::new("sh")
        .arg(path)
        .arg(format!("{}",
            args.iter()
            .map(|arg| format!("{}", arg))
            .collect::<String>()))
        .output()
        .context("linux::steam::steamcmd_command() failed! Error returned when running SteamCMD child process! | Err")?;

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

    let found_games: Vec<String> = match get_default_location() {
        Ok(path) => monarch_vdf::parse_library_file(&path).unwrap(),
        Err(e) => {
            error!(
                "Failed to get default path to Steam library.vdf! | Err: {:?}",
                e
            );
            Vec::new()
        }
    };

    if !found_games.is_empty() {
        games = parse_steam_ids(found_games, false).await;
    }

    games
}

/// Returns default path used by steam on Linux systems ($HOME/.steam)
fn get_default_location() -> Result<PathBuf> {
    let mut path: PathBuf = get_home_path().with_context(|| -> String {
        format!("linux::steam::get_default_location() failed! Failed to get home directory! | Err")
    })?;

    path.pop(); // Remove .monarch from path
    Ok(path.join(".steam/steam/steamapps/libraryfolders.vdf")) // Add path to libraryfolders.vdf
}

/// Runs specified command via Steam
pub fn run_command(args: &str) -> Result<()> {
    Command::new("steam").arg(args).spawn().context(format!(
        "linux::steam::run_command() failed! Failed to run Steam command {args} | Err"
    ))?;

    Ok(())
}
