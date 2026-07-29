use serde::{Deserialize, Serialize};

mod assets;
mod info;
mod user;

pub enum Platform {
    Windows,
    MacOs,
    Linux,
}

impl Platform {
    pub fn as_str(&self) -> &str {
        match self {
            Platform::Windows => "Windows",
            Platform::MacOs => "MacOs",
            Platform::Linux => "Linux",
        }
    }
}

/// An entitlement is essentially ownership of a game on the Epic Games Store
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entitlement {
    #[serde(rename = "id")]
    pub app_id: String,

    #[serde(rename = "entitlementName")]
    pub entitlement_name: String,

    #[serde(rename = "catalogItemId")]
    pub catalog_id: String,

    #[serde(rename = "namespace")]
    pub namespace: String,
}

pub use assets::{GameAsset, owned_assets, pick_asset_for_namespace};
pub use user::owned_games;
