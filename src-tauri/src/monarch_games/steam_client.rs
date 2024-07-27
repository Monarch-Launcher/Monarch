use anyhow::{bail, Context, Result};
use log::{error, warn};
use reqwest;
use scraper::{Html, Selector};
use serde_json::Value;
use std::path::PathBuf;
use tokio::task;
use toml;

use super::monarch_client::generate_default_folder;
use super::monarchgame::MonarchGame;
use crate::monarch_utils::monarch_credentials::get_password;
use crate::monarch_utils::monarch_fs::{
    generate_cache_image_path, generate_library_image_path, get_monarch_home, path_exists,
};
use crate::monarch_utils::monarch_settings::get_steam_settings;

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

/// Returns games found on Steam Store matching the search term with the corresponding store page as String.
/// Store page URL can then be used by frontend to call open_store() from commands.rs.
pub async fn find_game(name: &str) -> Vec<MonarchGame> {
    let mut target: String = String::from("https://store.steampowered.com/search/?term=");
    target.push_str(name);

    let mut games: Vec<MonarchGame> = Vec::new();

    if let Ok(response) = reqwest::get(&target).await {
        if let Ok(body) = response.text().await {
            games = parse_steam_page(&body).await;
        }
    }
    games
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
    let settings: toml::Value = match get_steam_settings() {
        Some(steam_settings) => steam_settings,
        None => bail!("steam_client::download_game() -> monarch_settings::get_steam_settings() returned None!")
    };

    if !can_manage_steam(&settings) {
        warn!("steam_client::download_game() User tried to install game without allowing Monarch to manage Steam! Cancelling download...");
        bail!("steam_client::download_game() | Err: Not allowed to manage games. Check settings.")
    }

    let username: String = get_username(&settings).with_context(|| {
        "steam_client::download_game() -> steam_client::get_username() returned None!"
    })?;

    let password: String =
        get_password("steam", &username).with_context(|| "steam_client::download_game() -> ")?;

    let mut install_dir: PathBuf = match get_steam_games_dir(&settings) {
        Some(result) => PathBuf::from(result),
        None => generate_default_folder().with_context(|| "steam_client::download_game() -> ")?,
    };
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
    let settings = get_steam_settings().with_context(|| "steam_client::uninstall_game() -> ")?;
    if !can_manage_steam(&settings) {
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

/// Returns whether or not Monarch is allowed to manage a users Steam games
fn can_manage_steam(settings: &toml::Value) -> bool {
    match settings.get("manage") {
        Some(value) => value == &toml::Value::Boolean(true),
        None => false,
    }
}

/// Returns username from toml::Value
fn get_username(settings: &toml::Value) -> Option<String> {
    if let Some(value) = settings.get("username") {
        if let Some(value_str) = value.as_str() {
            return Some(String::from(value_str));
        }
    }
    None
}

/// Returns the path to install Steam Games in.
fn get_steam_games_dir(settings: &toml::Value) -> Option<String> {
    if let Some(folders) = settings.get("game_folders") {
        if let Some(folder) = folders.get(0) {
            // Assume first folder is default one for now
            if let Some(str) = folder.as_str() {
                return Some(String::from(str));
            }
        }
    }
    None
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
    let mut game_info: String = String::from("");
    let mut target: String = String::from("https://store.steampowered.com/api/appdetails?appids=");
    target.push_str(&id);

    // GET info from Steam servers
    match reqwest::get(&target).await {
        Ok(response) => match response.text().await {
            Ok(body) => {
                game_info = body;
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
    if !game_info.is_empty() {
        if let Ok(game_json) = serde_json::from_str::<Value>(&game_info) {
            // Check if response from Steam contains "success: true"
            if game_json[&id]["success"] == Value::Bool(true) {
                // Create needed parameters
                let name: String = game_json[&id]["data"]["name"].to_string();
                let platform: String = String::from("steam");
                let exec_path: String = String::new();

                // TODO: Look into removing unwrap()
                let thumbnail_path = if is_cache {
                    String::from(generate_cache_image_path(&name).to_str().unwrap())
                } else {
                    String::from(generate_library_image_path(&name).to_str().unwrap())
                };

                let url: &str = game_json[&id]["data"]["header_image"].as_str().unwrap();

                // Create new MonarchGame
                let game: MonarchGame =
                    MonarchGame::new(&name, &platform, &id, &exec_path, &thumbnail_path);

                // Download thumbnail to display
                game.download_thumbnail(url).await;
                return Ok(game);
            }
        }
    }
    warn!("Failed to parse Steam game with id: {id}");
    bail!("Failed to parse Steam game with id: {id}")
}

/// Gets AppIDs and Links from Steam store search
async fn parse_steam_page(body: &str) -> Vec<MonarchGame> {
    let mut ids: Vec<String> = Vec::new();
    let mut links: Vec<String> = Vec::new();

    // TODO: Look into a clean way of removing unwrap()
    let game_selector = Selector::parse("a.search_result_row.ds_collapse_flag").unwrap(); // Has to be unwrap rn.

    for css_elem in Html::parse_document(body).select(&game_selector) {
        // Check for AppID
        if let Some(id) = css_elem.value().attr("data-ds-appid") {
            ids.push(id.to_string());

            // Check for link to steam page
            if let Some(link) = css_elem.value().attr("href") {
                links.push(link.to_string());
            } else {
                // Else remove
                ids.pop();
            }
        }
    }

    let mut monarch_games: Vec<MonarchGame> = parse_steam_ids(&ids, true).await;

    for i in 0..monarch_games.len() {
        monarch_games[i].store_page = links[i].clone();
    }

    monarch_games
}
