use log::info;
use ini::Ini;
use reqwest::Response;
use scraper::{Html, Selector, ElementRef};
use regex::Regex;
use std::fs;
use std::ffi::OsString;

use crate::monarch_utils::{monarch_winreg::is_installed, 
                           monarch_download::{download_and_run, download_image}, 
                           monarch_web::request_data,
                           monarch_fs::{generate_cache_image_name, generate_library_image_name, get_app_data_path, path_exists}};
use super::monarchgame::MonarchGame;

/// Installs Epic games launcher if not already installed
pub async fn get_epic() {
    let is_installed: bool = epic_is_installed();

    if is_installed {
        info!("Epic Games already installed!");
    }
    else {
        let target_url: &str = "https://launcher-public-service-prod06.ol.epicgames.com/launcher/api/installer/download/EpicGamesLauncherInstaller.msi";
        download_and_run(target_url).await;
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

/// Finds local steam library installed on current system
pub async fn get_library() -> Vec<MonarchGame> {
    let mut names: Vec<OsString> = Vec::new();
    let mut path: String = get_app_data_path().unwrap();
    path = path.replace("Roaming\\Monarch", "Local\\EpicGamesLauncher\\Saved\\Config\\Windows\\GameUserSettings.ini");


    let i = Ini::load_from_file(path).unwrap();    
    let default_location = i.get_from(Some("Launcher"), "DefaultAppInstallLocation").unwrap();
    
    for game in fs::read_dir(default_location).unwrap() {
        names.push(game.unwrap().file_name())
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
        let name = get_epic_name(titles[i]);
        let image_link = get_img_link(images[i]);
        let image_path = generate_cache_image_name(&name);

        let cur_game = MonarchGame::new(&name, "unknown", "epic", "temp", &image_path);
        games.push(cur_game);

        // Workaround for [tauri::command] not working with download_image().await in same thread 
        tokio::task::spawn(async move {
            download_image(image_link.as_str(), image_path.as_str()).await; 
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
    }

    return games
}

async fn get_game_info(name: &str) -> MonarchGame {
    let mut game_name: String = name.to_string();
    game_name = game_name.replace(" ", "-");

    let re: Regex = Regex::new(r"[^a-zA-Z0-9-]").unwrap();
    game_name = re.replace_all(&game_name, "").to_string();
    game_name = game_name.to_lowercase();

    let mut url: String = String::from("https://www.igdb.com/games/");
    url.push_str(&game_name);
    let response = request_data(&url).await.unwrap();

    println!("{}", url);

    let content = response.text().await.unwrap();
    let document = Html::parse_document(&content);

    println!("{:?}", content);

    let image_selector: Selector = Selector::parse("img.img-responsive.cover_big").unwrap();
    let images: Vec<ElementRef> = document.select(&image_selector).collect();

    println!("{:?}", images);

    let image_link: String = get_img_link(images[0]);
    let image_path = generate_library_image_name(name);

    let game: MonarchGame = MonarchGame::new(name, "epic", "unknown", "exec_path", &image_path);

    if !path_exists(&image_path) { // Only download if image is not in library dir
        // Workaround for [tauri::command] not working with download_image().await in same thread 
        tokio::task::spawn(async move {
            download_image(image_link.as_str(), image_path.as_str()).await; 
        });
    
    }

    return game
}

/// Creates url for thumbnail based on app id
fn get_img_link(elem: ElementRef) -> String {
    return elem.inner_html()
}