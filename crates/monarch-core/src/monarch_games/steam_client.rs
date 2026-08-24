use super::stores::StoreType;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use reqwest;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json::Value;
use simple_steam_totp::generate;
use std::path::PathBuf;
use tokio::task;
use tracing::{debug, error, info, warn};

use super::monarchgame::{MonarchGame, MonarchWebApiGame};
use crate::monarch_games::games::GameType;
use crate::monarch_games::games::SearchResult;
use crate::monarch_games::monarchgame::GameImageType;
use crate::monarch_games::monarchgame::StoreInfo;
use crate::monarch_games::stores::DownloadOptions;
use crate::monarch_games::stores::SearchFilter;
use crate::monarch_library::library;
use crate::monarch_utils::monarch_credentials::get_password;
use crate::monarch_utils::monarch_fs::{generate_cache_image_path, generate_library_image_path};
use crate::monarch_utils::monarch_settings::{get_settings, LauncherSettings};

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

pub struct SteamClient {}

impl SteamClient {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl StoreType for SteamClient {
    async fn search_games(&self, name: &str, _filter: &SearchFilter) -> Vec<Box<dyn SearchResult>> {
        find_game(name)
            .await
            .into_iter()
            .map(|g| Box::new(MonarchWebApiGame::from_monarchgame(g)) as Box<dyn SearchResult>)
            .collect::<Vec<Box<dyn SearchResult>>>()
    }

    async fn install_game(&self, game: &MonarchGame, _opts: &DownloadOptions) -> Result<()> {
        let game: MonarchGame = download_game(&game.name, &game.get_store_id())
            .await
            .with_context(|| "steam_client::install_game() -> ")?;
        library::add_game(&game)
            .await
            .with_context(|| "steam_client::install_game() -> ")
    }

    async fn uninstall_game(&self, game: &MonarchGame) -> Result<()> {
        match game.get_store_name().as_str() {
            "steam" => uninstall_client_game(&game.get_store_id())
                .with_context(|| "steam_client::uninstall_game() -> "),
            "steamcmd" => uninstall_game(&game.get_store_id())
                .await
                .with_context(|| "steam_client::uninstall_game() -> "),
            _ => {
                bail!(
                    "Invalid store! Expected 'steam' or 'steamcmd', instead got: {}!",
                    game.get_store_name()
                )
            }
        }
    }

    async fn update_game(&self, game: &MonarchGame) -> Result<()> {
        update_game(&game.get_store_id())
            .await
            .with_context(|| "steam_client::update_game() -> ")
    }

    fn game_is_installed(&self, _store_id: &str) -> bool {
        unimplemented!()
    }

    fn store_enabled(&self) -> bool {
        unimplemented!()
    }

    async fn launch_game(&self, game: &MonarchGame) -> Result<()> {
        match game.get_store_name().as_str() {
            "steam" => launch_client_game(game),
            "steamcmd" => {
                let game_clone: MonarchGame = game.clone();
                tokio::spawn(async move {
                    if let Err(e) = launch_cmd_game(&game_clone).await {
                        error!(
                            "steam_client::SteamClient::launch_game() -> {}",
                            e.chain().map(|e| e.to_string()).collect::<String>()
                        );
                    }
                });
                Ok(())
            }
            _ => bail!("Neither Steam client nor SteamCMD was detected as game store!"),
        }
    }
}

/// Returns if SteamCMD is installed on system or not.
pub fn steamcmd_is_installed() -> bool {
    steam::steamcmd_is_installed()
}

/// Downloads and installs SteamCMD on users computer.
pub async fn install_steamcmd() -> Result<()> {
    steam::install_steamcmd()
        .await
        .with_context(|| "steam_client::install_steamcmd() -> ")?;

    // Perform initial run of SteamCMD to create necessary files
    steam::steamcmd_command(vec!["+quit"])
        .await
        .with_context(|| "steam_client::install_steamcmd() -> ")?;

    // Symlink files needed for SteamCMD globaluser
    #[cfg(target_os = "linux")]
    {
        use crate::{monarch_games::linux::steam, monarch_utils::monarch_fs::get_monarch_home};

        let src_path: PathBuf = steam::get_default_location()
            .with_context(|| "steam_client::install_steamcmd() -> ")?;

        /*
        -------- Use this when figuring out how to put steamcmd in .local/bin and .local/lib --------

        let dest_path: PathBuf = get_unix_home()
            .unwrap()
            .join(".local")
            .join("lib")
            .join("steamcmd")
            .join("linux32");
        */

        let dest_path: PathBuf = get_monarch_home().join("steamcmd").join("linux32");

        let reaper_src: PathBuf = src_path.join("ubuntu12_32").join("reaper");
        let wrapper_src: PathBuf = src_path.join("ubuntu12_32").join("steam-launch-wrapper");
        let steamservice_src: PathBuf = src_path.join("ubuntu12_32").join("steamservice.so");

        let reaper_dest: PathBuf = dest_path.join("reaper");
        let wrapper_dest: PathBuf = dest_path.join("steam-launch-wrapper");
        let steamservice_dest: PathBuf = dest_path.join("steamservice.so");

        std::os::unix::fs::symlink(&reaper_src, &reaper_dest).with_context(|| {
            format!(
                "steam_client::install_steamcmd() Failed to symlink: {} -> {} | Err: ",
                reaper_src.display(),
                reaper_dest.display()
            )
        })?;
        std::os::unix::fs::symlink(&wrapper_src, &wrapper_dest).with_context(|| {
            format!(
                "steam_client::install_steamcmd() Failed to symlink: {} -> {} | Err: ",
                wrapper_src.display(),
                wrapper_dest.display()
            )
        })?;
        std::os::unix::fs::symlink(&steamservice_src, &steamservice_dest).with_context(|| {
            format!(
                "steam_client::install_steamcmd() Failed to symlink: {} -> {} | Err: ",
                steamservice_src.display(),
                steamservice_dest.display()
            )
        })?;
    }

    // Initial login to cache user credentials
    let login_arg = {
        let settings_lock = match get_settings() {
            Ok(lock) => lock,
            Err(e) => bail!(
                "steam_client::install_steamcmd() Failed to get settings | Err: {}",
                e
            ),
        };
        let settings = match settings_lock.read() {
            Ok(settings) => settings,
            Err(e) => bail!(
                "steam_client::install_steamcmd() Failed to get settings read lock | Err: {}",
                e
            ),
        };

        get_steamcmd_login(&settings.steam)
            .with_context(|| "steam_client::install_steamcmd() -> ")?
    };

    steam::steamcmd_command(vec!["-globaluser", &login_arg, "+quit"])
        .await
        .with_context(|| "steam_client::install_steamcmd() -> ")?;

    Ok(())
}

pub fn remove_steamcmd() -> Result<()> {
    if !steamcmd_is_installed() {
        warn!("SteamCMD not found!");
        bail!("SteamCMD not found!")
    }

    // TODO: Remove SteamCMD
    Ok(())
}

/// Returns games installed by Steam Client.
pub async fn get_library() -> Vec<MonarchGame> {
    let mut games = steam::get_library().await;
    for game in &mut games {
        game.is_installed = true;
    }
    games
}

/// Attempts to launch Steam Client game.
pub fn launch_client_game(game: &MonarchGame) -> Result<()> {
    let command: String = format!("steam://rungameid/{}", &game.get_store_id());
    steam::run_command(&command).with_context(|| "steam_client::launch_game() -> ")
}

/// Attempts to uninstall a Steam Client game.
pub fn uninstall_client_game(id: &str) -> Result<()> {
    let mut command: String = String::from("steam://uninstall/");
    command.push_str(id);
    steam::run_command(&command).with_context(|| "steam_client::launch_game() -> ")
}

/// Attemps to launch SteamCMD game.
pub async fn launch_cmd_game(game: &MonarchGame) -> Result<()> {
    let login_arg = {
        let settings_lock = match get_settings() {
            Ok(lock) => lock,
            Err(e) => bail!(
                "steam_client::launch_cmd_game() Failed to get settings | Err: {}",
                e
            ),
        };
        let settings = match settings_lock.read() {
            Ok(settings) => settings,
            Err(e) => bail!(
                "steam_client::launch_cmd_game() Failed to get settings read lock | Err: {}",
                e
            ),
        };

        get_steamcmd_login(&settings.steam)
            .with_context(|| "steam_client::launch_cmd_game() -> ")?
    };

    let id = game.get_store_id();

    let args: Vec<&str> = vec![
        "+@ShutdownOnFailedCommand 1",
        &login_arg,
        "+app_launch",
        &id,
        game.launch_args.as_deref().unwrap_or_default(),
    ];

    steam::steamcmd_command(args)
        .await
        .with_context(|| "steam_client::launch_cmd_game() -> ")
}

/// Download a Steam game via Monarch and SteamCMD.
pub async fn download_game(name: &str, id: &str) -> Result<MonarchGame> {
    let login_arg = {
        let settings_lock = match get_settings() {
            Ok(lock) => lock,
            Err(e) => bail!(
                "steam_client::download_game() Failed to get settings | Err: {}",
                e
            ),
        };
        let settings = match settings_lock.read() {
            Ok(settings) => settings,
            Err(e) => bail!(
                "steam_client::download_game() Failed to get settings read lock | Err: {}",
                e
            ),
        };

        if !settings.steam.manage {
            warn!("steam_client::download_game() User tried to install game without allowing Monarch to manage Steam! Cancelling download...");
            bail!(
                "steam_client::download_game() | Err: Not allowed to manage games. Check settings."
            )
        }

        let mut install_dir: PathBuf = PathBuf::from(&settings.monarch.game_folder);
        let sanitized_name: String = name.replace(" ", "\\ ");
        install_dir.push(sanitized_name);

        // Directory argument
        // TODO: Figure out why force_install_dir wipes libraryfolders.vdf
        //let mut install_dir_arg: String = String::from("+force_install_dir ");
        //install_dir_arg.push_str(&install_dir.to_string_lossy());

        // Login argument
        get_steamcmd_login(&settings.steam).with_context(|| "steam_client::download_game() -> ")?
    };

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
    steam::steamcmd_command(command)
        .await
        .with_context(|| "steam_client::download_game() -> ")?;

    let mut monarchgame: MonarchGame =
        parse_steam_ids(&[String::from(id)], false, true).await[0].clone();

    monarchgame.stores.push(StoreInfo {
        name: "steamcmd".to_string(),
        store_id: id.to_string(),
        store_url: "".to_string(),
    });

    monarchgame.managed_by_monarch = true;

    Ok(monarchgame)
}

/// Uninstall a Steam game via SteamCMD
pub async fn uninstall_game(id: &str) -> Result<()> {
    let login_arg = {
        let settings_lock = match get_settings() {
            Ok(lock) => lock,
            Err(e) => bail!(
                "steam_client::uninstall_game() Failed to get settings | Err: {}",
                e
            ),
        };
        let settings = match settings_lock.read() {
            Ok(settings) => settings,
            Err(e) => bail!(
                "steam_client::uninstall_game() Failed to get settings read lock | Err: {}",
                e
            ),
        };

        if !settings.steam.manage {
            warn!("steam_client::uninstall_game() User tried to uninstall game without allowing Monarch to manage Steam! Cancelling uninstall...");
            bail!("steam_client::uninstall_game() | Err: Not allowed to manage games. Check settings.")
        }

        get_steamcmd_login(&settings.steam)?
    };

    let remove_arg: String = format!("+app_uninstall {id}");
    let command: Vec<&str> = vec![
        "+@ShutdownOnFailedCommand 1",
        &login_arg,
        &remove_arg,
        "+quit",
    ];

    steam::steamcmd_command(command)
        .await
        .with_context(|| "steam_client::uninstall_game() -> ")
}

/// Uninstall a Steam game via SteamCMD
pub async fn update_game(id: &str) -> Result<()> {
    let login_arg = {
        let settings_lock = match get_settings() {
            Ok(lock) => lock,
            Err(e) => bail!(
                "steam_client::update_game() Failed to get settings | Err: {}",
                e
            ),
        };
        let settings = match settings_lock.read() {
            Ok(settings) => settings,
            Err(e) => bail!(
                "steam_client::update_game() Failed to get settings read lock | Err: {}",
                e
            ),
        };

        if !settings.steam.manage {
            warn!("steam_client::update_game() User tried to update game without allowing Monarch to manage Steam! Cancelling update...");
            bail!("steam_client::update_game() | Err: Not allowed to manage games. Check settings.")
        }

        get_steamcmd_login(&settings.steam)?
    };

    let update_arg: String = format!("+app_update {id} validate");
    let command: Vec<&str> = vec![
        "+@ShutdownOnFailedCommand 1",
        &login_arg,
        &update_arg,
        "+quit",
    ];

    steam::steamcmd_command(command)
        .await
        .with_context(|| "steam_client::update_game() -> ")
}

pub fn get_steamcmd_exe() -> PathBuf {
    steam::get_steamcmd_exe()
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
    let mut login_arg = format!("+login {username}");

    if !password.is_empty() {
        login_arg.push_str(&format!(" {password}"));
    }

    // Current solution is to store the secret in keystore, which essentially
    // disables the point of 2fa, at least on computers with Monarch.
    // TODO: Look into other possible solutions for Steamgaurd.
    match get_password("steam-secret", username) {
        Ok(secret) => {
            debug!("Found secret: {secret}");
            if !secret.is_empty() {
                info!("Steam TOTP detected in Monarch!");
                let totp = generate(&secret).unwrap();
                login_arg.push_str(&format!(" {totp}"));
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
    let monarch_url: &'static str = std::env!("MONARCH_URL");

    info!("Parsing {id} via {monarch_url}.");
    let mut game_info_opt: Option<MonarchWebApiGame> = None;
    let target: String = format!("{monarch_url}/api/games?store=steam&store_id={id}");

    // GET info from Steam servers
    match reqwest::get(&target).await {
        Ok(response) => match response.text().await {
            Ok(body) => {
                let web_games: Vec<MonarchWebApiGame> = serde_json::from_str(&body).unwrap();
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
        let mut monarch_game = MonarchGame::from(&game_info);

        monarch_game.stores = game_info.stores.iter().map(StoreInfo::from).collect();

        if is_cache {
            let path: String = String::from(
                generate_cache_image_path(&game_info.name, GameImageType::Cover)
                    .to_str()
                    .unwrap(),
            );
            monarch_game.thumbnail_path = path;
        } else {
            let cover_path: String = String::from(
                generate_library_image_path(&game_info.name, GameImageType::Cover)
                    .to_str()
                    .unwrap(),
            );
            let artwork_path: String = String::from(
                generate_library_image_path(&game_info.name, GameImageType::Artwork)
                    .to_str()
                    .unwrap(),
            );

            monarch_game.thumbnail_path = cover_path;
            monarch_game.artwork_path = artwork_path;
        };

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
        String::from(
            generate_cache_image_path(&name, GameImageType::Cover)
                .to_str()
                .unwrap(),
        )
    } else {
        String::from(
            generate_library_image_path(&name, GameImageType::Cover)
                .to_str()
                .unwrap(),
        )
    };
    let mut monarch_game = MonarchGame::new(&name, -1, "steam", &id, "", &thumbnail_path);
    monarch_game.thumbnail_url = cover_url;
    Ok(monarch_game)
}

#[derive(Deserialize)]
#[allow(unused)]
struct ProtonDbResults {
    #[serde(rename = "bestReportedTier")]
    best_reported_tier: String,

    confidence: String,
    score: f32,
    tier: String,
    total: i32,

    #[serde(rename = "trendingTier")]
    trending_tier: String,
}

/// Queries ProtonDB for game proton support rating
pub async fn get_protondb_rating(steam_appid: &str) -> Result<(String, String)> {
    let target: String =
        format!("https://www.protondb.com/api/v1/reports/summaries/{steam_appid}.json");
    let response = reqwest::get(&target).await?;
    let repsonse_text: String = response.text().await?;

    let proton_rating: ProtonDbResults = serde_json::from_str(&repsonse_text)?;

    Ok((
        proton_rating.tier,
        format!("https://www.protondb.com/app/{steam_appid}"),
    ))
}
