// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

use log::error;

mod monarch_games;
mod monarch_library;
mod monarch_utils;

use monarch_games::commands::{
    download_game, get_library, launch_game, open_store, refresh_library, remove_game, search_games,
};
use monarch_games::monarch_client;
use monarch_library::commands::{
    create_collection, delete_collection, get_collections, update_collection,
};
use monarch_utils::commands::{
    clear_cached_images, delete_password, get_settings, open_logs, revert_settings, set_password,
    set_settings,
};
use monarch_utils::monarch_fs::verify_monarch_folders;
use monarch_utils::monarch_logger::init_logger;
use monarch_utils::quicklaunch::init_quicklaunch;
use monarch_utils::{housekeeping, monarch_settings};
use tauri::{AppHandle, Manager};

pub static mut GLOBAL_APPHANDLE: Option<Box<AppHandle>> = None;

async fn init() {
    if let Err(e) = monarch_settings::init() {
        // Crash program if this fails
        panic!("Error during settings initialization! | Err: {e}");
    }
    init_logger(); // Starts logger
    verify_monarch_folders(); // Checks that directories are as Monarch expects
    housekeeping::start(); // Starts housekeeping loop

    // Usage if GLOBAL_APPHANDLE requires unsafe, due to it being mut
    let param_handle: AppHandle;
    unsafe {
        param_handle = *(GLOBAL_APPHANDLE.clone()).unwrap();
    }
    init_quicklaunch(&param_handle)
        .await
        .expect("Failed to setup quicklaunch");
}

#[tokio::main]
async fn main() {
    // Build Monarch Tauri app
    let monarch = tauri::Builder::default()
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
        .build(tauri::generate_context!())
        .expect("Failed to build Monarch!");

    // Set GLOBAL_APPHANDLE
    unsafe {
        GLOBAL_APPHANDLE = Some(Box::new(monarch.app_handle()));
        if GLOBAL_APPHANDLE.is_none() {
            panic!("Setting GLOBAL_APPHANDLE failed!")
        }
    }

    // Run some initial checks and setup
    init().await;

    // Start Monarch
    monarch.run(|_app_handle, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = event {
            api.prevent_exit();
        }
    });
}
