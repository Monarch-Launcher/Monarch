// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![allow(non_snake_case)] // Allow Monarch to use capital M
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::process::exit;

use log::error;

mod monarch_games;
mod monarch_library;
mod monarch_utils;

use monarch_games::commands::{
    download_game, get_library, launch_game, open_store, refresh_library, remove_game, search_games,
};
use monarch_library::commands::{
    create_collection, delete_collection, get_collections, update_collection,
};
use monarch_utils::commands::{
    clear_cached_images, delete_password, get_settings, open_logs, revert_settings, set_password,
    set_settings,
};
use monarch_utils::monarch_fs::verify_monarch_folders;
use monarch_utils::monarch_logger::init_logger;
use monarch_utils::{housekeeping, monarch_settings};

fn init() {
    if let Err(e) = monarch_settings::init() {
        // Crash program if this fails
        println!("Error during settings initialization! | Error: {e}");
        exit(1);
    }
    init_logger(); // Starts logger
    verify_monarch_folders(); // Checks that directories are as Monarch expects
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
            open_store,
            create_collection,
            update_collection,
            delete_collection,
            get_collections,
            open_logs,
            get_settings,
            set_settings,
            revert_settings,
            clear_cached_images,
            set_password,
            delete_password,
            remove_game,
        ])
        .run(tauri::generate_context!());

    // Better to write to log than to console with .expect() due to line nr 2, hiding console on Windows
    if let Err(e) = app_result {
        error!("Failed to build Tauri app! | Message: {:?}", e);
    }
}
