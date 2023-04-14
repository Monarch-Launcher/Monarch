use log::info;
use reqwest::Response;
use scraper::{Html, Selector, ElementRef};

use crate::monarch_utils::{monarch_winreg::is_installed, 
                           monarch_download::{download_and_run, download_image}, 
                           monarch_web::request_data,
                           monarch_fs::{generate_cache_image_name, generate_library_image_name}};
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

/// Creates url for thumbnail based on app id
fn get_img_link(elem: ElementRef) -> String {
    println!("{:?}", elem.inner_html());

    return String::new()
}