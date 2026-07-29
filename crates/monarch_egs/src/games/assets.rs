use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::User;

static LAUNCHER_URL: &str = "launcher-public-service-prod06.ol.epicgames.com";

/// A downloadable launcher asset. These fields are the ones required by the
/// assets/v2 CDN manifest endpoint (unlike entitlement name/catalog ids).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameAsset {
    #[serde(rename = "appName")]
    pub app_name: String,

    #[serde(rename = "catalogItemId")]
    pub catalog_item_id: String,

    #[serde(rename = "namespace")]
    pub namespace: String,

    #[serde(rename = "buildVersion", default)]
    pub build_version: String,

    #[serde(rename = "labelName", default)]
    pub label_name: String,
}

/// Returns owned downloadable assets for a platform (default Legendary path).
pub async fn owned_assets(user: &User, platform: &str) -> Vec<GameAsset> {
    let mut session = user.session();
    let client = Client::new();
    let url = format!("https://{LAUNCHER_URL}/launcher/api/public/assets/{platform}");

    let response = client
        .get(&url)
        .header("User-Agent", session.get_user_agent())
        .bearer_auth(session.get_access_token().await)
        .query(&[("label", "Live")])
        .send()
        .await
        .unwrap();

    if !response.status().is_success() {
        panic!("owned_assets() non-success status: {}", response.status());
    }

    let response_text = response.text().await.unwrap();
    serde_json::from_str::<Vec<GameAsset>>(&response_text).unwrap_or_default()
}

/// Pick the best downloadable asset for a namespace.
/// Skips obvious editor/devkit builds when a regular game asset exists.
pub fn pick_asset_for_namespace<'a>(
    assets: &'a [GameAsset],
    namespace: &str,
) -> Option<&'a GameAsset> {
    let ns_assets: Vec<&GameAsset> = assets
        .iter()
        .filter(|a| a.namespace == namespace)
        .collect();

    if ns_assets.is_empty() {
        return None;
    }

    ns_assets
        .iter()
        .copied()
        .find(|a| !is_likely_tool_asset(a))
        .or_else(|| ns_assets.first().copied())
}

fn is_likely_tool_asset(asset: &GameAsset) -> bool {
    let name = asset.app_name.to_ascii_lowercase();
    name.contains("devkit")
        || name.contains("editor")
        || name.ends_with("sdk")
        || name.contains("dedicatedserver")
}
