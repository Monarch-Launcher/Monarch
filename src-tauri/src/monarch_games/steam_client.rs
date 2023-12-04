use log::{error, warn};
use reqwest;
use scraper::{Html, Selector};
use serde_json::Value;
use toml;

use super::monarch_client::generate_default_folder;
use super::monarchgame::MonarchGame;
use crate::monarch_utils::monarch_credentials::get_password;
use crate::monarch_utils::monarch_fs::{generate_cache_image_name, generate_library_image_name};
use crate::monarch_utils::monarch_settings::get_steam_settings;
use std::collections::HashMap;
use std::path::PathBuf;

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
pub fn is_installed() -> bool {
    steam::steamcmd_is_installed()
}

#[cfg(windows)]
/// Downloads and installs SteamCMD on users computer.
pub async fn download_and_install() -> Result<(), String> {
    steam::install_steamcmd().await
}

#[cfg(not(windows))]
/// Downloads and installs SteamCMD on users computer.
pub async fn download_and_install() -> Result<(), String> {
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
pub fn launch_game(id: &str) -> Result<(), String> {
    let mut command: String = String::from("steam://rungameid/");
    command.push_str(id);
    steam::run_command(&command)
}

/// Attemps to launch SteamCMD game.
pub async fn launch_cmd_game(id: &str) -> Result<(), String> {
    let launch_arg: String = format!("app_launch {id}");
    let args: Vec<&str> = vec![&launch_arg];
    steam::steamcmd_command(args).await
}

/// Download a Steam game via Monarch and SteamCMD.
pub async fn download_game(name: &str, id: &str) -> Result<MonarchGame, String> {
    let settings: toml::Value;
    let username: String;
    let password: String;

    match get_steam_settings() {
        Some(settings_res) => {
            settings = settings_res;
        }
        None => {
            error!("steam_client::download_game() failed! get_steam_settings() returned None!");
            return Err("No steam settings found!".to_string());
        }
    }

    if can_manage_steam(&settings) {
        match get_username(&settings) {
            Some(username_res) => username = username_res,
            None => {
                error!("steam_client::download_game() failed! get_username() returned None!");
                return Err("No Steam username found!".to_string());
            }
        }
    } else {
        warn!("steam_client::download_game() User tried to install game without allowing Monarch to manage Steam! Cancelling download...");
        return Err("Not allowed to manage games! Check settings!".to_string());
    }

    match get_password("steam", &username) {
        Ok(password_res) => password = password_res,
        Err(e) => {
            error!("steam_client::download_game() failed! Could not get password for Steam from secure store! | Error: {e}");
            return Err("Failed to get Steam password from secure store!".to_string());
        }
    }


    let mut install_dir: PathBuf;
    match get_steam_games_dir(&settings) {
        Some(result) => install_dir = PathBuf::from(result),
        None => install_dir = generate_default_folder(),
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
    match steam::steamcmd_command(command).await { // Tell SteamCMD to download game 
        Ok(_) => {
            let monarchgame: MonarchGame = parse_steam_ids(vec![id.to_string()], false).await[0].clone();
            return Ok(monarchgame);
        }
        Err(e) => {
            error!("steam_client::download_game() failed! steamcmd_command() returned error! | Error: {e}");
            return Err("Failed to install game!".to_string());
        }
    }
}

/// Uninstall a Steam game via SteamCMD
pub async fn uninstall_game(id: &str) -> Result<(), String> {
    let settings: toml::Value;

    match get_steam_settings() {
        Some(settings_res) => {
            settings = settings_res;
        }
        None => {
            error!("steam_client::uninstall_game() failed! get_steam_settings() returned None!");
            return Err("No steam settings found!".to_string());
        }
    }

    if !can_manage_steam(&settings) {
        warn!("steam_client::uninstall_game() User tried to uninstall game without allowing Monarch to manage Steam! Cancelling uninstall...");
        return Err("Not allowed to manage games! Check settings!".to_string());
    }

    let remove_arg: String = format!("+app_uninstall {id}");
    let command: Vec<&str> = vec![&remove_arg, "+quit"];

    steam::steamcmd_command(command).await
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
    match settings.get("username") {
        Some(value) => match value.as_str() {
            Some(value_str) => return Some(String::from(value_str)),
            None => None,
        },
        None => None,
    }
}

/// Returns the path to install Steam Games in.
fn get_steam_games_dir(settings: &toml::Value) -> Option<String> {
    match settings.get("game_folders") {
        Some(folders) => match folders.get(0) { // Assume first folder is default one for now
            Some(folder) => {
                match folder.as_str() {
                    Some(str) => Some(String::from(str)),
                    None => None
                }
            }
            None => None
        }
        None => None
    }    
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
                let thumbnail_path: String;

                if is_cache {
                    thumbnail_path =
                        String::from(generate_cache_image_name(&name).unwrap().to_str().unwrap());
                } else {
                    thumbnail_path = String::from(
                        generate_library_image_name(&name)
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
    games
}

/// Gets AppIDs and Links from Steam store search
async fn parse_steam_page(body: &str) -> HashMap<String, MonarchGame> {
    let game_selector: Selector = Selector::parse("a.search_result_row.ds_collapse_flag").unwrap();

    let mut ids: Vec<String> = Vec::new();
    let mut links: Vec<String> = Vec::new();
    let mut games: HashMap<String, MonarchGame> = HashMap::new();

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
