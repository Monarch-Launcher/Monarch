// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

/*
mod monarch_games;
mod monarch_library;
mod monarch_utils;

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
        if let Err(e) =
            MONARCH_STATE.set_library_games(&crate::monarch_games::monarch_client::get_library())
        {
            panic!("init() Failed to set library games in state! | Err: {e}")
        }
    }

    housekeeping::start(); // Starts housekeeping loop
}

*/

use crate::gui::App;

mod gui;

fn main() {
    let monarch: App = App::new();
    monarch.run();
}
