mod auth;
mod download;
mod games;
mod utils;

pub use auth::{Session, User};
pub use download::{
    DownloadEvent, DownloadManager, DownloadPlan, DownloadProgress, DownloadReport,
    DownloaderOptions, Manifest, PrepManifestData, VerifyReport, VerifyStatus, get_game_manifest,
};
pub use games::{
    Entitlement, GameAsset, GameUpdate, InstalledBuild, Platform, check_updates,
    latest_build_version, owned_assets, owned_games, pick_asset_for_namespace,
};
