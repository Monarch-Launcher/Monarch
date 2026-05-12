use serde::{Deserialize, Serialize};

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

/// An asset is essentially a game on the Epic Games Store
#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    #[serde(rename = "appName")]
    pub app_id: String,

    #[serde(rename = "buildVersion")]
    pub build_version: String,

    #[serde(rename = "catalogItemId")]
    pub catalog_id: String,

    pub namespace: String,

    #[serde(rename = "assetId")]
    pub asset_id: String,
}

/// An entitlement is essentially ownership of a game on the Epic Games Store
#[derive(Debug, Serialize, Deserialize)]
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

pub use user::owned_games;
