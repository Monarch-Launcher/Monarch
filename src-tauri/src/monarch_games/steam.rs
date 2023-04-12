use log::{info, error};
use std::process::{Command, Child};
use std::io;
use std::io::Error;
use reqwest::Response;
use scraper::{Html, Selector, ElementRef};
use tokio;

use crate::monarch_utils::{monarch_winreg::{is_installed, get_reg_folder_contents}, 
                           monarch_download::{download_and_run, download_image}, 
                           monarch_web::request_data,
                           monarch_fs::{generate_cache_image_name, generate_library_image_name}};
use super::monarchgame::MonarchGame;

/*
---------- Public functions ----------
*/

/// Downloads Steam launcher if not already installed
pub async fn get_steam() {
    let is_installed: bool = steam_is_installed();

    if is_installed {
        info!("Steam already installed!")
        
    }
    else {
        let target_url: &str = "https://cdn.akamai.steamstatic.com/client/installer/SteamSetup.exe";
        download_and_run(target_url).await;
    }
}

/// Search function to find steam games
pub async fn find_game(name: &str) -> Vec<MonarchGame> {
    let mut games: Vec<MonarchGame> = Vec::new();
    let mut target: String = String::from("https://store.steampowered.com/search/?term=");
    target.push_str(name);

    info!("Searching: {}", target);

    if let Ok(response) = request_data(&target).await {    
        games = store_steam_game_parser(response).await;
    }

    return games
}

/// Opens the steam installer for a steam game, takes its steam id as parameter.
pub fn download_game(game: MonarchGame) {
    let name: &str = game.get_name();
    let id: &str = game.get_id();

    let mut game_command: String = String::from("steam://install/");
    game_command.push_str(id);

    let download_result: Result<Child, Error> = Command::new("PowerShell")
                                                        .arg("start")
                                                        .arg(&game_command)
                                                        .spawn(); // Run steam installer for specified game 

    match download_result {
        Ok(_) => {
            info!("Running steam installer for: {}", name);
        },
        Err(e) => {
            error!("Failed to run steam installer: {}(Game: {}) | Message: {:?}", game_command, name, e);
        }
    }
}

/// Launches steam game
pub fn launch_game(game: MonarchGame) {
    let name: &str = game.get_name();
    let id: &str = game.get_id();

    let mut game_command: String = String::from("steam://rungameid/");
    game_command.push_str(id);

    let launch_result: Result<Child, Error> = Command::new("PowerShell")
                                                        .arg("start")
                                                        .arg(&game_command)
                                                        .spawn(); // Run steam installer for specified game 

    match launch_result {
        Ok(_) => {
            info!("Launching game: {}", name);
        },
        Err(e) => {
            error!("Failed to launch game: {}({}) | Message: {:?}", game_command, name, e);
        }
    }
}

/// Opens Steam store page for specified game
pub fn purchase_game(game: MonarchGame) {
    let name: &str = game.get_name();
    let id: &str = game.get_id();

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
            error!("Failed to open store page: {}({}) | Message: {:?}", game_command, name, e);
        }
    }
}

/// Finds local steam library installed on current system via winreg
pub async fn get_library() {
    if let Ok(library) = get_reg_folder_contents("Valve\\Steam\\Apps") {
        let mut games: Vec<MonarchGame> = Vec::new();    
        
        for item in library.iter() { // Get info for each game
            let mut target: String = String::from("https://store.steampowered.com/app/");
            target.push_str(item);
            
            if let Ok(response) = request_data(&target).await {
                if let Ok(game) = library_steam_game_parser(response, item).await {
                    games.push(game);
                }
            }
        }
    }
}

/*
---------- Private functions ----------
*/

/// Returns whether or not Steam launcher is installed
fn steam_is_installed() -> bool {
    return is_installed(r"Valve\Steam");
}

/// Returns a HashMap of games with their respective Steam IDs.
async fn store_steam_game_parser(response: Response) -> Vec<MonarchGame> {
    let mut games: Vec<MonarchGame> = Vec::new();

    let content = response.text().await.unwrap();
    let document = Html::parse_document(&content);

    let title_selector: Selector = Selector::parse("span.title").unwrap();
    let id_selector: Selector = Selector::parse("a.search_result_row.ds_collapse_flag").unwrap();
    
    let titles: Vec<ElementRef> = document.select(&title_selector).collect();
    let ids: Vec<ElementRef> = document.select(&id_selector).collect();

    for i in 0..titles.len() {
        let name = get_steam_name(titles[i]);
        let id = get_steamid(ids[i]);
        let image_link = get_img_link(&id);
        let image_path = generate_cache_image_name(&name);

        let cur_game = MonarchGame::new(&name, &id, "steam", "temp", &image_path);
        games.push(cur_game);

        // Workaround for [tauri::command] not working with download_image().await in same thread 
        tokio::task::spawn(async move {
            download_image(image_link.as_str(), image_path.as_str()).await; 
        });
    }
    return games
}

async fn library_steam_game_parser(response: Response, id: &str) -> io::Result<MonarchGame> {
    let content = response.text().await.unwrap();
    let document = Html::parse_document(&content);

    let title_selector: Selector = Selector::parse("div.apphub_AppName").unwrap();
    let game_title: Vec<ElementRef> = document.select(&title_selector).collect();

    if game_title.len() > 0 {
        let name = get_steam_name(game_title[0]);
        let image_path = generate_library_image_name(&name);
        let image_link = get_img_link(&id);

        let download_path = image_path.clone().as_str();
        // Start downlaod of image in background
        tokio::task::spawn(async move {
            download_image(image_link.as_str(), download_path).await; 
        });

        return Ok(MonarchGame::new(&name, id, "steam", "temp", &image_path))
    }
    
    let err = io::Error::new(io::ErrorKind::NotFound, "No game found matching Registry entry!");
    return Err(err)
}

/// Extracts the name of the game from html element
fn get_steam_name(elem: ElementRef) -> String {
    elem.inner_html()
}

/// Parses html of Steams website to extract either an app id or a bundle id
fn get_steamid(elem: ElementRef) -> String {
    if let Some(app_id) = elem.value().attr("data-ds-appid") {
        return app_id.to_string()
    }
    if let Some(bundle_id) = elem.value().attr("data-ds-bundleid") {
        return bundle_id.to_string()
    }
    String::new() // Default returns empty String for now
}

/// Creates url for thumbnail based on app id
fn get_img_link(id: &str) -> String {
    let mut target = String::from("https://cdn.cloudflare.steamstatic.com/steam/apps/");    
    target.push_str(id);
    target.push_str("/header.jpg");

    return target
}