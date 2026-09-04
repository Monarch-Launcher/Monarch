/*
    Integrity verification for games installed and managed by Monarch.

    Games whose files were installed by Monarch can be checked against their
    store's manifest: Epic Games installs are compared file-by-file via SHA1
    hashes from the CDN manifest, SteamCMD installs are validated through
    `+app_update <id> validate`.
*/

use std::path::PathBuf;
use std::sync::Arc;

use tracing::{error, info, warn};

use monarch_egs::{DownloadManager, VerifyProgress as EgsVerifyProgress, get_game_manifest};

use crate::monarch_games::egs_client::EgsClient;
use crate::monarch_games::games::GameType;
use crate::monarch_games::monarchgame::MonarchGame;
use crate::monarch_games::steam_client;

/// Platform used when installing games through monarch_egs. Managed installs
/// always use Windows builds, even on Linux/macOS (via umu/proton).
static MANAGED_PLATFORM: &str = "Windows";

/// Epic Games install metadata required to fetch a build manifest.
struct EpicInstallMetadata {
    namespace: String,
    catalog_id: String,
    app_name: String,
}

/// Live progress of an integrity verification run.
#[derive(Debug, Clone, Copy)]
pub struct VerificationProgress {
    /// Files hashed so far.
    pub files_checked: u64,
    /// Total files to verify.
    pub total_files: u64,
}

impl VerificationProgress {
    /// Completion as a whole percentage, floored to the nearest percent.
    pub fn percent(&self) -> u64 {
        if self.total_files == 0 {
            return 100;
        }
        ((self.files_checked.min(self.total_files) as f64 / self.total_files as f64) * 100.0)
            .floor() as u64
    }
}

/// Called whenever the whole-percent progress of a verification changes.
pub type ProgressCallback = Arc<dyn Fn(VerificationProgress) + Send + Sync>;

/// Verifies the files of an installed game managed by Monarch and returns a
/// human-readable summary of the result. Nothing is modified on disk. For
/// manifest-based verification, `on_progress` (files checked / total files)
/// is invoked every time the floored whole-percent progress changes; SteamCMD
/// validation reports no per-file progress.
pub async fn verify_game_integrity(
    game: &MonarchGame,
    on_progress: Option<ProgressCallback>,
) -> Result<String, String> {
    if !game.is_installed || !game.managed_by_monarch {
        return Err(String::from(
            "Integrity verification is only available for games installed and managed by Monarch.",
        ));
    }

    if game.stores.iter().any(|store| store.name == "epicgames") {
        return verify_epic_install(game, on_progress).await;
    }

    if game.stores.iter().any(|store| store.name == "steamcmd") {
        return verify_steam_install(game).await;
    }

    Err(String::from(
        "Integrity verification is not supported for this game's store.",
    ))
}

/// Verifies a Monarch-installed Epic Games title against its CDN manifest.
async fn verify_epic_install(
    game: &MonarchGame,
    on_progress: Option<ProgressCallback>,
) -> Result<String, String> {
    let metadata = epic_install_metadata(game)?;
    let install_dir = validated_install_dir(game)?;

    let mut client = EgsClient::new();
    if !client.credentials_exist() {
        info!(
            "monarch_games::integrity::verify_epic_install() No Epic Games credentials found, cannot verify {}",
            game.name
        );
        return Err(String::from(
            "Sign in to the Epic Games Store to verify this game.",
        ));
    }

    client
        .load_existing_user()
        .await
        .map_err(|e| format!("Failed to load Epic Games session | Err: {e}"))?;

    let token = client.user().session().get_access_token().await;
    let manifest = get_game_manifest(
        &token,
        MANAGED_PLATFORM,
        &metadata.namespace,
        &metadata.catalog_id,
        &metadata.app_name,
    )
    .await
    .map_err(|e| format!("Failed to fetch the game's manifest | Err: {e}"))?;

    // Verification only reads and hashes local files; run it on a blocking
    // thread so large installs never stall the async workers. Progress events
    // flow through this channel and are forwarded to the caller's callback.
    let (tx, mut rx) = tokio::sync::mpsc::channel::<EgsVerifyProgress>(16);
    let forwarder = tokio::spawn(async move {
        while let Some(progress) = rx.recv().await {
            if let Some(callback) = &on_progress {
                callback(VerificationProgress {
                    files_checked: progress.files_checked,
                    total_files: progress.total_files,
                });
            }
        }
    });

    let result = tokio::task::spawn_blocking(move || {
        futures::executor::block_on(
            DownloadManager::new(manifest, install_dir).verify_with_progress(tx),
        )
    })
    .await
    .map_err(|e| format!("Integrity verification task failed | Err: {e}"))
    .and_then(|verified| verified.map_err(|e| format!("Failed to verify the installation | Err: {e}")));

    // The sender is dropped once verification finishes, ending the forwarder.
    let _ = forwarder.await;

    let report = result?;

    let total_files = report.ok + report.missing + report.mismatched;
    if report.missing == 0 && report.mismatched == 0 {
        info!(
            "monarch_games::integrity::verify_epic_install() All {total_files} files of {} verified successfully",
            game.name
        );
        return Ok(format!("All {total_files} files verified successfully."));
    }

    warn!(
        "monarch_games::integrity::verify_epic_install() {} failed verification | OK: {}, missing: {}, mismatched: {}",
        game.name, report.ok, report.missing, report.mismatched
    );
    Ok(format!(
        "Verification finished: {} of {total_files} files are damaged or missing. Re-download or update the game to repair them.",
        report.missing + report.mismatched
    ))
}

/// Validates a Monarch-installed SteamCMD title. SteamCMD has no read-only
/// integrity check; `+app_update <id> validate` re-hashes every file and
/// repairs anything that is missing or corrupt.
async fn verify_steam_install(game: &MonarchGame) -> Result<String, String> {
    steam_client::update_game(&game.get_store_id())
        .await
        .map_err(|e| {
            error!(
                "monarch_games::integrity::verify_steam_install() Failed to validate {} | Err: {}",
                game.name,
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            String::from("SteamCMD failed to validate the game files.")
        })?;

    Ok(String::from(
        "SteamCMD validated the game files and repaired anything that was damaged or missing.",
    ))
}

/// Collects the Epic metadata required for manifest-based verification,
/// mirroring the checks used by the update check so both features see the
/// exact same set of supported installs.
fn epic_install_metadata(game: &MonarchGame) -> Result<EpicInstallMetadata, String> {
    let store = game
        .stores
        .iter()
        .find(|store| store.name == "epicgames")
        .ok_or_else(|| String::from("Game is not linked to the Epic Games Store."))?;

    let catalog_id = game.properties.other.get("catalog_id");
    let app_name = game.properties.other.get("app_name");

    // Games installed before Monarch tracked EGS asset data cannot be verified.
    let Some(catalog_id) = catalog_id else {
        return Err(String::from(
            "This install is missing its Epic Games catalog id and cannot be verified.",
        ));
    };
    let Some(app_name) = app_name else {
        return Err(String::from(
            "This install is missing its Epic Games app name and cannot be verified.",
        ));
    };

    if store.store_id.is_empty() || catalog_id.is_empty() || app_name.is_empty() {
        return Err(String::from(
            "This install is missing Epic Games metadata and cannot be verified.",
        ));
    }

    Ok(EpicInstallMetadata {
        namespace: store.store_id.clone(),
        catalog_id: catalog_id.clone(),
        app_name: app_name.clone(),
    })
}

/// Returns the recorded install directory after checking it looks usable.
fn validated_install_dir(game: &MonarchGame) -> Result<PathBuf, String> {
    let install_dir = &game.properties.install_dir;
    if install_dir.is_empty() || install_dir == "Error" {
        return Err(String::from(
            "The install location of this game is unknown and cannot be verified.",
        ));
    }

    let path = PathBuf::from(install_dir);
    if !path.is_dir() {
        return Err(format!(
            "The install location no longer exists: {install_dir}"
        ));
    }

    Ok(path)
}
