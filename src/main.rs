// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

mod gui;
mod monarch_games;
mod monarch_library;
mod monarch_utils;

use crate::{
    gui::App,
    monarch_utils::{
        housekeeping, monarch_fs::verify_monarch_folders, monarch_logger::init_logger,
        monarch_settings, monarch_state::MONARCH_STATE,
    },
};

use tracing::debug;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

fn init() {
    if let Err(e) = monarch_settings::init() {
        // Crash program if this fails
        panic!("Error during settings initialization! | Err: {e}");
    }

    init_logger(); // Starts logger

    MONARCH_STATE
        .write()
        .expect("Failed to aquire write lock on MONARCH_STATE")
        .init();

    debug!("Initialised with MONARCH_STATE: {:?}", MONARCH_STATE);

    verify_monarch_folders(); // Checks that directories are as Monarch expects

    housekeeping::start(); // Starts housekeeping loop
}

fn main() {
    init();

    // Run Monarch
    App::run();
}
