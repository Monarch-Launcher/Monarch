use super::monarch_client;
use super::monarchgame::MonarchGame;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};

use super::monarch_client::MonarchClient;
use super::steam_client::SteamClient;
use super::stores::{SearchFilter, StoreType};
use crate::monarch_games::egs_client::EgsClient;
use crate::monarch_games::games::GameType;
use crate::monarch_games::monarchgame::MonarchWebApiGame;
use crate::monarch_games::steam_client;
use crate::monarch_games::stores::DownloadOptions;
use crate::monarch_library::{self, library};
use crate::monarch_utils::monarch_fs;
use crate::monarch_utils::monarch_fs::path_exists;
use crate::monarch_utils::monarch_vdf::ProtonVersion;
use monarch_egs::SupportedPlatforms;

#[cfg(target_os = "linux")]
use super::linux::steam;
#[cfg(target_os = "linux")]
use super::linux::umu;
#[cfg(target_os = "macos")]
use super::macos::steam;
#[cfg(target_os = "linux")]
use crate::monarch_utils::monarch_vdf::get_proton_versions;

/*
---------- General game related functions ----------
*/

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
        let client = EgsClient::new();
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
pub async fn refresh_library() -> Result<Vec<MonarchGame>, String> {
    match monarch_client::refresh_library().await {
        Ok(games) => Ok(games),
        Err(e) => {
            error!(
                "monarch_games::commands::refresh_library() -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            Err(String::from("Failed to refresh library!"))
        }
    }
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

/// Tell backend to generate a greyscale version of the game's thumbnail.
/// Returns early if the greyscale image already exists on disk.
pub async fn download_greyscale(game: &MonarchGame) -> Result<(), String> {
    if let Err(e) = game.download_greyscale().await {
        error!(
            "monarch_games::commands::download_greyscale() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(String::from("Failed to download greyscale thumbnail"));
    }
    Ok(())
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
pub async fn download_game(
    game: &mut MonarchGame,
    opts: &mut DownloadOptions,
) -> Result<Vec<MonarchGame>, String> {
    // For best user experience Monarch downloads all games by itself
    // instead of having to rely on 3rd party launchers.
    info!("Installing: {}", opts.game_name);
    debug!("Using options: {:?}", opts);

    if opts.folder.is_empty() {
        match MONARCH_STATE.read() {
            Ok(state) => match state.get_settings_ptr().read() {
                Ok(settings) => {
                    opts.folder = settings.monarch.game_folder.clone();
                }
                Err(e) => {
                    error!("monarch_games::commands::download_game() Failed to lock on settings | Err: {e}");
                    return Err(String::from("Failed to read settings"));
                }
            },
            Err(e) => {
                error!("monarch_games::commands::download_game() Failed to lock on MONARCH_STATE | Err: {e}");
                return Err(String::from("Failed to read app state!"));
            }
        }
    }

    let result: Result<(), String> = match opts.game_store.as_str() {
        "steam" => {
            if let Err(e) = SteamClient::new().install_game(game, &opts).await {
                return Err(e.to_string());
            }
            Ok(())
        }
        "epicgames" => {
            let mut client = EgsClient::new();
            client.load_existing_user().await.unwrap();
            client.install_game(game, opts).await.unwrap();
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
                "monarch_client::download_game() Failed to lock on MONARCH_STATE | Err: {}",
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

/// Returns the updates found by the latest update check for games managed by Monarch.
pub fn get_available_game_updates() -> Vec<super::updates::MonarchGameUpdate> {
    match MONARCH_STATE.read() {
        Ok(state) => state.get_available_updates(),
        Err(e) => {
            error!(
                "monarch_games::commands::get_available_game_updates() Failed to lock on MONARCH_STATE | Err: {e}"
            );
            Vec::new()
        }
    }
}

/// Checks all installed games managed by Monarch for available updates.
///
/// This is the same check that runs automatically on start-up: results are
/// stored in app state and any detected updates are queued for download.
pub async fn check_for_game_updates() -> Result<Vec<super::updates::MonarchGameUpdate>, String> {
    // The Epic session handling in monarch_egs panics on some network
    // failures. Catch it here so a failed manual check can never take
    // Monarch down, mirroring the start-up update check.
    let check = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(
        super::updates::check_for_game_updates(),
    ));

    match check.await {
        Ok(result) => result,
        Err(panic) => {
            error!(
                "monarch_games::commands::check_for_game_updates() Update check panicked! | Err: {panic:?}"
            );
            Err(String::from(
                "Something went wrong while checking for updates!",
            ))
        }
    }
}

/// Checks a single game managed by Monarch for updates and reports whether it
/// is up to date or an update was queued. Triggered from the
/// "Check for Updates" action in the actions modal.
pub async fn check_game_for_updates(
    game: &MonarchGame,
) -> Result<super::updates::GameUpdateCheck, String> {
    // Same panic hardening as the library-wide check.
    let check = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(
        super::updates::check_game_for_updates(game),
    ));

    match check.await {
        Ok(result) => result,
        Err(panic) => {
            error!(
                "monarch_games::commands::check_game_for_updates() Update check panicked! | Err: {panic:?}"
            );
            Err(format!(
                "Something went wrong while checking {} for updates!",
                game.name
            ))
        }
    }
}

/// Verifies the files of an installed game managed by Monarch and returns a
/// human-readable summary of the result. Triggered from the
/// "Verify Integrity of Files" action. `on_progress` is invoked with
/// (files checked, total files) whenever the whole-percent progress changes.
pub async fn verify_game_integrity(
    game: &MonarchGame,
    on_progress: Option<super::integrity::ProgressCallback>,
) -> Result<String, String> {
    info!("Verifying integrity of: {}", game.name);

    // Manifest fetching in monarch_egs unwraps on some network failures;
    // catch panics so a failed verification can never take Monarch down.
    let verify = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(
        super::integrity::verify_game_integrity(game, on_progress),
    ));

    match verify.await {
        Ok(result) => result,
        Err(panic) => {
            error!(
                "monarch_games::commands::verify_game_integrity() Verification panicked! | Err: {panic:?}"
            );
            Err(String::from(
                "Something went wrong while verifying game files!",
            ))
        }
    }
}

/// Number of downloads either in progress or waiting in the queue.
pub fn get_pending_download_count() -> usize {
    match MONARCH_STATE.read() {
        Ok(state) => match state.get_downloader_ptr().read() {
            Ok(downloader) => downloader.pending_job_count(),
            Err(e) => {
                error!(
                    "monarch_games::commands::get_pending_download_count() Failed to lock on downloader | Err: {e}"
                );
                0
            }
        },
        Err(e) => {
            error!(
                "monarch_games::commands::get_pending_download_count() Failed to lock on MONARCH_STATE | Err: {e}"
            );
            0
        }
    }
}

/// Tells Monarch to remove specified game
pub async fn remove_game(name: String, store: String, store_id: String) -> Result<(), String> {
    info!("Uninstalling: {name}");

    // Resolve the full game record so we can tell whether Monarch manages its files.
    let game: Option<MonarchGame> = match MONARCH_STATE.read() {
        Ok(state) => state
            .get_library_games()
            .into_iter()
            .find(|g| g.get_store_name() == store && g.get_store_id() == store_id),
        Err(e) => {
            error!(
                "monarch_games::commands::remove_game() Failed to lock on MONARCH_STATE | Err: {e}"
            );
            return Err(format!("Something went wrong while removing: {name}"));
        }
    };

    // Only remove files that Monarch itself downloaded.
    if let Some(game) = &game {
        if game.managed_by_monarch {
            if let Err(e) = remove_install_dir(game) {
                error!(
                    "monarch_games::commands::remove_game() Failed to remove install folder for {name} | Err: {e}"
                );
                return Err(format!(
                    "Something went wrong while removing: {name} \nCould not remove the install folder."
                ));
            }
        }
    }

    match store.as_str() {
        "steam" | "steamcmd" => {
            if let Err(e) = monarch_client::uninstall_game(&store, &store_id).await {
                error!(
                    "monarch_games::commands::remove_game() -> {}",
                    e.chain().map(|e| e.to_string()).collect::<String>()
                );
                return Err(format!("Something went wrong while removing: {name}"));
            }
        }
        "epicgames" => {
            let game = match game {
                Some(game) => game,
                None => {
                    error!(
                        "monarch_games::commands::remove_game() Game not found in library: {name}"
                    );
                    return Err(format!("Something went wrong while removing: {name}"));
                }
            };

            if let Err(e) = library::mark_game_uninstalled(&game).await {
                error!(
                    "monarch_games::commands::remove_game() -> {}",
                    e.chain().map(|e| e.to_string()).collect::<String>()
                );
                return Err(format!("Something went wrong while removing: {name}"));
            }
        }
        _ => {
            error!("monarch_games::commands::remove_game() Unsupported store: {store}");
            return Err(format!("Something went wrong while removing: {name}"));
        }
    }

    Ok(())
}

/// Removes the install directory of a game that Monarch itself downloaded.
fn remove_install_dir(game: &MonarchGame) -> Result<()> {
    info!(
        "monarch_games::commands::remove_install_dir() Removing install folder: {}",
        game.properties.install_dir
    );
    monarch_fs::remove_dir(&Path::new(&game.properties.install_dir))
        .with_context(|| "monarch_games::commands::remove_install_dir() -> ")
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

/// Updates the properties of a game in the library.
pub async fn update_game_properties(game: &MonarchGame) -> Result<(), String> {
    info!("Updating properties for: {}", game.name);
    match library::update_game_properties(game).await {
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

    #[cfg(target_os = "linux")]
    {
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
}

/// Checks which platforms are supported for a game from Epic Games Store.
/// Returns SupportedPlatforms indicating Windows/Linux/Mac support.
pub async fn check_egs_platform_support(game: &MonarchGame) -> Result<SupportedPlatforms, String> {
    let namespace = game
        .stores
        .iter()
        .find(|s| s.name == "epicgames")
        .map(|s| s.store_id.clone())
        .unwrap_or_default();

    if namespace.is_empty() {
        error!(
            "monarch_games::commands::check_egs_platform_support() Missing Epic Games namespace for {}",
            game.name
        );
        return Err("Missing Epic Games namespace".to_string());
    }

    let mut client = EgsClient::new();
    if let Err(e) = client.load_existing_user().await {
        error!(
            "monarch_games::commands::check_egs_platform_support() Failed to load EGS user | Err: {}",
            e
        );
        return Err("Failed to load Epic Games user".to_string());
    }

    match client.check_platform_support(&namespace).await {
        Ok(support) => Ok(support),
        Err(e) => {
            error!(
                "monarch_games::commands::check_egs_platform_support() -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            Err("Failed to check platform support".to_string())
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

    if let Err(e) = monarch_library::library::add_game(&game).await {
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
    info!("User marking game as uninstalled: {:?}", game);

    if let Err(e) = monarch_library::library::mark_game_uninstalled(&game).await {
        error!(
            "monarch_games::commands::manual_remove_game() -> {}",
            e.chain().map(|e| e.to_string()).collect::<String>()
        );
        return Err(format!(
            "Failed to mark game as uninstalled: {} from library!",
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
