use reqwest::Client;
use sha1::{Sha1, Digest};

use crate::utils::err::MonarchEgsError;
use super::DownloadManifest;

static CDN_URL: &str = "launcher-public-service-prod06.ol.epicgames.com";

/// Returns a download manifest for Epic Games game of namespace
pub async fn get_game_manifest(
    access_token: &str,
    platform: &str,
    namespace: &str,
    catalog_id: &str,
    app_name: &str,
) -> Result<DownloadManifest, MonarchEgsError> {
    let (manifest_urls, base_urls, hash) = get_cdn_urls(access_token, platform, namespace, catalog_id, app_name).await.unwrap();

    let client: Client = Client::new();

    for url in manifest_urls.iter() {
        println!("Attempting download of {}", url);

        let response = client.get(url).send().await.unwrap();

        if response.status().is_success() {
            let manifest_data: Vec<u8> = response.bytes().await.unwrap().to_vec();

            let computed_hash = Sha1::digest(&manifest_data)
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>();

            if computed_hash != hash {
                return Err(MonarchEgsError::HashMismatchError(format!("Hash mismatch for manifest! | Computed: {:?}, Expected: {}", computed_hash, hash)));
            }

            println!("Hash checked out!");
            return Ok(DownloadManifest {
                manifest_urls,
                base_urls,
                hash,
                manifest_data,
            })
        }
    }
    
    Err(MonarchEgsError::WebRequestError(format!("All manifest downloads failed!")))
}

async fn get_cdn_urls(
    access_token: &str,
    platform: &str,
    namespace: &str,
    catalog_id: &str,
    app_name: &str,
) -> Result<(Vec<String>, Vec<String>, String), MonarchEgsError> {
    let url: String = format!(
        "https://{CDN_URL}/launcher/api/public/assets/v2/platform/{platform}/namespace/{namespace}/catalogItem/{catalog_id}/app/{app_name}/label/Live",
    );

    let client: Client = Client::new();
    let response = client.get(&url).bearer_auth(access_token).send().await.unwrap();
    let response_object: serde_json::Value = response.json().await.unwrap();

    // Get manifest hash
    let hash: String = match response_object.get("elements") {
        Some(elements) => match elements.get(0) {
            Some(first) => match first.get("hash") {
                Some(hash) => hash.to_string().replace("\"", "").replace("\\", ""),
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
    let manifests: Vec<serde_json::Value> = match response_object.get("elements") {
        Some(elements) => match elements.get(0) {
            Some(first) => match first.get("manifests") {
                Some(manifests) => manifests.as_array().unwrap().to_vec(),
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

    let mut manifest_urls: Vec<String> = Vec::new();
    let mut base_urls: Vec<String> = Vec::new();
    for manifest in manifests.iter() {
        let mut url: String = manifest.get("uri").unwrap().to_string().replace("\"", "");
        let url_parts: Vec<&str> = url.split('/').collect();
        let base_url: String = url_parts.clone().into_iter().take(url_parts.len() - 1).collect::<Vec<&str>>().join("/");
        
        if let Some(query_params) = manifest.get("queryParams") {
            let params: String = query_params.as_array()
                .unwrap()
                .iter()
                .map(|value| {
                    let k = value.get("name").unwrap_or_default().to_string();
                    let v = value.get("value").unwrap_or_default().to_string();

                    if k.is_empty() || v.is_empty() {
                        return String::new();
                    }
                    
                    format!("&{}={}", k.replace("\"", ""), v.replace("\"", ""))
                }).collect::<String>();

            url.push_str(&format!("?{}", &params));
        }

        manifest_urls.push(url);
        base_urls.push(base_url);
    }

    println!("manifest_urls: {:?}", manifest_urls);
    println!("base_urls: {:?}", base_urls);

    Ok((manifest_urls, base_urls, hash))
}