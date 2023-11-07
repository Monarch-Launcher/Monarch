use super::super::monarchgame::MonarchGame;
use crate::monarch_utils::{
    monarch_fs::{
        create_dir, generate_library_image_name, get_app_data_path, get_home_path, path_exists,
    },
    monarch_vdf,
};
use core::result::Result;
use log::{error, info};
use serde_json;
use serde_json::Value;
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
    let mut path: PathBuf = get_app_data_path().unwrap();
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
pub fn install_steamcmd() {
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
        error!("Failed to run: {install_arg} | Message: {e}");
    }

    if let Err(e) = Command::new("sh").arg("-c").arg(&tar_arg).output() {
        error!("Failed to run: {tar_arg} | Message: {e}");
    }
}

/// Runs specified command via SteamCMD
/// Is currently async to work with Windows version
pub fn steamcmd_command(args: &str) -> Result<(), String> {
    if !steamcmd_is_installed() {
        install_steamcmd();
    }

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
        Ok(path) => monarch_vdf::parse_library_file(path),
        Err(e) => {
            error!(
                "Failed to get default path to Steam library.vdf! | Message: {:?}",
                e
            );
            Vec::new()
        }
    };

    if !found_games.is_empty() {
        games = parse_steam_ids(found_games).await;
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

/// Converts SteamApp ids into MonarchGames
async fn parse_steam_ids(ids: Vec<String>) -> Vec<MonarchGame> {
    let mut games: Vec<MonarchGame> = Vec::new();

    for id in ids {
        let mut game_info: String = String::from("");

        let mut target: String =
            String::from("https://store.steampowered.com/api/appdetails?appids=");
        target.push_str(id.as_str());

        // GET info from Steam servers
        match reqwest::get(&target).await {
            Ok(response) => match response.text().await {
                Ok(body) => {
                    game_info = body;
                }
                Err(e) => {
                    error!("Failed to parse response body! | Message: {e}");
                }
            },
            Err(e) => {
                error!("Failed to get respnse from: {target} | Message: {e}");
            }
        }

        // Parse content into MonarchGame
        if !game_info.is_empty() {
            let game_json: Value = serde_json::from_str(&game_info).unwrap();

            // Check if response from Steam contains "success: true"
            if game_json[&id]["success"] == Value::Bool(true) {
                // Create needed parameters
                let name: String = game_json[&id]["data"]["name"].to_string();
                let id: String = id;
                let platform: String = String::from("steam");
                let exec_path: String = String::new();
                let thumbnail_path: String = String::from(
                    generate_library_image_name(&name)
                        .unwrap()
                        .to_str()
                        .unwrap(),
                );

                let url: &str = game_json[&id]["data"]["header_image"].as_str().unwrap();

                // Create new MonarchGame
                let game: MonarchGame =
                    MonarchGame::new(&name, &platform, &id, &exec_path, &thumbnail_path);

                // Start tokio::task to download thumbail for game
                game.download_thumbnail(url);
                games.push(game);
            }
        }
    }

    games
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
