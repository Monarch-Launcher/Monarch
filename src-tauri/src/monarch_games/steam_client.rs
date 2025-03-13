use anyhow::{bail, Context, Result};
use log::{error, warn};
use reqwest;
use std::path::PathBuf;
use tokio::task;

use super::monarchgame::{MonarchGame, MonarchWebGame};
use crate::monarch_utils::monarch_credentials::get_password;
use crate::monarch_utils::monarch_fs::{
    generate_cache_image_path, generate_library_image_path, get_monarch_home, path_exists,
};
use crate::monarch_utils::monarch_settings::get_settings_state;

#[cfg(target_os = "windows")]
use super::windows::steam;

#[cfg(target_os = "macos")]
use super::macos::steam;

#[cfg(target_os = "linux")]
use super::linux::steam;

/*
* This file acts like a general interface between commands.rs and Steam.
*
* Basically just some fancy OS specific behaviour gets abstracted away for easier readabilty.
*/

/// Returns if SteamCMD is installed on system or not.
pub fn is_installed() -> bool {
    let path: PathBuf = get_steamcmd_dir();
    path_exists(&path)
}

#[cfg(windows)]
/// Downloads and installs SteamCMD on users computer.
pub async fn download_and_install() -> Result<()> {
    steam::install_steamcmd()
        .await
        .context("steam_client::download_and_install() failed! | Err")
}

#[cfg(not(windows))]
/// Downloads and installs SteamCMD on users computer.
pub async fn download_and_install() -> Result<()> {
    steam::install_steamcmd().with_context(|| "steam_client::download_and_install() -> ")?;
    steam::steamcmd_command(vec!["+set_steam_guard_code"])
        .with_context(|| "steam_client::download_and_install() -> ")
}

/// Returns games installed by Steam Client.
pub async fn get_library() -> Vec<MonarchGame> {
    steam::get_library().await
}

/// Attempts to launch Steam Client game.
pub fn launch_game(id: &str) -> Result<()> {
    let mut command: String = String::from("steam://rungameid/");
    command.push_str(id);
    steam::run_command(&command).with_context(|| "steam_client::launch_game() -> ")
}

/// Attemps to launch SteamCMD game.
pub fn launch_cmd_game(id: &str) -> Result<()> {
    let args: Vec<&str> = vec!["+app_launch", id];
    steam::steamcmd_command(args).with_context(|| "steam_client::launch_cmd_game() -> ")
}

/// Download a Steam game via Monarch and SteamCMD.
pub async fn download_game(name: &str, id: &str) -> Result<MonarchGame> {
    let settings = get_settings_state();
    let steam_settings = settings.steam;

    if !steam_settings.manage {
        warn!("steam_client::download_game() User tried to install game without allowing Monarch to manage Steam! Cancelling download...");
        bail!("steam_client::download_game() | Err: Not allowed to manage games. Check settings.")
    }

    let username: String = steam_settings.username;
    let password: String =
        get_password("steam", &username).with_context(|| "steam_client::download_game() -> ")?;

    let mut install_dir: PathBuf = PathBuf::from(settings.monarch.game_folder);
    install_dir.push(name);

    // Directory argument
    let mut install_dir_arg: String = String::from("+force_install_dir ");
    install_dir_arg.push_str(&install_dir.to_string_lossy());

    // Login argument
    let mut login_arg = String::from("+login ");
    login_arg.push_str(&username);
    login_arg.push(' ');
    login_arg.push_str(&password);

    // App ID argument
    let mut download_arg = String::from("+app_update ");
    download_arg.push_str(id);
    download_arg.push_str(" validate");

    // Build the command as a string with arguments in order
    let command: Vec<&str> = vec![&install_dir_arg, &login_arg, &download_arg, "+quit"];

    // TODO: Wait for Steamcmd to return
    // TODO: steam::steamcmd_command() should wait for SteamCMD to finish
    steam::steamcmd_command(command).with_context(|| "steam_client::download_game() -> ")?;

    let mut monarchgame: MonarchGame = parse_steam_ids(&[String::from(id)], false).await[0].clone();
    monarchgame.platform = "steamcmd".to_string();
    Ok(monarchgame)
}

/// Uninstall a Steam game via SteamCMD
pub async fn uninstall_game(id: &str) -> Result<()> {
    let steam_settings = get_settings_state().steam;
    if !steam_settings.manage {
        warn!("steam_client::uninstall_game() User tried to uninstall game without allowing Monarch to manage Steam! Cancelling uninstall...");
        bail!("steam_client::download_game() | Err: Not allowed to manage games. Check settings.")
    }

    let remove_arg: String = format!("+app_uninstall {id}");
    let command: Vec<&str> = vec![&remove_arg, "+quit"];

    steam::steamcmd_command(command).with_context(|| "steam_client::uninstall_game() -> ")
}

/// Returns path to Monarchs installed version of SteamCMD
pub fn get_steamcmd_dir() -> PathBuf {
    let path: PathBuf = get_monarch_home();
    path.join("SteamCMD")
}

/// Converts SteamApp ids into MonarchGames.
pub async fn parse_steam_ids(ids: &[String], is_cache: bool) -> Vec<MonarchGame> {
    let mut tasks = Vec::new();
    let mut games: Vec<MonarchGame> = Vec::new();

    for id in ids {
        let new_task = task::spawn(parse_id(id.clone(), is_cache));
        tasks.push(new_task);
    }

    for task in tasks {
        if let Ok(finished_task) = task.await {
            if let Ok(game) = finished_task {
                games.push(game);
            }
        }
    }

    return games;
}

/// Helper function to parse individual steam ids. Allows for concurrent parsing.
async fn parse_id(id: String, is_cache: bool) -> Result<MonarchGame> {
    let mut game_info_opt: Option<MonarchWebGame> = None;
    let target: String =
        format!("https://monarch-launcher.com/api/games?platform=steam&platform_id={id}");

    // GET info from Steam servers
    match reqwest::get(&target).await {
        Ok(response) => match response.text().await {
            Ok(body) => {
                let web_games: Vec<MonarchWebGame> = serde_json::from_str(&body).unwrap();
                if web_games.is_empty() {
                    bail!("Nothing returned for game with ID: {id}");
                }
                game_info_opt = Some(web_games.first().unwrap().clone());
            }
            Err(e) => {
                warn!("steam_client::parse_steam_ids() Failed to parse response body! | Err: {e}");
            }
        },
        Err(e) => {
            error!(
                "steam_client::parse_steam_ids() Failed to get response from: {target} | Err: {e}"
            );
        }
    }

    // Parse content into MonarchGame
    if let Some(game_info) = game_info_opt {
        let thumbnail_path = if is_cache {
            String::from(generate_cache_image_path(&game_info.name).to_str().unwrap())
        } else {
            String::from(
                generate_library_image_path(&game_info.name)
                    .to_str()
                    .unwrap(),
            )
        };
        let monarch_game = MonarchGame::new(
            &game_info.name,
            game_info.id,
            &game_info.platform,
            &game_info.platform_id,
            &game_info.store_page,
            "N/A",
            &thumbnail_path,
        );
        monarch_game.download_thumbnail(game_info.cover_url).await;
        return Ok(monarch_game);
    }

    warn!("Failed to parse Steam game with id: {id}");
    bail!("Failed to parse Steam game with id: {id}")
}
