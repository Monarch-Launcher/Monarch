// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::exit;

use log::error;

mod monarch_games;
mod monarch_library;
mod monarch_utils;

use monarch_games::commands::{
    download_game, get_library, launch_game, open_store, refresh_library, search_games, remove_game,
};
use monarch_library::commands::{
    create_collection, delete_collection, get_collections, update_collection,
};
use monarch_utils::commands::{clear_cached_images, get_settings, open_logs, set_settings, revert_settings, set_password, delete_password, build_quicklaunch, show_quicklaunch, hide_quicklaunch};
use monarch_utils::monarch_fs::{check_appdata_folder, check_resources_folder};
use monarch_utils::monarch_logger::init_logger;
use monarch_utils::{housekeeping, monarch_settings};
use monarch_utils::monarch_windows::kill_quicklaunch;
use tauri::{RunEvent, AppHandle};

/// Runs a pre-check to ensure system is as expected.
fn init() {
    check_appdata_folder(); // Verifies %appdata% (windows) or $HOME (unix) folder exists
    init_logger(); // Starts logger
    check_resources_folder(); // Verify folder structure
    
    if let Err(e) = monarch_settings::init() { // Crash program if this fails
        error!("Error during settings initialization! | Error: {e}");
        exit(1);
    }
    
    housekeeping::start(); // Starts housekeeping loop
}

/// Run this function on App exit.
fn on_exit(handle: &AppHandle) {
    if let Err(e) = kill_quicklaunch(handle) { // Attempt to kill quicklaunch
        error!("main::on_exit() error! Failed to kill quicklaunch! Quicklaunch possibly still running. | Err: {e}");
    }
}

fn main() {
    init();

    tauri::Builder::default()
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
            build_quicklaunch,
            show_quicklaunch,
            hide_quicklaunch,
        ])
        .build(tauri::generate_context!())
        .expect("Failed to build Tauri app!")
        .run(move |app_handle, event| match event {
            RunEvent::ExitRequested { api, .. } => {
                on_exit(&app_handle);
            }
            _ => {}
        });
}
