use log::{error, warn};
use reqwest;
use scraper::{Html, Selector};
use serde_json::Value;
use toml;
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{Context, Result, anyhow};

use super::monarch_client::generate_default_folder;
use super::monarchgame::MonarchGame;
use crate::monarch_utils::monarch_credentials::get_password;
use crate::monarch_utils::monarch_fs::{generate_cache_image_path, generate_library_image_path, path_exists, get_home_path};
use crate::monarch_utils::monarch_settings::get_steam_settings;

#[cfg(target_os = "windows")]
use super::windows::steam;

#[cfg(target_os = "linux")]
use super::linux::steam;

/*
* This file acts like a general interface between commands.rs and Steam.
*
* Basically just some fancy OS specific behaviour gets abstracted away for easier readabilty.
*/

/// Returns if SteamCMD is installed on system or not.
pub fn is_installed() -> Result<bool> {
    steamcmd_is_installed().context("steam_client::is_installed() failed! | Err")
}

#[cfg(windows)]
/// Downloads and installs SteamCMD on users computer.
pub async fn download_and_install() -> Result<()> {
    steam::install_steamcmd().await.context("steam_client::download_and_install() failed! | Err")
}

#[cfg(not(windows))]
/// Downloads and installs SteamCMD on users computer.
pub async fn download_and_install() -> Result<()> {
    steam::install_steamcmd()
}

/// Returns games installed by Steam Client.
pub async fn get_library() -> Vec<MonarchGame> {
    steam::get_library().await
}

/// Returns games found on Steam Store matching the search term with the corresponding store page as String.
/// Store page URL can then be used by frontend to call open_store() from commands.rs.
pub async fn find_game(name: &str) -> HashMap<String, MonarchGame> {
    let mut target: String = String::from("https://store.steampowered.com/search/?term=");
    target.push_str(name);

    let mut games: HashMap<String, MonarchGame> = HashMap::new();

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
    steam::run_command(&command).context("steam_client::launch_game() failed! | Err")
}

/// Attemps to launch SteamCMD game.
pub fn launch_cmd_game(id: &str) -> Result<()> {
    let launch_arg: String = format!("app_launch {id}");
    let args: Vec<&str> = vec![&launch_arg];
    steam::steamcmd_command(args).context("steam_client::launch_cmd_game() failed! | Err")
}

/// Download a Steam game via Monarch and SteamCMD.
pub async fn download_game(name: &str, id: &str) -> Result<MonarchGame> {
    let settings: toml::Value;
    let username: String;

    match get_steam_settings() {
        Some(settings_res) => {
            settings = settings_res;
        }
        None => {
            error!("steam_client::download_game() failed! get_steam_settings() returned None!");
            return Err(anyhow!("steam_client::download_game() failed! Not Steam settings found!"))
        }
    }

    if can_manage_steam(&settings) {
        match get_username(&settings) {
            Some(username_res) => username = username_res,
            None => {
                error!("steam_client::download_game() failed! get_username() returned None!");
                return Err(anyhow!("steam_client::download_game() failed! Not Steam username found!"));
            }
        }
    } else {
        warn!("steam_client::download_game() User tried to install game without allowing Monarch to manage Steam! Cancelling download...");
        return Err(anyhow!("steam_client::download_game() failed! Not allowed to manage games. Check settings."));
    }

    let password: String = get_password("steam", &username).with_context(|| 
        -> String {format!("steam_client::download_game() failed! Could not get password for Steam from secure store! | Err")})?;  
    
    let mut install_dir: PathBuf;
    match get_steam_games_dir(&settings) {
        Some(result) => install_dir = PathBuf::from(result),
        None => install_dir = generate_default_folder().with_context(|| 
            -> String {format!("steam_client::download_game() failed! Error returned when getting default game folder! | Err")})?,
    }
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
    if let Err(e) = steam::steamcmd_command(command) { // Tell SteamCMD to download game
        error!("steam_client::download_game() failed! steamcmd_command() returned error! | Error: {e}");
        return Err(anyhow!("steam_client::download_game() failed! steamcmd_command() returned error"));
    } 

    let monarchgame: MonarchGame = parse_steam_ids(vec![String::from(id)], false).await[0].clone();
    return Ok(monarchgame);
}

/// Uninstall a Steam game via SteamCMD
pub async fn uninstall_game(id: &str) -> Result<()> {
    let settings: toml::Value;

    match get_steam_settings() {
        Some(settings_res) => {
            settings = settings_res;
        }
        None => {
            error!("steam_client::uninstall_game() failed! get_steam_settings() returned None!");
            return Err(anyhow!("steam_client::uninstall_game() failed! Not Steam settings found!"));
        }
    }

    if !can_manage_steam(&settings) {
        warn!("steam_client::uninstall_game() User tried to uninstall game without allowing Monarch to manage Steam! Cancelling uninstall...");
        return Err(anyhow!("steam_client::download_game() failed! Not allowed to manage games. Check settings."));
    }

    let remove_arg: String = format!("+app_uninstall {id}");
    let command: Vec<&str> = vec![&remove_arg, "+quit"];

    steam::steamcmd_command(command).context("steam_client::uninstall_game() failed! | Err")
}

/// Returns path to Monarchs installed version of SteamCMD
pub fn get_steamcmd_dir() -> Result<PathBuf> {
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

/// Returns whether or not Monarch is allowed to manage a users Steam games
fn can_manage_steam(settings: &toml::Value) -> bool {
    match settings.get("manage") {
        Some(value) => return value == &toml::Value::Boolean(true),
        None => return false,
    }
}

/// Returns username from toml::Value
fn get_username(settings: &toml::Value) -> Option<String> {
    if let Some(value) = settings.get("username") {
        if let Some(value_str) = value.as_str() {
            return Some(String::from(value_str))
        }
    }
    None
}

/// Returns the path to install Steam Games in.
fn get_steam_games_dir(settings: &toml::Value) -> Option<String> {
    if let Some(folders) = settings.get("game_folders") {
         if let Some(folder) = folders.get(0) { // Assume first folder is default one for now
            if let Some(str) = folder.as_str() {
                return Some(String::from(str))
            }
        }
    }
    None
}

/// Converts SteamApp ids into MonarchGames.
pub async fn parse_steam_ids(ids: Vec<String>, is_cache: bool) -> Vec<MonarchGame> {
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
                    warn!("steam_client::parse_steam_ids() warning! Failed to parse response body! | Err: {e}");
                }
            },
            Err(e) => {
                error!("steam_client::parse_steam_ids() warning! Failed to get respnse from: {target} | Err: {e}");
            }
        }

        // Parse content into MonarchGame
        if !game_info.is_empty() {
            if let Ok(game_json) = serde_json::from_str::<Value>(&game_info) {
                // Check if response from Steam contains "success: true"
                if game_json[&id]["success"] == Value::Bool(true) {
                    // Create needed parameters
                    let name: String = game_json[&id]["data"]["name"].to_string();
                    let id: String = id;
                    let platform: String = String::from("steam");
                    let exec_path: String = String::new();
                    let thumbnail_path: String;

                    if is_cache {
                        thumbnail_path =
                            String::from(generate_cache_image_path(&name).unwrap().to_str().unwrap());
                    } else {
                        thumbnail_path = String::from(
                            generate_library_image_path(&name)
                                .unwrap()
                                .to_str()
                                .unwrap(),
                        );
                    }

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
    }
    games
}

/// Gets AppIDs and Links from Steam store search
async fn parse_steam_page(body: &str) -> HashMap<String, MonarchGame> {
    let mut ids: Vec<String> = Vec::new();
    let mut links: Vec<String> = Vec::new();
    let mut games: HashMap<String, MonarchGame> = HashMap::new();

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

    let monarch_games = parse_steam_ids(ids, true).await;

    for i in 0..monarch_games.len() {
        games.insert(links[i].clone(), monarch_games[i].clone());
    }

    games
}
