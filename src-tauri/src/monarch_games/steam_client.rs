use anyhow::{bail, Context, Result};
use reqwest;
use scraper::{Html, Selector};
use serde_json::Value;
use simple_steam_totp::generate;
use std::path::PathBuf;
use tauri::AppHandle;
use tokio::task;
use tracing::{error, info, warn};

use super::monarchgame::{MonarchGame, MonarchWebGame};
use crate::monarch_utils::monarch_credentials::get_password;
use crate::monarch_utils::monarch_fs::{
    generate_cache_image_path, generate_library_image_path, get_monarch_home, path_exists,
};
use crate::monarch_utils::monarch_settings::{get_settings_state, LauncherSettings};

#[cfg(target_os = "windows")]
use super::windows::steam;

#[cfg(target_os = "macos")]
use super::macos::steam;

#[cfg(target_os = "linux")]
use super::linux::steam;

/*
* This file acts like a general interface between commands.rs and Steam.
*
* Basically just some fancy OS specific behaviour gets abstracted away for easier readabilty.
*/

/// Returns if SteamCMD is installed on system or not.
pub fn is_installed() -> bool {
    let path: PathBuf = get_steamcmd_dir();
    path_exists(&path)
}

/// Downloads and installs SteamCMD on users computer.
pub async fn download_and_install(handle: &AppHandle) -> Result<()> {
    steam::install_steamcmd(handle)
        .await
        .with_context(|| "steam_client::download_and_install() -> ")
}

/// Returns games installed by Steam Client.
pub async fn get_library() -> Vec<MonarchGame> {
    steam::get_library().await
}

/// Attempts to launch Steam Client game.
pub fn launch_client_game(game: &MonarchGame) -> Result<()> {
    let command: String = format!("steam://rungameid/{}", &game.platform_id);
    steam::run_command(&command).with_context(|| "steam_client::launch_game() -> ")
}

/// Attempts to uninstall a Steam Client game.
pub fn uninstall_client_game(id: &str) -> Result<()> {
    let mut command: String = String::from("steam://uninstall/");
    command.push_str(id);
    steam::run_command(&command).with_context(|| "steam_client::launch_game() -> ")
}

/// Attemps to launch SteamCMD game.
pub async fn launch_cmd_game(handle: &AppHandle, game: &MonarchGame) -> Result<()> {
    let settings = get_settings_state();
    let steam_settings = settings.steam;
    let login_arg = get_steamcmd_login(&steam_settings)?;

    let args: Vec<&str> = vec![
        "+@ShutdownOnFailedCommand 1",
        &login_arg,
        "+app_launch",
        &game.platform_id,
        &game.launch_args,
        "+quit",
    ];

    steam::steamcmd_command(handle, args)
        .await
        .with_context(|| "steam_client::launch_cmd_game() -> ")
}

/// Download a Steam game via Monarch and SteamCMD.
pub async fn download_game(handle: &AppHandle, name: &str, id: &str) -> Result<MonarchGame> {
    let settings = get_settings_state();
    let steam_settings = settings.steam;

    if !steam_settings.manage {
        warn!("steam_client::download_game() User tried to install game without allowing Monarch to manage Steam! Cancelling download...");
        bail!("steam_client::download_game() | Err: Not allowed to manage games. Check settings.")
    }

    let mut install_dir: PathBuf = PathBuf::from(settings.monarch.game_folder);
    let sanitized_name = name.replace(" ", "\\ ");
    install_dir.push(sanitized_name);

    // Directory argument
    // TODO: Figure out why force_install_dir wipes libraryfolders.vdf
    //let mut install_dir_arg: String = String::from("+force_install_dir ");
    //install_dir_arg.push_str(&install_dir.to_string_lossy());

    // Login argument
    let login_arg =
        get_steamcmd_login(&steam_settings).with_context(|| "steam_client::download_game() -> ")?;

    // App ID argument
    let mut download_arg = String::from("+app_update ");
    download_arg.push_str(id);
    download_arg.push_str(" validate");

    // Build the command as a string with arguments in order
    let command: Vec<&str> = vec![
        "+@ShutdownOnFailedCommand 1",
        &login_arg,
        &download_arg,
        "+quit",
    ];

    // TODO: Wait for Steamcmd to return
    // TODO: steam::steamcmd_command() should wait for SteamCMD to finish
    steam::steamcmd_command(handle, command)
        .await
        .with_context(|| "steam_client::download_game() -> ")?;

    let mut monarchgame: MonarchGame =
        parse_steam_ids(&[String::from(id)], false, true).await[0].clone();
    monarchgame.platform = "steamcmd".to_string();
    Ok(monarchgame)
}

/// Uninstall a Steam game via SteamCMD
pub async fn uninstall_game(handle: &AppHandle, id: &str) -> Result<()> {
    let steam_settings = get_settings_state().steam;
    if !steam_settings.manage {
        warn!("steam_client::uninstall_game() User tried to uninstall game without allowing Monarch to manage Steam! Cancelling uninstall...");
        bail!("steam_client::download_game() | Err: Not allowed to manage games. Check settings.")
    }

    let login_arg = get_steamcmd_login(&steam_settings)?;
    let remove_arg: String = format!("+app_uninstall {id}");
    let command: Vec<&str> = vec![
        "+@ShutdownOnFailedCommand 1",
        &login_arg,
        &remove_arg,
        "+quit",
    ];

    steam::steamcmd_command(handle, command)
        .await
        .with_context(|| "steam_client::uninstall_game() -> ")
}

/// Uninstall a Steam game via SteamCMD
pub async fn update_game(handle: &AppHandle, id: &str) -> Result<()> {
    let steam_settings = get_settings_state().steam;
    if !steam_settings.manage {
        warn!("steam_client::uninstall_game() User tried to uninstall game without allowing Monarch to manage Steam! Cancelling uninstall...");
        bail!("steam_client::download_game() | Err: Not allowed to manage games. Check settings.")
    }

    let login_arg = get_steamcmd_login(&steam_settings)?;
    let update_arg: String = format!("+app_update {id} validate");
    let command: Vec<&str> = vec![
        "+@ShutdownOnFailedCommand 1",
        &login_arg,
        &update_arg,
        "+quit",
    ];

    steam::steamcmd_command(handle, command)
        .await
        .with_context(|| "steam_client::update_game() -> ")
}

/// Returns path to Monarchs installed version of SteamCMD
pub fn get_steamcmd_dir() -> PathBuf {
    let path: PathBuf = get_monarch_home();
    path.join("SteamCMD")
}

/// Converts SteamApp ids into MonarchGames.
pub async fn parse_steam_ids(
    ids: &[String],
    is_cache: bool,
    using_monarch: bool,
) -> Vec<MonarchGame> {
    let mut tasks = Vec::new();
    let mut games: Vec<MonarchGame> = Vec::new();

    for id in ids {
        let new_task = if using_monarch {
            task::spawn(parse_id_monarch_com(id.clone(), is_cache))
        } else {
            task::spawn(parse_id_steampowered_com(id.clone(), is_cache))
        };
        tasks.push(new_task);
    }

    for task in tasks {
        if let Ok(finished_task) = task.await {
            if let Ok(game) = finished_task {
                games.push(game);
            }
        }
    }

    games
}

/// Since login is used for multiple commands it gets
/// abstracted to it's own function.
fn get_steamcmd_login(steam_settings: &LauncherSettings) -> Result<String> {
    let username: &str = &steam_settings.username;
    let password: String = match get_password("steam", &username) {
        Ok(p) => p,
        Err(e) => {
            warn!("steam_client::get_steamcmd_login() Failed to get password for {username}! | Err: {e}");
            info!("SteamCMD will prompt for password.");
            String::from("")
        }
    };

    // Login argument
    let mut login_arg = String::from("+login ");
    login_arg.push_str(username);

    if !password.is_empty() {
        login_arg.push(' ');
        login_arg.push_str(&password);
    }

    // Current solution is to store the secret in keystore, which essentially
    // disables the point of 2fa, at least on computers with Monarch.
    // TODO: Look into other possible solutions for Steamgaurd.
    match get_password("steam-secret", username) {
        Ok(secret) => {
            if !secret.is_empty() {
                info!("Steam TOTP detected in Monarch!");
                let totp = generate(&secret).unwrap();
                login_arg.push(' ');
                login_arg.push_str(&totp);
            } else {
                warn!("Steam TOTP was found! However the string was empty.");
            }
        }
        Err(e) => {
            error!("steam_client::get_steamcmd_login() Did not find steam secret. | Err: {e}");
            warn!("No Steam TOTP detected! Might require mobile 2fa.");
        }
    }

    Ok(login_arg)
}

/// Helper function to parse individual steam ids. Allows for concurrent parsing.
async fn parse_id_monarch_com(id: String, is_cache: bool) -> Result<MonarchGame> {
    info!("Parsing {id} via monarch-launcher.com.");
    let mut game_info_opt: Option<MonarchWebGame> = None;
    let target: String =
        format!("https://monarch-launcher.com/api/games?platform=steam&platform_id={id}");

    // GET info from Steam servers
    match reqwest::get(&target).await {
        Ok(response) => match response.text().await {
            Ok(body) => {
                let web_games: Vec<MonarchWebGame> = serde_json::from_str(&body).unwrap();
                if web_games.is_empty() {
                    bail!("Nothing returned for game with ID: {id}");
                }
                game_info_opt = Some(web_games.first().unwrap().clone());
            }
            Err(e) => {
                warn!("steam_client::parse_steam_ids() Failed to parse response body! | Err: {e}");
            }
        },
        Err(e) => {
            error!(
                "steam_client::parse_steam_ids() Failed to get response from: {target} | Err: {e}"
            );
        }
    }

    // Parse content into MonarchGame
    if let Some(game_info) = game_info_opt {
        let thumbnail_path = if is_cache {
            String::from(generate_cache_image_path(&game_info.name).to_str().unwrap())
        } else {
            String::from(
                generate_library_image_path(&game_info.name)
                    .to_str()
                    .unwrap(),
            )
        };

        let mut monarch_game = MonarchGame::from(&game_info);
        monarch_game.thumbnail_path = thumbnail_path;
        monarch_game.download_thumbnail(game_info.cover_url).await;
        return Ok(monarch_game);
    }

    warn!("Failed to parse Steam game with id: {id}");
    bail!("Failed to parse Steam game with id: {id}")
}

/// Function to search steam store directly from Monarch client, skipping monarch-launcher.com
pub async fn find_game(name: &str) -> Vec<MonarchGame> {
    let mut target: String = String::from("https://store.steampowered.com/search/?term=");
    target.push_str(name);

    let mut games: Vec<MonarchGame> = Vec::new();

    if let Ok(response) = reqwest::get(&target).await {
        if let Ok(body) = response.text().await {
            games = parse_steam_page(&body).await;
        }
    }
    games
}

/// Gets AppIDs and Links from Steam store search
async fn parse_steam_page(body: &str) -> Vec<MonarchGame> {
    let mut ids: Vec<String> = Vec::new();
    let mut links: Vec<String> = Vec::new();

    let game_selector = Selector::parse("a.search_result_row.ds_collapse_flag").unwrap(); // Has to be unwrap rn.

    for css_elem in Html::parse_document(body).select(&game_selector) {
        // Check for AppID
        if let Some(id) = css_elem.value().attr("data-ds-appid") {
            ids.push(id.to_string());

            // Check for link to steam page
            if let Some(link) = css_elem.value().attr("href") {
                links.push(link.to_string());
            } else {
                // Else remove
                ids.pop();
            }
        }
    }

    parse_steam_ids(&ids, true, false).await
}

/// Helper function to parse individual steam ids. Allows for concurrent parsing.
async fn parse_id_steampowered_com(id: String, is_cache: bool) -> Result<MonarchGame> {
    info!("Parsing {id} via Steam.");
    let target: String = format!("https://store.steampowered.com/api/appdetails?appids={id}");

    let game_info: String;
    // GET info from Steam servers
    match reqwest::get(&target).await {
        Ok(response) => match response.text().await {
            Ok(body) => {
                game_info = body;
            }
            Err(e) => {
                warn!("steam_client::parse_steam_ids() warning! Failed to parse response body! | Err: {e}");
                bail!("Error when getting request body!");
            }
        },
        Err(e) => {
            error!("steam_client::parse_steam_ids() warning! Failed to get respnse from: {target} | Err: {e}");
            bail!("Error when running GET {target}");
        }
    }

    let game_json: Value = serde_json::from_str(&game_info).unwrap();
    let name: String = game_json[&id]["data"]["name"]
        .to_string()
        .trim_matches('"')
        .to_string();

    let store_url = format!("https://store.steampowered.com/app/{id}");
    let cover_url: String =
        format!("https://steamcdn-a.akamaihd.net/steam/apps/{id}/library_600x900_2x.jpg");

    // Parse content into MonarchGame
    let thumbnail_path = if is_cache {
        String::from(generate_cache_image_path(&name).to_str().unwrap())
    } else {
        String::from(generate_library_image_path(&name).to_str().unwrap())
    };
    let monarch_game = MonarchGame::new(&name, -1, "steam", &id, &store_url, "", &thumbnail_path);
    monarch_game.download_thumbnail(cover_url).await;
    Ok(monarch_game)
}
