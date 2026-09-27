// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

use monarch_app::gui::App;
use monarch_core::monarch_utils::{monarch_logger::init_logger, monarch_settings};

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

fn main() {
    if let Err(e) = monarch_settings::init() {
        // Crash program if this fails
        panic!("Error during settings initialization! | Err: {e}");
    }

    init_logger(); // Starts logger

    #[cfg(target_os = "windows")]
    monarch_app::window::apply_rounded_corners();

    // Run Monarch
    App::run();
}
