use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::User;
use crate::games::GameAsset;
use crate::utils::err::MonarchEgsError;

use super::owned_assets;

static LAUNCHER_URL: &str = "launcher-public-service-prod06.ol.epicgames.com";

/// Locally installed build info for one Epic game, used as update-check input.
/// `build_version` is the manifest `build_version()` recorded at install time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledBuild {
    pub namespace: String,
    pub catalog_item_id: String,
    pub app_name: String,
    pub build_version: String,
}

/// An available update for an installed game, i.e. the remote Live build no
/// longer matches the installed one.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameUpdate {
    pub namespace: String,
    pub catalog_item_id: String,
    pub app_name: String,
    pub installed_build_version: String,
    pub latest_build_version: String,
}

impl GameUpdate {
    /// True when the remote build differs from the installed build.
    pub fn is_update_available(&self) -> bool {
        build_differs(&self.installed_build_version, &self.latest_build_version)
    }
}

/// Checks a batch of installed games against Epic's Live assets list.
///
/// Performs a single request for all owned assets, then compares each entry of
/// `installed` against its matching asset (by namespace + app_name). Games with
/// no matching asset are skipped — there is nothing to compare against.
pub async fn check_updates(
    user: &User,
    platform: &str,
    installed: &[InstalledBuild],
) -> Result<Vec<GameUpdate>, MonarchEgsError> {
    let assets = owned_assets(user, platform).await?;
    Ok(check_updates_against_assets(&assets, installed))
}

/// Pure comparison used by [`check_updates`], exposed internally for testing.
fn check_updates_against_assets(
    assets: &[GameAsset],
    installed: &[InstalledBuild],
) -> Vec<GameUpdate> {
    let mut updates = Vec::new();

    for build in installed {
        let Some(asset) = find_asset(assets, build) else {
            continue;
        };

        if build_differs(&build.build_version, &asset.build_version) {
            updates.push(GameUpdate {
                namespace: build.namespace.clone(),
                catalog_item_id: asset.catalog_item_id.clone(),
                app_name: asset.app_name.clone(),
                installed_build_version: build.build_version.clone(),
                latest_build_version: asset.build_version.clone(),
            });
        }
    }

    updates
}

/// Returns the latest Live build version for a single game without downloading
/// its manifest. Hits the same label endpoint as the manifest CDN URL lookup.
pub async fn latest_build_version(
    user: &User,
    platform: &str,
    namespace: &str,
    catalog_item_id: &str,
    app_name: &str,
) -> Result<String, MonarchEgsError> {
    let mut session = user.session();

    let url: String = format!(
        "https://{LAUNCHER_URL}/launcher/api/public/assets/v2/platform/{platform}/namespace/{namespace}/catalogItem/{catalog_item_id}/app/{app_name}/label/Live",
    );

    let client: Client = Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", session.get_user_agent())
        .bearer_auth(session.get_access_token().await)
        .send()
        .await
        .map_err(|e| {
            MonarchEgsError::WebRequestError(format!(
                "latest_build_version() request failed! | Err: {e}"
            ))
        })?;

    if !response.status().is_success() {
        return Err(MonarchEgsError::WebRequestError(format!(
            "latest_build_version() non-success status: {}",
            response.status()
        )));
    }

    let response_object: serde_json::Value = response.json().await.map_err(|e| {
        MonarchEgsError::ParsingError(format!(
            "latest_build_version() failed to parse response! | Err: {e}"
        ))
    })?;

    parse_label_response(&response_object).ok_or_else(|| {
        MonarchEgsError::ParsingError("Missing 'buildVersion' attribute".to_string())
    })
}

/// Extracts the build version from a label endpoint response. Accepts both a
/// top-level object and an `{ "elements": [ ... ] }` wrapper.
fn parse_label_response(response: &serde_json::Value) -> Option<String> {
    if let Some(version) = response.get("buildVersion").and_then(|v| v.as_str()) {
        return Some(version.to_string());
    }

    response
        .get("elements")?
        .get(0)?
        .get("buildVersion")?
        .as_str()
        .map(|v| v.to_string())
}

/// Finds the asset matching an installed build by namespace + app_name,
/// falling back to catalog_item_id when app_names diverge between labels.
fn find_asset<'a>(assets: &'a [GameAsset], build: &InstalledBuild) -> Option<&'a GameAsset> {
    assets
        .iter()
        .find(|a| a.namespace == build.namespace && a.app_name == build.app_name)
        .or_else(|| {
            assets.iter().find(|a| {
                a.namespace == build.namespace && a.catalog_item_id == build.catalog_item_id
            })
        })
}

/// Builds differ when the local version is unknown or doesn't match remote.
fn build_differs(installed: &str, latest: &str) -> bool {
    if latest.is_empty() {
        return false;
    }
    installed.trim() != latest.trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(namespace: &str, app_name: &str, version: &str) -> GameAsset {
        GameAsset {
            app_name: app_name.to_string(),
            catalog_item_id: format!("cat-{app_name}"),
            namespace: namespace.to_string(),
            build_version: version.to_string(),
            label_name: "Live".to_string(),
        }
    }

    fn build(namespace: &str, app_name: &str, version: &str) -> InstalledBuild {
        InstalledBuild {
            namespace: namespace.to_string(),
            catalog_item_id: format!("cat-{app_name}"),
            app_name: app_name.to_string(),
            build_version: version.to_string(),
        }
    }

    #[test]
    fn detects_outdated_game() {
        let assets = vec![asset("ns1", "GameA", "2.0")];
        let installed = vec![build("ns1", "GameA", "1.0")];

        let updates = check_updates_against_assets(&assets, &installed);

        assert_eq!(updates.len(), 1);
        assert!(updates[0].is_update_available());
        assert_eq!(updates[0].installed_build_version, "1.0");
        assert_eq!(updates[0].latest_build_version, "2.0");
    }

    #[test]
    fn up_to_date_game_is_not_reported() {
        let assets = vec![asset("ns1", "GameA", "2.0")];
        let installed = vec![build("ns1", "GameA", "2.0")];

        assert!(check_updates_against_assets(&assets, &installed).is_empty());
    }

    #[test]
    fn game_without_remote_asset_is_skipped() {
        let assets = vec![asset("ns1", "GameA", "2.0")];
        let installed = vec![
            build("ns-other", "GameB", "1.0"),
            build("ns1", "GameB", "1.0"),
        ];

        assert!(check_updates_against_assets(&assets, &installed).is_empty());
    }

    #[test]
    fn empty_local_version_counts_as_update() {
        let assets = vec![asset("ns1", "GameA", "2.0")];
        let installed = vec![build("ns1", "GameA", "")];

        assert_eq!(check_updates_against_assets(&assets, &installed).len(), 1);
    }

    #[test]
    fn empty_remote_version_is_never_an_update() {
        let assets = vec![asset("ns1", "GameA", "")];
        let installed = vec![build("ns1", "GameA", "1.0")];

        assert!(check_updates_against_assets(&assets, &installed).is_empty());
    }

    #[test]
    fn matching_falls_back_to_catalog_item_id() {
        let mut a = asset("ns1", "GameA-Live", "3.0");
        a.catalog_item_id = "cat-GameA".to_string();
        let assets = vec![a];
        let installed = vec![build("ns1", "GameA", "1.0")];

        assert_eq!(check_updates_against_assets(&assets, &installed).len(), 1);
    }

    #[test]
    fn version_comparison_trims_whitespace() {
        assert!(!build_differs(" 1.0 ", "1.0"));
        assert!(build_differs("1.0", "1.1"));
        assert!(!build_differs("", ""));
    }

    #[test]
    fn parses_top_level_and_wrapped_label_responses() {
        let top = serde_json::json!({ "buildVersion": "1.2.3" });
        assert_eq!(parse_label_response(&top), Some("1.2.3".to_string()));

        let wrapped = serde_json::json!({
            "elements": [{ "buildVersion": "4.5.6" }]
        });
        assert_eq!(parse_label_response(&wrapped), Some("4.5.6".to_string()));

        assert_eq!(parse_label_response(&serde_json::json!({})), None);
    }
}
