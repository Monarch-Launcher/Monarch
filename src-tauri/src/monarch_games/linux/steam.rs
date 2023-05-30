use log::{error, info};
use reqwest::Response;
use scraper::{ElementRef, Html, Selector};
use std::io::{Error, stdout};
use std::process::{Child, Command, Output};
use tokio;
use core::result::Result;
use std::path::PathBuf;

use super::super::monarchgame::MonarchGame;
use crate::monarch_utils::{
    monarch_download::{download_and_run, download_image},
    monarch_fs::{generate_cache_image_name, generate_library_image_name, path_exists, get_app_data_path},
    monarch_web::request_data,
    monarch_vdf
};


/*
---------- Public functions ----------
*/

/// Downloads Steam launcher if not already installed
pub async fn get_steam() {
    let is_installed: bool = steam_is_installed();

    if is_installed {
        info!("Steam already installed!")
    } else {
        let target_url: &str = "https://cdn.akamai.steamstatic.com/client/installer/SteamSetup.exe";
        if let Err(e) = download_and_run(target_url).await {
            error!("Error occured while attempting to download and run Steam installer! | Message: {:?}", e);
        }
    }
}

/// Search function to find steam games
pub async fn find_game(name: &str) -> Vec<MonarchGame> {
    let mut games: Vec<MonarchGame> = Vec::new();
    let mut target: String = String::from("https://store.steampowered.com/search/?term=");
    target.push_str(name);

    info!("Searching: {}", target);

    if let Ok(response) = request_data(&target).await {
        games = steam_store_parser(response).await;
    }

    return games;
}

/// Opens the steam installer for a steam game
pub fn download_game(name: &str, id: &str) {
    let mut game_command: String = String::from("steam://install/");
    game_command.push_str(id);

    let download_result: Result<Child, Error> = Command::new("PowerShell")
        .arg("start")
        .arg(&game_command)
        .spawn(); // Run steam installer for specified game

    match download_result {
        Ok(_) => {
            info!("Running steam installer for: {}", name);
        }
        Err(e) => {
            error!(
                "Failed to run steam installer: {}(Game: {}) | Message: {:?}", game_command, name, e);
        }
    }
}

/// Launches steam game
pub fn launch_game(name: &str, id: &str) {
    let mut game_command: String = String::from("steam://rungameid/");
    game_command.push_str(id);

    let launch_result: Result<Child, Error> = Command::new("PowerShell")
        .arg("start")
        .arg(&game_command)
        .spawn(); // Run steam installer for specified game
    match launch_result {
        Ok(_) => {
            info!("Launching game: {}", name);
        }
        Err(e) => {
            error!(
                "Failed to launch game: {}({}) | Message: {:?}",
                game_command, name, e
            );
        }
    }
}

/// Opens Steam store page for specified game
pub fn purchase_game(name: &str, id: &str) {
    let mut game_command: String = String::from("steam://purchase/");
    game_command.push_str(id);

    let launch_result: Result<Child, Error> = Command::new("PowerShell")
        .arg("start")
        .arg(&game_command)
        .spawn(); // Run steam installer for specified game
    match launch_result {
        Ok(_) => {
            info!("Opening store page: {}", name);
        }
        Err(e) => {
            error!(
                "Failed to open store page: {}({}) | Message: {:?}",
                game_command, name, e
            );
        }
    }
}

/// Finds local steam library installed on current system
pub async fn get_library() -> Vec<MonarchGame> {
    if !steam_is_installed() {
        info!("Steam not installed! Skipping...");
        return Vec::new();
    }
    
    let found_games: Vec<String>;
    match get_default_location() {
        Ok(path) => { found_games = monarch_vdf::parse_library_file(path); }
        Err(e) => {
            error!("Failed to get default path to Steam library.vdf! | Message: {:?}", e);
            found_games = Vec::new();
        }   
    }
    
    return library_steam_game_parser(found_games).await;
}


/// Returns whether or not Steam launcher is installed
fn steam_is_installed() -> bool {
    let result: Result<Output, Error> = Command::new("find")
                                                .arg("/usr/bin")
                                                .arg("-name")
                                                .arg("steam")
                                                .output();

    match result {
        Ok(output) => {
            if !output.stdout.is_empty() { // Assume that if result is empty Steam is not on System
                return true
            }
        }
        Err(e) => {
            error!("Failed to search for Steam on system using 'find /usr/bin -name steam' | Message: {:?}", e);
            info!("Assuming Steam is not installed on System.");
        }
    }
    return false
}

/// Returns a HashMap of games with their respective Steam IDs.
async fn steam_store_parser(response: Response) -> Vec<MonarchGame> {
    let mut games: Vec<MonarchGame> = Vec::new();

    let content = response.text().await.unwrap();
    let document = Html::parse_document(&content);

    let title_selector: Selector = Selector::parse("span.title").unwrap();
    let id_selector: Selector = Selector::parse("a.search_result_row.ds_collapse_flag").unwrap();

    let titles: Vec<ElementRef> = document.select(&title_selector).collect();
    let ids: Vec<ElementRef> = document.select(&id_selector).collect();

    for i in 0..titles.len() {
        let name: String = get_steam_name(titles[i]);
        let platform_id: String = get_steamid(ids[i]);
        let image_link: String = get_img_link(&platform_id);
        let image_path: PathBuf;
        
        match generate_cache_image_name(&name) {
            Ok(path) => { image_path = path; } 
            Err(e) => {
                error!("Failed to get cache image path! | Message: {:?}", e);
                image_path = PathBuf::from("unknown");
            }
        }

        let cur_game = MonarchGame::new(&name, "steam", &platform_id, "temp", image_path.to_str().unwrap());
        games.push(cur_game);

        if !path_exists(image_path.clone()) { // Only download if image is not in cache dir
            // Workaround for [tauri::command] not working with download_image().await in same thread 
            tokio::task::spawn(async move {
                download_image(image_link.as_str(), image_path).await; 
            });
        }
    }
    return games;
}

async fn library_steam_game_parser(ids: Vec<String>) -> Vec<MonarchGame> {
    let mut games: Vec<MonarchGame> = Vec::new();
    let title_selector: Selector = Selector::parse("div.apphub_AppName").unwrap();

    for id in ids {
        let mut target: String = String::from("https://store.steampowered.com/app/");
        target.push_str(&id);

        if let Ok(content) = request_data(&target).await {
            let document = Html::parse_document(&content.text().await.unwrap());
            let name_refs: Vec<ElementRef> = document.select(&title_selector).collect();

            if name_refs.len() >= 1 {
                let name: String = get_steam_name(name_refs[0]);
                let image_link: String = get_img_link(&id);
                let image_path: PathBuf;
        
                match generate_library_image_name(&name) {
                    Ok(path) => { image_path = path; } 
                    Err(e) => {
                        error!("Failed to get library image path! | Message: {:?}", e);
                        image_path = PathBuf::from("unknown");
                    }
                }

                let game: MonarchGame = MonarchGame::new(&name, "steam", &id, "temp", image_path.to_str().unwrap());
                games.push(game);
                info!("Found Steam game: {}", name);

                if !path_exists(image_path.clone()) { // Only download if image is not in library dir
                    // Workaround for [tauri::command] not working with download_image().await in same thread 
                    tokio::task::spawn(async move {
                        download_image(image_link.as_str(), image_path).await; 
                    });
                
                }
            }
        }
    }
    return games;
}

/// Extracts the name of the game from html element
fn get_steam_name(elem: ElementRef) -> String {
    elem.inner_html()
}

/// Parses html of Steams website to extract either an app id or a bundle id
fn get_steamid(elem: ElementRef) -> String {
    if let Some(app_id) = elem.value().attr("data-ds-appid") {
        return app_id.to_string();
    }
    if let Some(bundle_id) = elem.value().attr("data-ds-bundleid") {
        return bundle_id.to_string();
    }
    String::new() // Default returns empty String for now
}

/// Creates url for thumbnail based on app id
fn get_img_link(id: &str) -> String {
    let mut target = String::from("https://cdn.cloudflare.steamstatic.com/steam/apps/");
    target.push_str(id);
    target.push_str("/header.jpg");

    return target;
}

fn get_default_location() -> Result<PathBuf, String> {
    match get_app_data_path() {
        Ok(mut path) => {
            path.pop();
            path.push(".steam/steam/steamapps/libraryfolders.vdf");

            return Ok(path)
        }
        Err(e) => {
            error!("Failed to get $HOME directory! | Message: {:?}", e);
            return Err("Failed to get $HOME directory!".to_string())
        }
    }
}
