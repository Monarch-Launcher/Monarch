use super::super::monarchgame::MonarchGame;
use crate::monarch_games::steam_client::parse_steam_ids;
use crate::monarch_utils::{
    monarch_fs::{create_dir, get_appdata_path, get_home_path, path_exists},
    monarch_vdf,
};
use core::result::Result;
use log::{error, info};
use std::io::Error;
use std::path::PathBuf;
use std::process::{Command, Output};
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
    let mut path: PathBuf = get_steamcmd_dir();
    path.push("steamcmd.sh");
    path_exists(&path)
}

/// Installs SteamCMD for user in .monarch
pub fn install_steamcmd() -> Result<(), String> {
    let path: PathBuf = get_steamcmd_dir();

    if !path_exists(&path) {
        create_dir(&path).unwrap();
    }

    let mut install_arg: String = String::from("curl -sqL ");
    install_arg.push_str(
        r#""https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz" -o "#,
    );
    install_arg.push_str(path.to_str().unwrap());
    install_arg.push_str("/steamcmd_linux.tar.gz");

    let mut tar_arg: String = String::from("tar zxvf ");
    tar_arg.push_str(path.to_str().unwrap());
    tar_arg.push_str("/steamcmd_linux.tar.gz");
    tar_arg.push_str(" -C ");
    tar_arg.push_str(path.to_str().unwrap());

    println!("Running {install_arg} && {tar_arg}");

    if let Err(e) = Command::new("sh").arg("-c").arg(&install_arg).output() {
        error!(
            "linux::steam::install_steamcmd() failed! Failed to run: {install_arg} | Error: {e}"
        );
    }

    if let Err(e) = Command::new("sh").arg("-c").arg(&tar_arg).output() {
        error!("linux::steam::install_steamcmd() failed! Failed to run: {tar_arg} | Error: {e}");
        return Err("Failed to install SteamCMD!".to_string());
    }

    Ok(())
}

/// Runs specified command via SteamCMD
/// Is currently async to work with Windows version
pub fn steamcmd_command(args: &str) -> Result<(), String> {
    let mut path: PathBuf = get_steamcmd_dir();
    path.push("steamcmd.sh");

    match Command::new("sh").arg(path).arg(args).spawn() {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failed to run steamcmd {args} | Message: {e}");
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
    let result: Result<Output, Error> = Command::new("find")
        .arg("/usr/bin")
        .arg("-name")
        .arg("steam")
        .output();

    match result {
        Ok(output) => {
            if !output.stdout.is_empty() {
                // Assume that if result is empty Steam is not installed on System
                return true;
            }
        }
        Err(e) => {
            error!("Failed to search for Steam on system using 'find /usr/bin -name steam' | Message: {:?}", e);
            info!("Assuming Steam is not installed on System.");
        }
    }
    false
}

/// Tells user to install Steam manually on Linux
pub async fn get_steam() -> Result<(), String> {
    info!("Can't automatically install Steam on Linux! You need to install it yourself via your package manager.");
    Err("Can't automatically install Steam on Linux!".to_string())
}

/// Finds local steam library installed on current system
pub async fn get_library() -> Vec<MonarchGame> {
    if !steam_is_installed() {
        info!("Steam not installed! Skipping...");
        return Vec::new();
    }

    let mut games: Vec<MonarchGame> = Vec::new();

    let found_games: Vec<String> = match get_default_location() {
        Ok(path) => monarch_vdf::parse_library_file(&path),
        Err(e) => {
            error!(
                "Failed to get default path to Steam library.vdf! | Message: {:?}",
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
fn get_default_location() -> Result<PathBuf, String> {
    match get_home_path() {
        Ok(mut path) => {
            path.push(".steam/steam/steamapps/libraryfolders.vdf");

            Ok(path)
        }
        Err(e) => {
            error!("Failed to get $HOME directory! | Message: {:?}", e);
            Err("Failed to get $HOME directory!".to_string())
        }
    }
}

/// Runs specified command via Steam
pub fn run_command(args: &str) -> Result<(), String> {
    match Command::new("steam").arg(args).spawn() {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failed to run steam {args} | Message: {e}");
            Err("Failed to run steam command!".to_string())
        }
    }
}
