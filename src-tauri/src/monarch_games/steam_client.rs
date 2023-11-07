use log::error;
use serde_json::Value;
use reqwest;
use scraper::{Html, Selector};

use crate::monarch_utils::monarch_fs::generate_library_image_name;
use super::monarch_client::generate_default_folder;
use super::monarchgame::MonarchGame;
use std::collections::HashMap;

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

/// Downloads and install SteamCMD on users computer.
pub async fn download_and_install() -> Result<(), String> {
    steam::install_steamcmd().await
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
pub fn launch_cmd_game(id: &str) -> Result<(), String> {
    let mut args: String = String::from("app_launch ");
    args.push_str(id);
    steam::steamcmd_command(&args)
}

/// Download a Steam game via Monarch and SteamCMD.
pub fn download_game(id: &str) -> Result<(), String> {
    // Directory argument
    let mut install_dir: String = String::from(" +force_install_dir ");
    install_dir.push_str(generate_default_folder().to_str().unwrap());

    let username: String = todo!();
    let password: String = todo!();
    let two_factor: String = todo!();

    // Login argument
    let mut login = String::from(" +login ");
    login.push_str(&username.trim());
    login.push(' ');
    login.push_str(&password.trim());
    login.push(' ');
    login.push_str(&two_factor.trim());

    // App ID argument
    let mut download = String::from(" +app_update ");
    download.push_str(id);
    download.push_str(" +validate");

    // Build the command as a string with arguments in order
    let mut command: String = String::from(&install_dir);
    command.push_str(&login);
    command.push_str(&download);

    steam::steamcmd_command(&command) // Tell SteamCMD to download game
}

/// Converts SteamApp ids into MonarchGames
pub async fn parse_steam_ids(ids: Vec<String>) -> Vec<MonarchGame> {
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

/// Gets AppIDs and Links from Steam store search
async fn parse_steam_page(body: &str) -> HashMap<String, MonarchGame> {
    let a_selector: Selector = Selector::parse("a").unwrap();

    let mut ids: Vec<String> = Vec::new();
    let mut links: Vec<String> = Vec::new();
    let mut games: HashMap<String, MonarchGame> = HashMap::new();

    for a_tag in Html::parse_document(body).select(&a_selector) {
        // Check for AppID
        if let Some(id) = a_tag.value().attr("data-ds-appid") {
            ids.push(id.to_string());

            // Check for link to steam page
            if let Some(link) = a_tag.value().attr("href") {
                links.push(link.to_string());
            } else { // Else remove
                ids.pop();
            }
        }
    }

    let monarch_games = parse_steam_ids(ids).await;

    for i in 0..monarch_games.len() {
        games.insert(links[i].clone(), monarch_games[i].clone());
    }
    games
}