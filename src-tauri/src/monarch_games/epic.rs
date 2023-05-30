use log::{info, error};
use ini::Ini;
use reqwest::Response;
use scraper::{Html, Selector, ElementRef};
use std::fs;
use std::ffi::OsString;
use std::path::PathBuf;

use crate::monarch_utils::{monarch_winreg::is_installed, 
                           monarch_download::{download_and_run, download_image}, 
                           monarch_web::request_data,
                           monarch_fs::{generate_cache_image_name, get_app_data_path}};
use super::monarchgame::MonarchGame;

/// Installs Epic games launcher if not already installed
pub async fn get_epic() {
    let is_installed: bool = epic_is_installed();

    if is_installed {
        info!("Epic Games already installed!");
    }
    else {
        let target_url: &str = "https://launcher-public-service-prod06.ol.epicgames.com/launcher/api/installer/download/EpicGamesLauncherInstaller.msi";
        if let Err(e) = download_and_run(target_url).await {
            error!("Error occured while attempting to download and run Epic Games installer! | Message: {:?}", e);
        }
    }
}

pub async fn find_game(name: &str) -> Vec<MonarchGame> {
    let mut games: Vec<MonarchGame> = Vec::new();
    let mut target: String = String::from("https://store.epicgames.com/en-US/browse?q=");
    target.push_str(name);
    target.push_str("&sortBy=releaseDate&sortDir=DESC&count=30");

    info!("Searching: {}", target);

    if let Ok(response) = request_data(&target).await {    
        games = epic_store_parser(response).await;
    }

    return games
}

/// Finds local epic games library installed on current system
pub async fn get_library() -> Vec<MonarchGame> {
    let mut games: Vec<MonarchGame> = Vec::new();
    let path: PathBuf = get_app_data_path().unwrap();
    
    if !epic_is_installed() {
        info!("Epic Games not installed! Skipping...");
        return games
    }

    //path = path.replace("Roaming\\Monarch", "Local\\EpicGamesLauncher\\Saved\\Config\\Windows\\GameUserSettings.ini");

    match Ini::load_from_file(path) {
        Ok(i) => {
            match i.get_from(Some("Launcher"), "DefaultAppInstallLocation") {
                Some(default_location) => {
                    games = get_names(default_location).await;
                }
                None => {
                    error!("Failed to find DefaultAppInstallLocation!");
                }
            }
        } 
        Err(e) => {
            error!("Failed to load ini file! | Message: {:?}", e);
        }
    }

    return games
}

/// Returns games names found in default Epic Games location
async fn get_names(default_location: &str) -> Vec<MonarchGame> {
    let mut names: Vec<OsString> = Vec::new();
    match fs::read_dir(default_location) {
        Ok(games) => {
            for game in games {
                names.push(game.unwrap().file_name())
            }
            info!("Found Epic Games games: {:?}", names);
        }
        Err(e) => {
            error!("Failed to read Epic Games GameUserSettings.ini in default location! | Message: {:?}", e);
        }
    }
    
    let games: Vec<MonarchGame> = library_game_parser(names).await;
    return games;
}


/// Returns whether or not Epic games launcehr is installed
fn epic_is_installed() -> bool {
    return is_installed(r"Epic Games\EpicGamesLauncher");
}

async fn epic_store_parser(response: Response) -> Vec<MonarchGame> {
    let mut games: Vec<MonarchGame> = Vec::new();

    let content = response.text().await.unwrap();
    let document = Html::parse_document(&content);

    let title_selector: Selector = Selector::parse("a.css-g3jcms").unwrap();
    let image_selector: Selector = Selector::parse("img.css-174g26k").unwrap();
    
    let titles: Vec<ElementRef> = document.select(&title_selector).collect();
    let images: Vec<ElementRef> = document.select(&image_selector).collect();

    for i in 0..titles.len() {
        let name: String = get_epic_name(titles[i]);
        let image_link: String = get_img_link(images[i]);
        let image_path: PathBuf;

        match generate_cache_image_name(&name) {
            Ok(path) => { image_path = path; }
            Err(e) => {
                error!("Failed to get image cache path! | Message: {:?}", e);
                image_path = PathBuf::from("unknown");
            }
        }

        let cur_game: MonarchGame = MonarchGame::new(&name, "epic", "epic", "temp", image_path.to_str().unwrap());
        games.push(cur_game);
        
        // Workaround for [tauri::command] not working with download_image().await in same thread 
        tokio::task::spawn(async move {
            download_image(image_link.as_str(), image_path).await; 
        });
    }
    return games
}

/// Extracts the name of the game from html element
fn get_epic_name(elem: ElementRef) -> String {
    elem.inner_html()
}

async fn library_game_parser(names: Vec<OsString>) -> Vec<MonarchGame> {
    let mut games: Vec<MonarchGame> = Vec::new();

    for name in names {
        games.push(get_game_info(name.to_str().unwrap()).await);
        info!("Found Epic Games game: {}", name.to_str().unwrap());
    }

    return games
}

async fn get_game_info(name: &str) -> MonarchGame {
    return MonarchGame::new(name, "epic", "unknown", "exec_path", "temp");
}

/// Creates url for thumbnail based on app id
fn get_img_link(elem: ElementRef) -> String {
    return elem.inner_html()
}