use super::monarch_client;
use super::monarchgame::MonarchGame;
use anyhow::Result;
use rand::rng;
use rand::seq::SliceRandom;
use std::path::PathBuf;
use tracing::{error, info, warn};

use super::monarch_client::MonarchClient;
use super::steam_client::SteamClient;
use super::stores::{SearchFilter, StoreType};
use crate::monarch_games::games::{GameType, SearchResult};
use crate::monarch_games::legendary_client::{self, LegendaryClient};
use crate::monarch_games::monarchgame::MonarchWebApiGame;
use crate::monarch_games::steam_client;
use crate::monarch_games::stores::DownloadOptions;
use crate::monarch_library::{self, games_library};
use crate::monarch_utils::monarch_fs;
use crate::monarch_utils::monarch_fs::path_exists;
use crate::monarch_utils::monarch_state::MONARCH_STATE;
use crate::monarch_utils::monarch_vdf::{get_proton_versions, ProtonVersion};

#[cfg(target_os = "windows")]
use super::windows::steam;

#[cfg(target_os = "macos")]
use super::macos::steam;

#[cfg(target_os = "linux")]
use super::linux::steam;
#[cfg(target_os = "linux")]
use super::linux::umu;


/*
---------- General game related functions ----------
*/

/// Returns MonarchGames from library.json
pub fn get_library() -> Result<Vec<MonarchGame>, String> {
    match games_library::get_games() {
        Ok(games) => Ok(games),
        Err(e) => {
            error!(
                "monarch_games::commands::get_library -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            Err(String::from("Something went wrong getting library!"))
        }
    }
}

pub fn get_home_recomendations() -> Result<Vec<MonarchGame>, String> {
    match get_library() {
        Ok(mut games) => {
            if games.len() > 4 {
                games.shuffle(&mut rng());
                Ok(games[0..4].to_vec())
            } else {
                return Ok(games);
            }
        }
        Err(e) => {
            error!("monarch_games::commands::get_home_recomendations() Failed to get recomendations! | Err: {e}");
            Err(String::from("Something went wrong getting library!"))
        }
    }
}

/// Search for games on Monarch, currently only support Steam search
pub async fn search_games(name: String, filter: SearchFilter) -> Vec<MonarchWebApiGame> {
    if filter.monarch {
        let client = MonarchClient::new();
        return client
            .search_games(&name, &filter)
            .await
            .into_iter()
            .map(|g| g.to_search_result())
            .collect();
    }

    let mut games: Vec<MonarchWebApiGame> = Vec::new();

    if filter.steam_powered {
        let client = SteamClient::new();
        games.append(
            &mut client
                .search_games(&name, &filter)
                .await
                .into_iter()
                .map(|g| g.to_search_result())
                .collect(),
        );
    }

    if filter.egs {
        let client = LegendaryClient::new();
        games.append(
            &mut client
                .search_games(&name, &filter)
                .await
                .into_iter()
                .map(|g| g.to_search_result())
                .collect(),
        );
    }

    games
}

/// Manually refreshes the entire Monarch library, currently only supports Steam & Epic Games (kinda) still WIP
pub async fn refresh_library() -> Vec<MonarchGame> {
    monarch_client::refresh_library().await
}

/// Tell backend to download cover/thumbnail for game.
pub async fn download_thumbnail(game: &MonarchGame) -> Result<(), String> {
    if let Err(e) = game.download_thumbnail().await {
        error!(
            "monarch_games::commands::download_thumbnail() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from("Failed to download thumbnail"));
    }

    // Make sure image has been saved
    let path = PathBuf::from(&game.thumbnail_path);
    if !path_exists(&path) {
        warn!(
            "Cover reported finished downloading, not found: {}",
            game.thumbnail_path
        );

        for _ in 0..3 {
            std::thread::sleep(std::time::Duration::from_secs(1));
            if path_exists(&path) {
                break;
            }
        }

        error!(
            "monarch_games::commands::download_thumbnail() Could not find: {}",
            game.thumbnail_path
        );
    }

    Ok(())
}

/// Tell backend to download cover/thumbnail for game.
pub async fn download_artwork(game: &MonarchGame) -> Result<(), String> {
    if let Err(e) = game.download_artwork().await {
        error!(
            "monarch_games::commands::download_artwork() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from("Failed to download artwork"));
    }
    Ok(())
}

pub async fn search_page_download_thumbnail(game: MonarchWebApiGame) -> Result<(), String> {
    download_thumbnail(&game.into_monarchgame()).await
}

/// Launch a game
pub async fn launch_game(game: &MonarchGame) -> Result<(), String> {
    info!("Launching game: {}", game.name);
    if let Err(e) = game.launch().await {
        error!(
            "monarch_games::commands::launch_game() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(format!(
            "Something went wrong while launching: {}",
            game.name
        ));
    }
    Ok(())
}

/// Tells Monarch to download specified game
pub async fn download_game(opts: DownloadOptions) -> Result<Vec<MonarchGame>, String> {
    // For best user experience Monarch downloads all games by itself
    // instead of having to rely on 3rd party launchers.
    info!("Installing: {}", opts.game_name);

    let game = MonarchGame::new(
        &opts.game_name,
        0,
        &opts.game_store,
        &opts.game_store_id,
        "",
        "",
        "",
    );

    let result: Result<(), String> = match opts.game_store.as_str() {
        "steam" => {
            if let Err(e) = SteamClient::new().install_game(&game, &opts).await {
                return Err(e.to_string());
            }
            Ok(())
        }
        "epicgames" => {
            if let Err(e) = LegendaryClient::new().install_game(&game, &opts).await {
                return Err(e.to_string());
            }
            Ok(())
        }
        _ => Err(String::from("Unsupported store")),
    };

    if let Err(e) = result {
        return Err(e);
    }

    match MONARCH_STATE.read() {
        Ok(state) => {
            return Ok(state.get_library_games());
        }
        Err(e) => {
            error!(
                "monarch_client::launch_game() Failed to lock on MONARCH_STATE | Err: {}",
                e
            );
            return Err(String::from("Failed to get updated library."));
        }
    }
}

/// Tells Monarch to download specified game
pub async fn update_game(name: String, store: String, store_id: String) -> Result<(), String> {
    info!("Updating: {name}");
    match monarch_client::update_game(&store, &store_id).await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!(
                "monarch_games::commands::check_for_game_update() -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            Err(format!("Something went wrong while updating: {name} \nMake sure game is installed via Monarch if you want to update."))
        }
    }
}

/// Tells Monarch to remove specified game
pub async fn remove_game(name: String, store: String, store_id: String) -> Result<(), String> {
    info!("Uninstalling: {name}");
    if let Err(e) = monarch_client::uninstall_game(&store, &store_id).await {
        error!(
            "monarch_games::commands::remove_game() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(format!("Something went wrong while removing: {name}"));
    }
    Ok(())
}

pub async fn move_game_to_monarch(
    name: String,
    store: String,
    store_id: String,
) -> Result<(), String> {
    info!("Moving {name} from {store} to Monarch...");

    // First remove the game from old store
    if let Err(e) = monarch_client::uninstall_game(&store, &store_id).await {
        error!(
            "monarch_games::commands::move_game_to_monarch() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(format!("Something went wrong while removing: {name}"));
    }

    // Then reinstall on Monarch
    if let Err(e) = monarch_client::download_game(&name, &store, &store_id).await {
        error!(
            "monarch_games::commands::move_game_to_monarch() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(format!("Something went wrong while downloading: {name}"));
    }

    info!("Finished moving {name} to Monarch");
    Ok(())
}

/*
/// Open "Purchase window" for a game
pub async fn open_store(url: String) -> Result<(), String> {
    let window: MiniWindow = MiniWindow::new("store", &url, 1280.0, 720.0);
    if let Err(e) = window.build_window(&handle).await {
        error!(
            "monarch_games::commands::open_store() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from(
            "Something went wrong while opening store page!",
        ));
    }

    if let Err(e) = window.show_window(&handle) {
        error!(
            "monarch_games::commands::open_store() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from(
            "Something went wrong while opening store page!",
        ));
    }
    Ok(())
}
 */

/// Updates the properties of a game in the library.
pub async fn update_game_properties(game: &MonarchGame) -> Result<(), String> {
    info!("Updating properties for: {}", game.name);
    match games_library::update_game_properties(game) {
        Ok(_) => Ok(()),
        Err(e) => {
            error!(
                "monarch_games::commands::update_game_properties() -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            Err(String::from(
                "Something went wrong while updating game properties!",
            ))
        }
    }
}

pub fn proton_versions() -> Result<Vec<ProtonVersion>, String> {
    #[cfg(not(target_os = "linux"))]
    return Ok(vec![]);

    // Get libraryfolders.vdf
    let library_path = match steam::get_default_libraryfolders_location() {
        Ok(p) => p,
        Err(e) => {
            error!(
                "monarch_games::commands::proton_versions() -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            return Err(String::from(
                "Something went wrong while getting proton versions!",
            ));
        }
    };

    // Then get proton versions
    match get_proton_versions(&library_path) {
        Ok(p) => Ok(p),
        Err(e) => {
            error!(
                "monarch_games::commands::proton_versions() -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            Err(String::from(
                "Something went wrong while getting proton versions!",
            ))
        }
    }
}

pub async fn manual_add_game(mut game: MonarchGame) -> Result<(), String> {
    info!("User adding game binary: {:?}", game);

    game.manually_generate_id();

    if monarch_fs::is_in_cache_dir(&PathBuf::from(&(game.thumbnail_path))) {
        info!("Found thumbnail in cache, copying to library");

        match monarch_fs::copy_cache_to_library(&PathBuf::from(&(game.thumbnail_path))) {
            Ok(path) => {
                info!("Copied thumbnail to library: {}", path.display());
                game.thumbnail_path = path.to_str().unwrap().to_string();
            }
            Err(e) => {
                error!(
                    "monarch_games::commands::manual_add_game() -> {}",
                    e.chain().map(|e| e.to_string()).collect::<String>()
                );
            }
        }
    }

    if let Err(e) = monarch_library::games_library::add_game(&game) {
        error!(
            "monarch_games::commands::manual_add_game() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(format!("Failed to add game: {}", game.name));
    }

    return Ok(());
}

pub fn get_executables(game: &mut MonarchGame) -> Result<Vec<PathBuf>, String> {
    if game.properties.install_dir.is_empty() {
        #[cfg(target_os = "linux")]
        use super::linux::steam;

        #[cfg(target_os = "windows")]
        use super::windows::steam;

        #[cfg(target_os = "macos")]
        use super::macos::steam;

        // Get the installation folder of game from libraryfolders.vdf
        match steam::get_default_libraryfolders_location() {
            Ok(path) => {
                use crate::monarch_utils::monarch_vdf;

                if let Err(e) = monarch_vdf::set_install_dir(game, &path) {
                    error!(
                        "monarch_games::commands::get_executables() -> {}",
                        e.chain().map(|e| e.to_string()).collect::<String>()
                    );
                    return Err(format!(
                        "Set the correct installation directory for: {}",
                        game.name
                    ));
                }
            }
            Err(e) => {
                error!(
                    "monarch_games::commands::get_executables() -> {}",
                    e.chain().map(|e| e.to_string()).collect::<String>()
                );
                return Err(format!("Failed to get executables for game: {}", game.name));
            }
        }
    }

    // Search for executable files in the installation directory
    match monarch_fs::get_executables(&PathBuf::from(&game.properties.install_dir)) {
        Ok(exes) => Ok(exes),
        Err(e) => {
            error!(
                "monarch_games::commands::get_executables() -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            Err(format!("Failed to get executables for game: {}", game.name))
        }
    }
}

pub async fn get_game_properties(game: &mut MonarchGame) {
    monarch_client::get_game_properties(game).await
}

pub async fn manual_remove_game(game: MonarchGame) -> Result<(), String> {
    info!("User removing game binary: {:?}", game);

    if let Err(e) = monarch_library::games_library::remove_game(&game) {
        error!(
            "monarch_games::commands::manual_remove_game() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(format!(
            "Failed to remove game: {} from library!",
            game.name
        ));
    }

    return Ok(());
}

pub fn umu_is_installed() -> bool {
    #[cfg(target_os = "linux")]
    {
        use super::linux::umu;
        return umu::umu_is_installed();
    }

    #[cfg(not(target_os = "linux"))]
    false
}

pub fn install_umu() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        use super::linux::umu;
        info!("Downloading umu-launcher...");

        if let Err(e) = umu::install_umu() {
            error!(
                "monarch_games::commands::install_umu() -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            return Err(format!("Failed to download umu-launcher!"));
        }

        return Ok(());
    }

    #[cfg(not(target_os = "linux"))]
    {
        use tracing::warn;
        warn!("Attempted to download umu-launcher under something other than Linux!");
        return Err(format!("Can only use umu-launcher under Linux!"));
    }
}

pub fn steamcmd_is_installed() -> bool {
    use super::steam_client;
    steam_client::steamcmd_is_installed()
}

pub async fn install_steamcmd() -> Result<(), String> {
    use super::steam_client;
    if let Err(e) = steam_client::install_steamcmd().await {
        error!(
            "monarch_games::commands::install_steamcmd() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from("Failed to download SteamCMD!"));
    }
    Ok(())
}

pub fn legendary_is_installed() -> bool {
    legendary_client::legendary_is_installed()
}

pub fn install_legendary() -> Result<(), String> {
    if let Err(e) = legendary_client::install_legendary() {
        error!(
            "monarch_games::commands::install_legendary() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from("Failed to download Legendary!"));
    }

    let client: LegendaryClient = LegendaryClient::new();
    if let Err(e) = client.login() {
        error!(
            "monarch_games::commands::install_legendary() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from("Failed to login to Legendary!"));
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn remove_umu() -> Result<(), String> {
    if let Err(e) = umu::remove_umu() {
        error!(
            "monarch_games::commands::remove_umu() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from("Failed to remove Umu launcher!"));
    }
    Ok(())
}

pub fn remove_steamcmd() -> Result<(), String> {
    if let Err(e) = steam_client::remove_steamcmd() {
        error!(
            "monarch_games::commands::remove_steamcmd() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from("Failed to remove SteamCMD!"));
    }
    Ok(())
}

pub fn remvoe_legendary() -> Result<(), String> {
    if let Err(e) = legendary_client::remvoe_legendary() {
        error!(
            "monarch_games::commands::remove_steamcmd() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from("Failed to remove Legendary!"));
    }
    Ok(())
}