use reqwest::{Client, Response};
use serde::Deserialize;
use std::collections::HashMap;

use crate::User;
use crate::utils::err::MonarchEgsError;

static METADATA_URL: &str = "catalog-public-service-prod06.ol.epicgames.com";

#[derive(Debug, Deserialize, Clone)]
pub struct GameMetadata {
    pub id: String,
    pub namespace: String,
    pub title: String,
    #[serde(rename = "customAttributes")]
    pub custom_attributes: HashMap<String, AttributeValue>,
    pub categories: Vec<CategoryItem>,
    #[serde(rename = "mainGameItem")]
    pub main_game_item: Option<MainGameItem>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct AttributeValue {
    pub value: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CategoryItem {
    pub path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReleaseInfo {
    pub app_id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MainGameItem {
    pub id: String,
    pub title: String,
    pub release_info: Vec<ReleaseInfo>,
}

/// Returns metadata for a game from the EGS catalog API.
/// This info is required for properly launching EGS games.
pub async fn get_game_metadata(
    user: &User,
    namespace: &str,
    catalog_id: &str,
    country_code: &str,
    locale: &str,
) -> Result<GameMetadata, MonarchEgsError> {
    let mut session = user.session();
    let client = Client::new();
    let url: String =
        format!("https://{METADATA_URL}/catalog/api/shared/namespace/{namespace}/bulk/items");

    let access_token: String = session.get_access_token().await;

    let response: Response = client
        .get(url)
        .bearer_auth(&access_token)
        .query(&[("id", catalog_id)])
        .query(&[("includeDLCDetails", false)])
        .query(&[("includeMainGameDetails", true)])
        .query(&[("country", country_code)])
        .query(&[("locale", locale)])
        .send()
        .await
        .map_err(|e| {
            MonarchEgsError::WebRequestError(format!(
                "get_game_metadata() request failed! | Err: {e}"
            ))
        })?;

    let meta_map: HashMap<String, GameMetadata> = response.json().await.map_err(|e| {
        MonarchEgsError::ParsingError(format!("get_game_metadata() Failed to parse response into HashMap<String, GameMetadata>! | Err: {e}"))
    })?;
    let meta: GameMetadata = meta_map.get(catalog_id).cloned().unwrap();

    Ok(meta)
}
