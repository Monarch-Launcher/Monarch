use log::{info, error};
use std::io::Error;
use std::process::{Command, Child};
use std::collections::HashMap;
use core::result::Result;
use std::path::PathBuf;

use crate::monarch_utils::monarch_download::download_and_run;
use crate::monarch_utils::monarch_fs::{generate_cache_image_name, generate_library_image_name, path_exists};
use crate::monarch_utils::monarch_download::download_image;
use super::super::monarchgame::MonarchGame;

#[cfg(target_os = "windows")]
use crate::monarch_utils::monarch_winreg::is_installed;

/*
This is hopefully not a long time solution. For now running battlenet://<game> only opens battlenet page and doesn't run game.
Here are game codes:
    Destiny 2: DST2
    Diablo 3: D3
    Hearthstone: WTCG
    Heroes of the Storm: Hero
    Overwatch2: Pro
    Starcraft 2: SC2
    World of Warcraft: WoW
*/

/*
---------- Public Blizzard related functions ----------
*/

/// Installs Battle.net launcher
pub async fn get_blizzard() {
    let is_installed: bool = blizzard_is_installed();

    if is_installed {
        info!("Battle.net already installed!");
    }
    else {
        let target_url: &str = "https://eu.battle.net/download/getInstaller?os=win&installer=Battle.net-Setup.exe";
        if let Err(e) = download_and_run(target_url).await {
            error!("Error occured while attempting to download and run Battle.net installer! | Message: {:?}", e);
        }
    }
}

/// Attempts to run Blizzard game
pub fn launch_game(name: &str, id: &str) {
    
}

/// Attempt to find blizzard game matching search term
pub fn find_game(name: &str) -> Vec<MonarchGame> {
    let known_games: [&str; 7] = ["Destiny 2", "Diablo 3", "Hearthstone", "Heroes of the Storm", "Overwatch 2", "Starcraft 2", "World of Warcraft"];
    let mut games: Vec<MonarchGame> = Vec::new();

    for game_name in known_games {
        let lower_known_name: String = game_name.to_lowercase().replace(" ", "");
        let lower_input_name: String = name.to_lowercase().replace(" ", "");
        
        if lower_known_name.contains(lower_input_name.as_str()) {
            if let Ok(game) = get_blizz_game(game_name, false) {
                games.push(game);
            }
        }
    }

    return games
}

/// Finds local steam library installed on current system
pub async fn get_library() -> Vec<MonarchGame> {
    let games: Vec<MonarchGame> = Vec::new();

    return games
}

/*
----------- Private Blizzard related function -----------
*/

/// Sepcifically checks if Battle.net is installed
fn blizzard_is_installed() -> bool {
    return false;
}

/// Creates and returns Blizzard owned MonarchGame
fn get_blizz_game(name: &str, is_library: bool) -> Result<MonarchGame, String> {
    let names_and_ids: HashMap<&str, &str> = HashMap::from([
        ("Destiny 2", "DST2"),
        ("Diablo 3", "D3"),
        ("Hearthstone", "WTCG"),
        ("Heroes of he Storm", "Hero"),
        ("Overwatch 2", "Pro"),
        ("Starcraft 2", "SC2"),
        ("World of Warcraft", "WoW")
    ]);

    let names_and_links: HashMap<&str, &str> = HashMap::from([
        ("Destiny 2", "https://images.contentstack.io/v3/assets/blte410e3b15535c144/blt8599cdc8468fb924/630fd93c2d08277c7e733f1e/hero_bg_desktop.jpg"),
        ("Diablo 3", "https://wallpaperaccess.com/full/7471248.jpg"),
        ("Hearthstone", "https://wallpaperaccess.com/full/7471195.jpg"),
        ("Heroes of the Storm", "https://wallpaperaccess.com/full/7471312.jpg"),
        ("Overwatch 2", "https://wallpaperaccess.com/full/7471219.jpg"),
        ("Starcraft 2", "https://wallpaperaccess.com/full/7471222.jpg"),
        ("World of Warcraft", "https://wallpaperaccess.com/full/1692125.jpg")
    ]);

    let path: PathBuf;

    if is_library {
        match generate_library_image_name(name) {
            Ok(image_path) => {
                path = image_path;
            }
            Err(e) => {
                path = PathBuf::from("unknown");
                error!("Failed to get library image name! | Message: {:?}", e);
            }
        }
    } else {
        match generate_cache_image_name(name) {
            Ok(image_path) => {
                path = image_path;
            }
            Err(e) => {
                path = PathBuf::from("unknown");
                error!("Failed to get cache image name! | Message: {:?}", e);
            }
        }
    }

    let link: &str;
    match names_and_links.get(name) {
        Some(map_link) => { link = map_link; }
        None => {
            error!("Failed to get game from HashMap! (get_blizz_game())");
            return Err("Failed to get game from known Blizzard games!".to_string())
        }
    }

    let path_clone: PathBuf = path.clone();
    if !path_exists(path.clone()) { // Only download if image is not in cache dir
        // Workaround for [tauri::command] not working with download_image().await in same thread 
        tokio::task::spawn(async move {
            download_image(link, path_clone).await; 
        });
    }

    return Ok(MonarchGame::new(name, "blizzard", names_and_ids.get(name).unwrap(), "temp", path.to_str().unwrap()))
}