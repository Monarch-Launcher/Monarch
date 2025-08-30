// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

mod monarch_games;
mod monarch_library;
mod monarch_utils;

use std::process::exit;

use futures::executor;
use monarch_games::commands::{
    download_game, get_home_recomendations, get_library, launch_game, move_game_to_monarch,
    open_store, proton_versions, refresh_library, remove_game, search_games, update_game,
    update_game_properties, manual_add_game, manual_remove_game, umu_is_installed, install_umu, get_executables
};
use monarch_library::commands::{
    create_collection, delete_collection, get_collections, update_collection,
};
use monarch_utils::commands::{
    async_read_from_pty, async_write_to_pty, clear_cached_images, close_terminal, delete_password,
    delete_secret, get_settings, open_logs, open_terminal, revert_settings, set_password,
    set_secret, set_settings, zoom_window,
};
use monarch_utils::monarch_fs::verify_monarch_folders;
use monarch_utils::monarch_logger::init_logger;
use monarch_utils::{housekeeping, monarch_settings};
use tauri::Manager;
use tracing::{error, info};

use crate::monarch_utils::monarch_state::MONARCH_STATE;
use crate::monarch_utils::quicklaunch::{init_quicklaunch, quicklaunch_is_enabled};

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

fn init() {
    if let Err(e) = monarch_settings::init() {
        // Crash program if this fails
        panic!("Error during settings initialization! | Err: {e}");
    }
    init_logger(); // Starts logger
    verify_monarch_folders(); // Checks that directories are as Monarch expects

    // Set initial monarch state
    unsafe {
        if let Err(e) = MONARCH_STATE.set_library_games(&crate::monarch_games::monarch_client::get_library()) {
            panic!("init() Failed to set library games in state! | Err: {e}")
        }
    }

    housekeeping::start(); // Starts housekeeping loop
}

fn main() {
    // Run some initial checks and setup
    init();

    // Setting this enviornment variable fixes performance issues when
    // scrolling under Linux.
    // Also appears like it might help with weird multiwindow rendering
    // behaviour.
    #[cfg(target_os = "linux")]
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");

    // Build Monarch Tauri app
    let monarch = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
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
            async_read_from_pty,
            async_write_to_pty,
            open_terminal,
            close_terminal,
            set_secret,
            delete_secret,
            update_game_properties,
            move_game_to_monarch,
            proton_versions,
            manual_add_game,
            manual_remove_game,
            zoom_window,
            umu_is_installed,
            install_umu,
            get_executables,
        ])
        .setup(|app| {
            #[cfg(desktop)]
            {
                if quicklaunch_is_enabled() {
                    if let Err(e) = executor::block_on(init_quicklaunch(app.handle())) {
                        error!("main() Failed to initialize quicklaunch! | Err: {e}")
                    }
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Only exit monarch on main window close
                if window.title().expect("Failed to get window title!") == "Monarch" {
                    // Closure to close all windows on main window close.
                    api.prevent_close();
                    window.app_handle().cleanup_before_exit();
                    // Log success and exit
                    info!("main() Tauri cleanup done! Exiting...");
                    exit(0);
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("Failed to build Monarch!");

    // Start Monarch
    monarch.run(|_app_handle, _event| {
        // Monarch running...
    });
}
