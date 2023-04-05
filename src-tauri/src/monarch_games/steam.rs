use log::{info, error};
use std::process::{Command, Child};
use std::io;
use std::io::Error;
use reqwest::Response;
use scraper::{Html, Selector, ElementRef};

use crate::monarch_utils::{winreg_searcher::{is_installed, get_reg_folder_contents}, downloader::download_and_run, webscraping::request_data};
use crate::unwrap_some_or_return;
use super::monarchgame::MonarchGame;

/// Sctruct for storing information about steam game

/*
---------- Here are public functions for doing anything related to Steam ----------
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
pub async fn find_steam_game(name: &str) -> Vec<MonarchGame> {
    let mut target: String = String::from("https://store.steampowered.com/search/?term=");
    target.push_str(name);

    info!("Searching: {}", target);

    let response: Response = request_data(&target).await;
    let games: Vec<MonarchGame> = find_steam_game_parser(response).await;

    return games
}

/// Opens the steam installer for a steam game, takes its steam id as parameter.
pub async fn download_steam_game(id: &str) -> io::Result<()> {
    let mut game_command: String = String::from("steam://install/");
    game_command.push_str(id);

    let download_result: Result<Child, Error> = Command::new("PowerShell")
                                                        .arg("start")
                                                        .arg(game_command)
                                                        .spawn(); // Run steam installer for specified game 

    match download_result {
        Ok(_) => {
            info!("Running steam installer for game with id: {}", id);
            Ok(())
        },
        Err(e) => {
            error!("Failed to run steam installer for game with id: {} | Message: {:?}", id, e);
            Err(e)
        }
    }
}

/// Launches steam game
pub async fn run_steam_game(id: &str) -> io::Result<()> {
    let mut game_command: String = String::from("steam://rungameid/");
    game_command.push_str(id);

    let launch_result: Result<Child, Error> = Command::new("PowerShell")
                                                        .arg("start")
                                                        .arg(game_command)
                                                        .spawn(); // Run steam installer for specified game 

    match launch_result {
        Ok(_) => {
            info!("Launching game: {}", id);
            Ok(())
        },
        Err(e) => {
            error!("Failed to launch game: {} via steam! | Message: {:?}", id, e);
            Err(e)
        }
    }
}

/// Opens Steam store page for specified game
pub async fn buy_steam_game(id: &str) -> io::Result<()> {
    let mut game_command: String = String::from("steam://purchase/");
    game_command.push_str(id);

    let launch_result: Result<Child, Error> = Command::new("PowerShell")
                                                        .arg("start")
                                                        .arg(game_command)
                                                        .spawn(); // Run steam installer for specified game 

    match launch_result {
        Ok(_) => {
            info!("Launching game: {}", id);
            Ok(())
        },
        Err(e) => {
            error!("Failed to launch game: {} via steam! | Message: {:?}", id, e);
            Err(e)
        }
    }
}

/// Finds local steam library installed on current system via winreg
pub async fn get_steam_library() -> io::Result<Vec<MonarchGame>> {
    let result = get_reg_folder_contents("Valve\\Steam\\Apps");
    
    match result {
        Ok(library) => {
            let mut games: Vec<MonarchGame> = Vec::new();
            
            for item in library.iter() {
                let mut target: String = String::from("https://store.steampowered.com/app/");
                target.push_str(item);
                let response: Response = request_data(&target).await;

                match library_steam_game_parser(response, item).await {
                    Ok(game) => { games.push(game) }
                    Err(_) => { }
                }
            }

            return Ok(games)
        }
        Err(e) => { println!("Failed to get library: {:?}", e) }
    }
    let err: io::Error = io::Error::new(io::ErrorKind::NotFound, "Failed to get Steam game library!"); 
    return Err(err)
}

/*
---------- Here are private "behind the scenes" functions for the public facing functions ----------
*/

/// Returns whether or not Steam launcher is installed
fn steam_is_installed() -> bool {
    return is_installed(r"Valve\Steam");
}

/// Returns a HashMap of games with their respective Steam IDs.
async fn find_steam_game_parser(response: Response) -> Vec<MonarchGame> {
    let mut games: Vec<MonarchGame> = Vec::new();

    let content = response.text().await.unwrap();
    let document = Html::parse_document(&content);

    let title_selector: Selector = Selector::parse("span.title").unwrap();
    let id_selector: Selector = Selector::parse("a.search_result_row.ds_collapse_flag").unwrap();
    
    let game_titles: Vec<ElementRef> = document.select(&title_selector).collect();
    let game_ids: Vec<ElementRef> = document.select(&id_selector).collect();

    for i in 0..game_titles.len() {
        let cur_game = MonarchGame::new(&get_steam_name(game_titles[i]), &get_steamid(game_ids[i]), "steam");
        games.push(cur_game);
    }
    return games
}

async fn library_steam_game_parser(response: Response, id: &str) -> io::Result<MonarchGame> {
    let content = response.text().await.unwrap();
    let document = Html::parse_document(&content);

    let title_selector: Selector = Selector::parse("div.apphub_AppName").unwrap();
    let game_title: Vec<ElementRef> = document.select(&title_selector).collect();

    if game_title.len() > 0 {
        return Ok(MonarchGame::new(&get_steam_name(game_title[0]), id, "steam"))
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
    let app_id: Option<&str> = elem.value().attr("data-ds-appid");

    match app_id {
        Some(id) => id.to_string(),
        None => {
            let bundle_id: Option<&str> = elem.value().attr("data-ds-bundleid");
            unwrap_some_or_return!(bundle_id, "").to_string() // Return the Some() value or just "" as string
        }
    }
}