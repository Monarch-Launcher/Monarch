use serde::Deserialize;

use crate::MonarchSteamError;

#[derive(Debug, Deserialize, Clone)]
struct ConnectionManager {
    endpoint: String,
    legacy_endpoint: String,
    #[serde(rename = "type")]
    endpoint_type: String,
    dc: String,
    realm: String,
    load: u32,
    wtd_load: f32,
}

#[derive(Debug, Deserialize, Clone)]
struct DirBody {
    success: bool,
    serverlist: Vec<ConnectionManager>,
}

#[derive(Debug, Deserialize, Clone)]
struct DirResponse {
    response: DirBody,
}

/// Queries Steam for Connection Manager servers.
pub async fn discover(cell_id: u32) -> Result<Vec<ConnectionManager>, MonarchSteamError> {
    let url = format!("https://api.steampowered.com/ISteamDirectory/GetCMListForConnect/v1/?cellid={cell_id}&maxcount=20");

    let response = reqwest::get(&url)
        .await
        .map_err(|e| MonarchSteamError::WebRequestError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(MonarchSteamError::WebRequestError(format!("{url} returned HTTP {}", response.status().to_string())));
    }

    let dir_resp: DirResponse = response.json()
        .await
        .map_err(|e| MonarchSteamError::WebRequestError(e.to_string()))?;

    return Ok(dir_resp.response.serverlist)
}

/// Selects a suitable ConnectionManager from a list.
pub fn select_cm(list: &[ConnectionManager]) -> Option<ConnectionManager> {
    for cm in list {
        if cm.endpoint_type == "websockets" {
            return Some(cm.clone())
        }
    }
    None
}