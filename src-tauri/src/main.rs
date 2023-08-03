// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::error;

mod monarch_utils;
mod monarch_games;
mod monarch_library;

use monarch_utils::commands::{open_logs,
                              clear_cached_images};
use monarch_utils::housekeeping;
use monarch_utils::monarch_fs::{check_appdata_folder,
                                check_resources_folder};
use monarch_utils::monarch_logger::init_logger;
use monarch_games::commands::{search_games, 
                              refresh_library,
                              get_library,  
                              launch_game, 
                              download_game, 
                              purchase_game};
use monarch_library::commands::{create_collection,
                                update_collection,
                                delete_collection,
                                get_collections};

fn init() {
    check_appdata_folder(); // Verifies %appdata% (windows) or $HOME (unix) folder exists
    init_logger(); // Starts logger
    check_resources_folder(); // Verify folder structure
    housekeeping::start(); // Starts housekeeping loop
}

fn main() {
    init();

    let app_result = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            search_games, 
            refresh_library,
            get_library,  
            launch_game, 
            download_game, 
            purchase_game,
            create_collection,
            update_collection,
            delete_collection,
            get_collections,
            open_logs,
            clear_cached_images,
            ])
        .run(tauri::generate_context!());

    // Better to write to log than to console with .expect() due to line nr 2, hiding console on Windows
    if let Err(e) = app_result {
        error!("Failed to build Tauri app! | Message: {:?}", e);
    }
}