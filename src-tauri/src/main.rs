// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::error;

mod monarch_utils;
mod monarch_games;

use monarch_utils::commands::get_logs;
use monarch_utils::monarch_fs::init_monarch_fs;
use monarch_utils::monarch_logger::init_logger;
use monarch_games::commands::{search_games, 
                              refresh_library,
                              get_library,  
                              launch_game, 
                              download_game, 
                              purchase_game, 
                              steam_downloader,
                              blizzard_downloader,
                              epic_downloader};

fn main() {
    init_monarch_fs();
    init_logger();

    let app_result = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            search_games, 
            refresh_library,
            get_library,  
            launch_game, 
            download_game, 
            purchase_game, 
            steam_downloader,
            blizzard_downloader,
            epic_downloader,
            get_logs
            ])
        .run(tauri::generate_context!());

    // Better to write to log than to console with .expect() due to line nr 2, hiding console on Windows
    if let Err(e) = app_result {
        error!("Failed to build Tauri app! | Message: {:?}", e);
    }
}