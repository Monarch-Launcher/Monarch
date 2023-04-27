use log::{info, error};
use std::io::Error;
use std::process::{Command, Child};
use std::collections::HashMap;

use crate::monarch_utils::{monarch_winreg::is_installed, monarch_download::download_and_run};
use crate::monarch_utils::monarch_fs::{generate_cache_image_name, generate_library_image_name, path_exists};
use crate::monarch_utils::monarch_download::download_image;
use super::monarchgame::MonarchGame;

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
        download_and_run(target_url).await;
    }
}

/// Attempts to run Blizzard game
pub fn launch_game(name: &str, id: &str) {
    let mut game_command: String = String::from("battlenet://");
    game_command.push_str(id);

    let exec_result: Result<Child, Error> = Command::new("PowerShell")
                                                    .arg("start")
                                                    .arg(&game_command)
                                                    .spawn(); // Run steam installer for specified game
    match exec_result {
        Ok(_) => {
            info!("Launching game: {}", name);
        }
        Err(e) => {
            error!("Failed to launch game: {}({}) | Message: {:?}", game_command, name, e);
        }
    }
}

/// Attempt to find blizzard game matching search term
pub fn find_game(name: &str) -> Vec<MonarchGame> {
    let known_games: [&str; 7] = ["Destiny 2", "Diablo 3", "Hearthstone", "Heroes of the Storm", "Overwatch 2", "Starcraft 2", "World of Warcraft"];
    let mut games: Vec<MonarchGame> = Vec::new();

    for game_name in known_games {
        let lower_known_name: String = game_name.to_lowercase().replace(" ", "");
        let lower_input_name: String = name.to_lowercase().replace(" ", "");
        
        if lower_known_name.contains(lower_input_name.as_str()) {
            let game: MonarchGame = get_blizz_game(game_name, false);
            games.push(game)
        }
    }

    return games
}

/*
----------- Private Blizzard related function -----------
*/

/// Sepcifically checks if Battle.net is installed
fn blizzard_is_installed() -> bool {
    return is_installed(r"Blizzard Entertainment\Battle.net");
}

/// Creates and returns Blizzard owned MonarchGame
fn get_blizz_game(name: &str, is_library: bool) -> MonarchGame {
    let path: String;
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

    if is_library {
        path = generate_library_image_name(name);
    } else {
        path = generate_cache_image_name(name);
    }

    let link = names_and_links.get(name).unwrap().clone();
    let send_path = path.clone();

    println!("{}, {}", link, send_path);

    if !path_exists(&path) { // Only download if image is not in cache dir
        // Workaround for [tauri::command] not working with download_image().await in same thread 
        tokio::task::spawn(async move {
            download_image(link, &send_path).await; 
        });
    }

    MonarchGame::new(name, "blizzard", names_and_ids.get(name).unwrap(), "temp", &path)
}