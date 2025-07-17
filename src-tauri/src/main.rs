// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

mod monarch_games;
mod monarch_library;
mod monarch_utils;

use std::process::exit;

use monarch_games::commands::{
    download_game, get_home_recomendations, get_library, launch_game, open_store, refresh_library,
    remove_game, search_games, update_game, update_game_properties, move_game_to_monarch
};
use monarch_library::commands::{
    create_collection, delete_collection, get_collections, update_collection,
};
use monarch_utils::commands::{
    async_read_from_pty, async_write_to_pty, clear_cached_images, close_terminal, delete_password,
    delete_secret, get_settings, hide_quicklaunch, init_quicklaunch, open_logs, open_terminal,
    quicklaunch_is_enabled, revert_settings, set_password, set_secret, set_settings,
    show_quicklaunch,
};
use monarch_utils::monarch_fs::verify_monarch_folders;
use monarch_utils::monarch_logger::init_logger;
use monarch_utils::{housekeeping, monarch_settings};
use tauri::Manager;
use tracing::{info, warn};

fn init() {
    if let Err(e) = monarch_settings::init() {
        // Crash program if this fails
        panic!("Error during settings initialization! | Err: {e}");
    }
    init_logger(); // Starts logger
    verify_monarch_folders(); // Checks that directories are as Monarch expects
    housekeeping::start(); // Starts housekeeping loop
}

fn main() {
    // Setting this enviornment variable fixes performance issues when
    // scrolling under Linux.
    // Also appears like it might help with weird multiwindow rendering
    // behaviour.
    #[cfg(target_os = "linux")]
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");

    // Build Monarch Tauri app
    let monarch = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            search_games,
            refresh_library,
            get_library,
            launch_game,
            download_game,
            update_game,
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
            get_home_recomendations,
            init_quicklaunch,
            show_quicklaunch,
            hide_quicklaunch,
            quicklaunch_is_enabled,
            async_read_from_pty,
            async_write_to_pty,
            open_terminal,
            close_terminal,
            set_secret,
            delete_secret,
            update_game_properties,
            move_game_to_monarch,
        ])
        .on_window_event(|event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
                // Only exit monarch on main window close
                if event.window().title().expect("Failed to get window title!") == "Monarch" {
                    // Closure to close all windows on main window close.
                    api.prevent_close();
                    // Iterate over all windows (eg. quicklaunch)
                    for (name, window) in event.window().app_handle().windows() {
                        if let Err(e) = window.close() {
                            warn!("Failed to close window: {name} | Err: {e}");
                        }
                    }
                    // Log success and exit
                    info!("main() All windows closed! Exiting...");
                    exit(0);
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("Failed to build Monarch!");

    // Run some initial checks and setup
    init();

    // Start Monarch
    monarch.run(|_app_handle, _event| {
        // Monarch running...
    });
}
