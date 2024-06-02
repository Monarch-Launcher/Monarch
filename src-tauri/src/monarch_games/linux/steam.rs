use super::super::monarchgame::MonarchGame;
use crate::monarch_games::steam_client::{get_steamcmd_dir, parse_steam_ids};
use crate::monarch_utils::monarch_windows::run_in_terminal;
use crate::monarch_utils::{
    monarch_fs::{create_dir, get_unix_home, path_exists},
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
    let dest_path: PathBuf = get_steamcmd_dir();

    if !path_exists(&dest_path) {
        create_dir(&dest_path).with_context(|| "linux::steam::install_steamcmd() -> ")?;
    }

    let download_arg: &str = r#"curl -sqL "https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz" | tar zxvf -"#;

    let installation_script = format!(r#"mkdir {};
cd {};
{};
sleep 2;"#, dest_path.display(), dest_path.display(), &download_arg); // Sleep for 2 seconds to allow user to see what is happening.

    info!("Running: SteamCMD installation script: \n----------\n{installation_script}\n----------");

    run_in_terminal(&installation_script).with_context(|| format!("linux::steam::install_steamcmd() -> "))?;
    Ok(())
}

/// Runs specified command via SteamCMD
/// Is currently async to work with Windows version
/// TODO: Come back and add a way of showing the output of SteamCMD
pub fn steamcmd_command(args: Vec<&str>) -> Result<()> {
    let mut path: PathBuf = get_steamcmd_dir();
    path.push("steamcmd.sh");
    let args_string: String = args.iter().map(|arg| format!(" {arg}")).collect::<String>();

    info!("Running command: {} {}; sleep 5;", path.display(), args_string);
    run_in_terminal(&format!("{} {}; sleep 5;", path.display(), args_string)).with_context(|| "linux::steam::steamcmd_command() -> ")?;

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

    let found_games: Vec<String> = match get_default_location() {
        Ok(path) => monarch_vdf::parse_library_file(&path).unwrap(),
        Err(e) => {
            error!(
                "linux::steam::get_library() Failed to get default path to Steam library.vdf! | Err: {e}",
            );
            Vec::new()
        }
    };

    if !found_games.is_empty() {
        games = parse_steam_ids(&found_games, false).await;
    }

    games
}

/// Returns default path used by steam on Linux systems ($HOME/.steam)
fn get_default_location() -> Result<PathBuf> {
    let path: PathBuf =
        get_unix_home().with_context(|| "linux::steam::get_default_location() -> ".to_string())?;

    Ok(path.join(".steam/steam/steamapps/libraryfolders.vdf")) // Add path to libraryfolders.vdf
}

/// Runs specified command via Steam
pub fn run_command(args: &str) -> Result<()> {
    Command::new("steam").arg(args).spawn().with_context(|| {
        format!("linux::steam::run_command() Failed to run Steam command {args} | Err")
    })?;

    Ok(())
}
