// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod monarch_utils;
mod monarch_games;
use monarch_utils::monarch_fs::init_monarch_fs;
use monarch_utils::logger::init_logger;
use monarch_games::commands::{steam_downloader, launch_steam_game, search_games,
                              blizzard_downloader, launch_blizzard_game, epic_downloader};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    init_monarch_fs();
    init_logger().unwrap();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            steam_downloader,
            launch_steam_game,
            search_games,
            blizzard_downloader,
            launch_blizzard_game,
            epic_downloader
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
