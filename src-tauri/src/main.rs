// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::error;

mod monarch_utils;
mod monarch_games;
use monarch_utils::monarch_fs::init_monarch_fs;
use monarch_utils::logger::init_logger;
use monarch_games::commands::{steam_downloader, launch_steam_game, search_games,
                              download_steam_game, blizzard_downloader, launch_blizzard_game, 
                              epic_downloader};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    init_monarch_fs();
    init_logger().unwrap();

    let app_result = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            steam_downloader,
            launch_steam_game,
            search_games,
            download_steam_game,
            blizzard_downloader,
            launch_blizzard_game,
            epic_downloader
            ])
        .run(tauri::generate_context!());

    // Better to write to log than to console with .expect() due to line nr 2, hiding console on Windows
    match app_result {
        Ok(_) => { }
        Err(e) => { error!("Failed to build Tauri app! | Message: {:?}", e) }
    }
}