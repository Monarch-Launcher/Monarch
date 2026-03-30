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
        housekeeping, monarch_fs::verify_monarch_folders, monarch_logger::init_logger, monarch_settings, monarch_sql::init_db, monarch_state::MONARCH_STATE
    },
};

use tracing::{debug, error};

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

async fn init() {
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

    match MONARCH_STATE.read() {
        Ok(state) => {
            let pool = state.get_db_pool_ref();
            init_db(pool).await.expect("Failed to run init_db()!"); // Verify database tables exist
        }
        Err(e) => {
            error!("Failed to acquire read lock on MONARCH_STATE! | Err: {e}");
            panic!("Failed to acquire read lock on MONARCH_STATE! | Err: {e}")
        }
    }

    housekeeping::start(); // Starts housekeeping loop
}

#[tokio::main]
async fn main() {
    init().await;

    // Run Monarch
    App::run();
}
