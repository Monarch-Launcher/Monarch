mod auth;
mod download;
mod games;
mod launch;
mod utils;

pub use auth::{Session, User};
pub use download::{
    DownloadEvent, DownloadManager, DownloadPhase, DownloadPlan, DownloadProgress, DownloadReport,
    DownloaderOptions, Manifest, PrepManifestData, VerifyProgress, VerifyReport, VerifyStatus,
    get_game_manifest,
};
pub use launch::{build_egs_launch_command, CompatLayer, EgsLaunchCommand};
pub use games::{
    Entitlement, GameAsset, GameUpdate, InstalledBuild, Platform, SupportedPlatforms,
    check_platform_support, check_updates, latest_build_version, owned_assets, owned_games,
    pick_asset_for_namespace,
};
