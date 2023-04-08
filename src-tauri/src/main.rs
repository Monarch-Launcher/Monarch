// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::error;

mod monarch_utils;
mod monarch_games;

use monarch_utils::monarch_fs::init_monarch_fs;
use monarch_utils::logger::init_logger;
use monarch_games::commands::{steam_downloader, search_games, refresh_library, blizzard_downloader, 
                              launch_game, download_game, purchase_game, epic_downloader};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    init_monarch_fs();
    init_logger();

    let app_result = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            steam_downloader,
            search_games,
            refresh_library,
            launch_game,
            download_game,
            purchase_game,
            blizzard_downloader,
            epic_downloader
            ])
        .run(tauri::generate_context!());

    // Better to write to log than to console with .expect() due to line nr 2, hiding console on Windows
    if let Err(e) = app_result {
        error!("Failed to build Tauri app! | Message: {:?}", e);
    }
}