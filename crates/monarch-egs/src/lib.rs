mod auth;
mod download;
mod games;
mod utils;

pub use auth::{GameToken, Session, User};
pub use download::{
    DownloadEvent, DownloadManager, DownloadPhase, DownloadPlan, DownloadProgress, DownloadReport,
    DownloaderOptions, Manifest, PrepManifestData, VerifyProgress, VerifyReport, VerifyStatus,
    get_game_manifest,
};
pub use games::{
    AttributeValue, CompatLayer, EgsLaunchCommand, Entitlement, GameAsset, GameMetadata,
    GameUpdate, InstalledBuild, MainGameItem, Platform, ReleaseInfo, SupportedPlatforms,
    build_egs_launch_command, check_platform_support, check_updates, get_game_metadata,
    latest_build_version, owned_assets, owned_games, pick_asset_for_namespace,
};
