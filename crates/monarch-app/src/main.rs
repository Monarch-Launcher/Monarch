// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

use monarch_core::{
    monarch_library::library,
    monarch_utils::{
        housekeeping,
        monarch_fs::verify_monarch_folders,
        monarch_logger::init_logger,
        monarch_settings,
        monarch_sql::{init_db, repair_or_migrate_db},
        monarch_state::MONARCH_STATE,
    },
};

use monarch_app::gui::App;

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
        .init()
        .await;

    debug!("Initialised with MONARCH_STATE: {:?}", MONARCH_STATE);

    verify_monarch_folders(); // Checks that directories are as Monarch expects

    match MONARCH_STATE.read() {
        Ok(state) => {
            let pool = state.get_db_pool_arc();
            init_db(&pool).await.expect("Failed to run init_db()!"); // Verify database tables exist
            repair_or_migrate_db(&pool)
                .await
                .expect("Failed to run repair_or_migrate_db()!"); // Verify database tables structure
        }
        Err(e) => {
            error!("Failed to acquire read lock on MONARCH_STATE! | Err: {e}");
            panic!("Failed to acquire read lock on MONARCH_STATE! | Err: {e}")
        }
    }

    let games = library::get_games().await.expect("Didn't expect to fail!");
    MONARCH_STATE
        .write()
        .expect("Failed to aquire write lock on MONARCH_STATE")
        .set_library_games(&games);

    housekeeping::start(); // Starts housekeeping loop

    // Checks for updates of games managed by Monarch on a background thread,
    // only runs if enabled in settings.
    monarch_core::monarch_games::updates::start_startup_check();
}

fn main() {
    futures::executor::block_on(init());

    #[cfg(target_os = "windows")]
    monarch_app::window::apply_rounded_corners();

    // Run Monarch
    App::run();
}
