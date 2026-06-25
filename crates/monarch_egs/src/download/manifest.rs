use reqwest::Client;

use crate::utils::err::MonarchEgsError;

pub struct Manifest {}

static MANIFEST_URL: &str = "launcher-public-service-prod06.ol.epicgames.com";
static CDN_URL: &str = "launcher-public-service-prod06.ol.epicgames.com/";

pub async fn get_cdn_urls(
    platform: &str,
    namespace: &str,
    catalog_id: &str,
    app_name: &str,
) -> Result<Vec<String>, MonarchEgsError> {
    let url: String = format!(
        "https://{CDN_URL}/launcher/api/public/assets/v2/platform/{platform}/namespace/{namespace}/catalogItem/{catalog_id}/app/{app_name}/label/Live",
    );

    let client: Client = Client::new();
    let response = client.get(&url).send().await.unwrap();
    let response_object: serde_json::Value = response.json().await.unwrap();

    println!("{:?}", response_object);

    /*
    // Get manifest hash
    let hash: String = match text.get("elements") {
        Some(elements) => match elements.get(0) {
            Some(first) => match first.get("hash") {
                Some(hash) => hash.to_string(),
                None => {
                    return Err(MonarchEgsError::ParsingError(
                        "Missing 'hash' attribute".to_string(),
                    ));
                }
            },
            None => {
                return Err(MonarchEgsError::ParsingError(
                    "'elements' missing index 0".to_string(),
                ));
            }
        },
        None => {
            return Err(MonarchEgsError::ParsingError(
                "Missing 'elements' attribute".to_string(),
            ));
        }
    };

    // Get manifest hash
    let hash: String = match text.get("elements") {
        Some(elements) => match elements.get(0) {
            Some(first) => match first.get("hash") {
                Some(hash) => hash.to_string(),
                None => {
                    return Err(MonarchEgsError::ParsingError(
                        "Missing 'hash' attribute".to_string(),
                    ));
                }
            },
            None => {
                return Err(MonarchEgsError::ParsingError(
                    "'elements' missing index 0".to_string(),
                ));
            }
        },
        None => {
            return Err(MonarchEgsError::ParsingError(
                "Missing 'elements' attribute".to_string(),
            ));
        }
    };
    */

    Ok(vec![])
}

/// Returns a download manifest for Epic Games game of namespace
pub async fn get_game_manifest(
    namespace: &str,
    app_name: &str,
    catalog_id: &str,
    platform: Option<&str>,
    label: Option<&str>,
) -> Manifest {
    let platform_: &str = platform.unwrap_or("Windows");
    let label_: &str = label.unwrap_or("Live");

    let url: String = format!(
        "https://{MANIFEST_URL}/launcher/api/public/assets/v2/platform/{platform_}/namespace/{namespace}/catalogItem/{catalog_id}/app/{app_name}/label/{label_}",
    );

    let client: Client = Client::new();

    let response = client.get(&url).send().await.unwrap();

    println!("resp: {:?}", response);

    let text = response.text().await.unwrap();
    println!("text: {}", text);

    Manifest {}
}
