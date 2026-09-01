/*
    Automatic start-up update checks for games managed by Monarch.

    Games whose files were installed by Monarch itself can currently only be
    installed or updated through monarch_egs, so update availability is
    determined by comparing the locally recorded build version against Epic's
    Live assets list.
*/

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::thread;
use tracing::{error, info, warn};

use monarch_egs::{GameUpdate, InstalledBuild};

use crate::monarch_games::egs_client::EgsClient;
use crate::monarch_games::monarchgame::MonarchGame;
use crate::monarch_games::stores::DownloadOptions;
use crate::monarch_utils::monarch_game_downloader::DownloadJob;
use crate::monarch_utils::monarch_settings::get_settings;
use crate::monarch_utils::monarch_state::MONARCH_STATE;

/// Platform used when installing games through monarch_egs. Managed installs
/// always use Windows builds, even on Linux/macOS (via umu/proton).
static MANAGED_PLATFORM: &str = "Windows";

/// An available update for a game managed by Monarch, paired with the library
/// game it belongs to so the UI can present it without extra lookups.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonarchGameUpdate {
    pub game_name: String,
    pub game_id: String,
    pub update: GameUpdate,
}

/// The outcome of checking a single game for updates, as reported by
/// check_game_for_updates().
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameUpdateCheck {
    /// The installed build matches Epic's current Live build.
    UpToDate,
    /// A newer build was found and queued for download.
    UpdateAvailable { latest_build_version: String },
}

/// An installed game managed by Monarch together with the metadata required to
/// check it for updates.
struct ManagedInstall {
    game: MonarchGame,
    build: InstalledBuild,
}

/// Spawns the start-up update check on a background thread with its own tokio
/// runtime, mirroring housekeeping::start(), so start-up is never blocked on
/// network requests.
pub fn start_startup_check() {
    thread::spawn(|| {
        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(e) => {
                error!(
                    "monarch_games::updates::start_startup_check() Failed to create tokio runtime! | Err: {e}"
                );
                return;
            }
        };

        // The Epic session handling in monarch_egs panics on some network
        // failures. Catch it here so a failed check can never take Monarch
        // down during start-up.
        let check =
            futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(run_startup_check()));

        if let Err(panic) = runtime.block_on(check) {
            error!(
                "monarch_games::updates::start_startup_check() Update check panicked! | Err: {panic:?}"
            );
        }
    });
}

/// Runs the start-up update check unless the user disabled it in settings.
pub async fn run_startup_check() {
    if !auto_update_check_enabled() {
        info!(
            "monarch_games::updates::run_startup_check() Skipping start-up update check, disabled in settings"
        );
        return;
    }

    match check_for_game_updates().await {
        Ok(updates) if updates.is_empty() => {
            info!(
                "monarch_games::updates::run_startup_check() All games managed by Monarch are up to date"
            );
        }
        Ok(updates) => {
            for game_update in &updates {
                info!(
                    "monarch_games::updates::run_startup_check() Update available for {} ({} -> {})",
                    game_update.game_name,
                    game_update.update.installed_build_version,
                    game_update.update.latest_build_version
                );
            }
        }
        Err(e) => {
            warn!(
                "monarch_games::updates::run_startup_check() Start-up update check failed | Err: {e}"
            );
        }
    }
}

/// Whether automatic start-up update checks are enabled in settings.
fn auto_update_check_enabled() -> bool {
    match get_settings() {
        Ok(settings_lock) => settings_lock
            .read()
            .map(|settings| settings.monarch.check_updates_on_startup)
            .unwrap_or_else(|e| {
                error!(
                    "monarch_games::updates::auto_update_check_enabled() Failed to lock on settings | Err: {e}"
                );
                false
            }),
        Err(e) => {
            error!(
                "monarch_games::updates::auto_update_check_enabled() Failed to get settings | Err: {e}"
            );
            false
        }
    }
}

/// Checks all installed games managed by Monarch for available updates and
/// stores the result in MONARCH_STATE. Can also be triggered manually, e.g.
/// from a "check for updates" button.
pub async fn check_for_game_updates() -> Result<Vec<MonarchGameUpdate>, String> {
    let games: Vec<MonarchGame> = match MONARCH_STATE.read() {
        Ok(state) => state.get_library_games(),
        Err(e) => {
            error!(
                "monarch_games::updates::check_for_game_updates() Failed to lock on MONARCH_STATE | Err: {e}"
            );
            return Err(String::from("Failed to read app state!"));
        }
    };

    let installs = collect_managed_installs(&games);

    if installs.is_empty() {
        store_available_updates(Vec::new());
        return Ok(Vec::new());
    }

    let mut client = EgsClient::new();
    if !client.credentials_exist() {
        info!(
            "monarch_games::updates::check_for_game_updates() No Epic Games credentials found, skipping update check"
        );
        store_available_updates(Vec::new());
        return Ok(Vec::new());
    }

    client
        .load_existing_user()
        .await
        .map_err(|e| format!("Failed to load Epic Games session | Err: {e}"))?;

    let builds: Vec<InstalledBuild> = installs
        .iter()
        .map(|install| install.build.clone())
        .collect();
    let updates = monarch_egs::check_updates(client.user(), MANAGED_PLATFORM, &builds)
        .await
        .map_err(|e| format!("Epic Games update check failed | Err: {e}"))?;

    let game_updates: Vec<MonarchGameUpdate> = updates
        .into_iter()
        .map(|update| {
            let install = installs.iter().find(|install| {
                install.build.namespace == update.namespace
                    && install.build.app_name == update.app_name
            });
            MonarchGameUpdate {
                game_name: install
                    .map(|install| install.game.name.clone())
                    .unwrap_or_else(|| update.app_name.clone()),
                game_id: install
                    .map(|install| install.game.id.clone())
                    .unwrap_or_default(),
                update,
            }
        })
        .collect();

    store_available_updates(game_updates.clone());

    // Queue the detected updates so they are ready to download from the
    // download page, without starting them automatically.
    if !game_updates.is_empty() {
        queue_detected_updates(&game_updates, &installs).await;
    }

    Ok(game_updates)
}

/// Checks a single installed game managed by Monarch for updates. Same
/// comparison as the start-up check but scoped to one game; a detected update
/// is queued for download without starting it, and the stored update-check
/// results for other games are left untouched.
pub async fn check_game_for_updates(game: &MonarchGame) -> Result<GameUpdateCheck, String> {
    let Some(install) = collect_managed_install(game) else {
        return Err(format!(
            "{} cannot be checked because it was not installed by Monarch or its install metadata is incomplete.",
            game.name
        ));
    };

    let mut client = EgsClient::new();
    if !client.credentials_exist() {
        return Err(String::from(
            "Sign in to the Epic Games Store to check this game for updates.",
        ));
    }

    client
        .load_existing_user()
        .await
        .map_err(|e| format!("Failed to load Epic Games session | Err: {e}"))?;

    let builds = vec![install.build.clone()];
    let updates = monarch_egs::check_updates(client.user(), MANAGED_PLATFORM, &builds)
        .await
        .map_err(|e| format!("Epic Games update check failed | Err: {e}"))?;

    let Some(update) = updates.into_iter().next() else {
        // Up to date: drop any stale queued entry recorded for this game.
        info!(
            "monarch_games::updates::check_game_for_updates() {} is up to date",
            game.name
        );
        store_game_update_result(&game.id, None);
        return Ok(GameUpdateCheck::UpToDate);
    };

    info!(
        "monarch_games::updates::check_game_for_updates() Update available for {} ({} -> {}), queuing",
        game.name,
        update.installed_build_version,
        update.latest_build_version
    );

    let latest_build_version = update.latest_build_version.clone();
    let game_update = MonarchGameUpdate {
        game_name: game.name.clone(),
        game_id: game.id.clone(),
        update,
    };

    queue_detected_updates(std::slice::from_ref(&game_update), std::slice::from_ref(&install))
        .await;
    store_game_update_result(&game.id, Some(game_update));

    Ok(GameUpdateCheck::UpdateAvailable { latest_build_version })
}

/// Queues download jobs for the given updates without starting them. Jobs land
/// at the back of the downloader queue where the user can start them from the
/// download page. Games that are already queued or downloading are skipped.
async fn queue_detected_updates(updates: &[MonarchGameUpdate], installs: &[ManagedInstall]) {
    let default_folder: String = match get_settings() {
        Ok(settings_lock) => settings_lock
            .read()
            .map(|settings| settings.monarch.game_folder.clone())
            .unwrap_or_else(|e| {
                error!(
                    "monarch_games::updates::queue_detected_updates() Failed to lock on settings | Err: {e}"
                );
                String::new()
            }),
        Err(e) => {
            error!(
                "monarch_games::updates::queue_detected_updates() Failed to get settings | Err: {e}"
            );
            String::new()
        }
    };

    let client = EgsClient::new();
    let mut prepared: Vec<(MonarchGame, DownloadJob)> = Vec::new();

    for update in updates {
        let Some(install) = installs.iter().find(|install| {
            install.game.id == update.game_id && install.build.namespace == update.update.namespace
        }) else {
            continue;
        };
        let game = install.game.clone();

        // Update in place: the download handler writes into <folder>/<game>,
        // so reuse the parent of the recorded install directory.
        let opts = DownloadOptions {
            folder: install_parent_folder(&game, &default_folder),
            store: String::from("epicgames"),
            game_name: game.name.clone(),
            game_store: String::from("epicgames"),
            game_store_id: update.update.namespace.clone(),
            os: std::env::consts::OS.to_string(),
            compatibility: game.compatibility.clone(),
        };

        match client.prepare_download_job(&game, &opts).await {
            Ok(job) => prepared.push((game, job)),
            Err(e) => {
                warn!(
                    "monarch_games::updates::queue_detected_updates() Failed to prepare update for {} | Err: {}",
                    game.name,
                    e.chain().map(|e| e.to_string()).collect::<String>()
                );
            }
        }
    }

    if prepared.is_empty() {
        return;
    }

    match MONARCH_STATE.read() {
        Ok(state) => match state.get_downloader_ptr().write() {
            Ok(mut downloader) => {
                if let Err(e) = downloader.register_egs_handler() {
                    warn!(
                        "monarch_games::updates::queue_detected_updates() Failed to register EGS download handler | Err: {e}"
                    );
                }

                for (game, job) in prepared {
                    if downloader.is_queued(&game) || downloader.is_downloading_game(&game) {
                        info!(
                            "monarch_games::updates::queue_detected_updates() {} already queued or downloading, skipping",
                            game.name
                        );
                        continue;
                    }

                    info!(
                        "monarch_games::updates::queue_detected_updates() Queuing update for {}",
                        game.name
                    );
                    downloader.queue_download(job);
                }
            }
            Err(e) => {
                error!(
                    "monarch_games::updates::queue_detected_updates() Failed to lock on downloader | Err: {e}"
                );
            }
        },
        Err(e) => {
            error!(
                "monarch_games::updates::queue_detected_updates() Failed to lock on MONARCH_STATE | Err: {e}"
            );
        }
    }
}

/// Returns the folder an update should be downloaded to. The download handler
/// appends the game name to this folder, so this is the parent of the game's
/// existing install directory; falls back to Monarch's default game folder.
fn install_parent_folder(game: &MonarchGame, default_folder: &str) -> String {
    let install_dir = &game.properties.install_dir;
    if !install_dir.is_empty() && install_dir != "Error" {
        let parent = PathBuf::from(install_dir)
            .parent()
            .map(|parent| parent.to_path_buf())
            .unwrap_or_default();

        if parent.as_os_str().is_empty() {
            return default_folder.to_string();
        }
        return parent.to_string_lossy().to_string();
    }
    default_folder.to_string()
}

/// Persists the latest update check results so the UI can pick them up.
fn store_available_updates(updates: Vec<MonarchGameUpdate>) {
    match MONARCH_STATE.write() {
        Ok(mut state) => state.set_available_updates(updates),
        Err(e) => {
            error!(
                "monarch_games::updates::store_available_updates() Failed to lock on MONARCH_STATE | Err: {e}"
            );
        }
    }
}

/// Persists the result of a single-game update check, replacing any previous
/// entry for that game while leaving other games' results untouched.
fn store_game_update_result(game_id: &str, update: Option<MonarchGameUpdate>) {
    match MONARCH_STATE.write() {
        Ok(mut state) => {
            let mut updates: Vec<MonarchGameUpdate> = state
                .get_available_updates()
                .into_iter()
                .filter(|existing| existing.game_id != game_id)
                .collect();
            if let Some(update) = update {
                updates.push(update);
            }
            state.set_available_updates(updates);
        }
        Err(e) => {
            error!(
                "monarch_games::updates::store_game_update_result() Failed to lock on MONARCH_STATE | Err: {e}"
            );
        }
    }
}

/// Collects library games whose files were installed by Monarch itself and
/// therefore can only be updated through monarch_egs. Games missing the
/// metadata required for a comparison are skipped.
fn collect_managed_installs(games: &[MonarchGame]) -> Vec<ManagedInstall> {
    games.iter().filter_map(collect_managed_install).collect()
}

/// Returns the metadata required to check `game` for updates, or `None` when
/// it is not an installed game managed through monarch_egs or its install
/// metadata is incomplete.
fn collect_managed_install(game: &MonarchGame) -> Option<ManagedInstall> {
    if !game.is_installed || !game.managed_by_monarch {
        return None;
    }

    let store = game.stores.iter().find(|store| store.name == "epicgames")?;

    let catalog_id = game.properties.other.get("catalog_id");
    let app_name = game.properties.other.get("app_name");
    let version = &game.properties.version;

    // Games installed before Monarch tracked EGS asset data, or installs
    // without a recorded build version, cannot be reliably compared.
    let Some(catalog_id) = catalog_id else {
        warn!(
            "monarch_games::updates::collect_managed_install() Missing EGS catalog id for {}",
            game.name
        );
        return None;
    };
    let Some(app_name) = app_name else {
        warn!(
            "monarch_games::updates::collect_managed_install() Missing EGS app name for {}",
            game.name
        );
        return None;
    };

    if store.store_id.is_empty()
        || catalog_id.is_empty()
        || app_name.is_empty()
        || version.is_empty()
        || version == "Error"
    {
        warn!(
            "monarch_games::updates::collect_managed_install() Incomplete install metadata for {}",
            game.name
        );
        return None;
    }

    Some(ManagedInstall {
        game: game.clone(),
        build: InstalledBuild {
            namespace: store.store_id.clone(),
            catalog_item_id: catalog_id.clone(),
            app_name: app_name.clone(),
            build_version: version.clone(),
        },
    })
}
