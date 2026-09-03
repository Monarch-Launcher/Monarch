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
pub use games::{
    AttributeValue, Entitlement, GameAsset, GameMetadata, GameUpdate, InstalledBuild, MainGameItem,
    Platform, ReleaseInfo, SupportedPlatforms, check_platform_support, check_updates,
    get_game_metadata, latest_build_version, owned_assets, owned_games, pick_asset_for_namespace,
};
pub use launch::{CompatLayer, EgsLaunchCommand, build_egs_launch_command};
