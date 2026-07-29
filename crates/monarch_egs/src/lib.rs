mod auth;
mod download;
mod games;
mod utils;

pub use auth::{Session, User};
pub use download::{DownloadManager, PrepManifestData, Manifest, get_game_manifest};
pub use games::{
    Entitlement, GameAsset, Platform, owned_assets, owned_games, pick_asset_for_namespace,
};
