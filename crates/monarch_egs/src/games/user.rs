use std::collections::HashMap;

use reqwest::{Client, Response};
use serde_json::Value;

use crate::{Session, User, games::Entitlement};

static _ALL_OFFERS: &str = "launcher-public-service-prod06.ol.epicgames.com";
static ENTITLEMENTS_URL: &str = "entitlement-public-service-prod08.ol.epicgames.com";
static METADATA_URL: &str = "catalog-public-service-prod06.ol.epicgames.com";

pub async fn owned_games(user: &User) -> Vec<Entitlement> {
    let mut session: Session = user.session();

    let client = Client::new();
    let url: String = format!(
        "https://{}/entitlement/api/account/{}/entitlements",
        ENTITLEMENTS_URL,
        session.get_account_id(),
    );
    let params: HashMap<&str, String> = HashMap::from([
        ("label", session.get_label()),
        ("start", "0".to_string()),
        ("count", "1000".to_string()),
    ]);

    let response: Response = client
        .get(&url)
        .header("User-Agent", session.get_user_agent())
        .bearer_auth(session.get_access_token().await)
        .query(&params)
        .send()
        .await
        .unwrap();

    if response.status().is_server_error() {
        // TODO: Do something
        panic!("http 5XX")
    } else if response.status().is_client_error() {
        // TODO: Do something
        panic!("http 4XX")
    } else if !response.status().is_success() {
        // TODO: Do something
        panic!("not 2XX")
    }

    let response_text: String = response.text().await.unwrap();
    serde_json::from_str::<Vec<Entitlement>>(&response_text).unwrap()
}

async fn get_games_metadata(session: &mut Session, entitlements: &[Entitlement]) {
    let client = Client::new();

    for game in entitlements {
        let url: String = format!(
            "https://{}/catalog/api/shared/namespace/{}/bulk/items",
            METADATA_URL, game.namespace
        );
        let params: HashMap<&str, String> = HashMap::from([
            ("label", session.get_label()),
            ("includeMainGameDetails", "true".to_string()),
            ("includeDLCDetails", "false".to_string()),
            ("id", game.catalog_id.clone()),
        ]);

        let response: Response = client
            .get(&url)
            .header("User-Agent", session.get_user_agent())
            .bearer_auth(session.get_access_token().await)
            .query(&params)
            .send()
            .await
            .unwrap();

        // println!("{:?}", response);

        if response.status().is_server_error() {
            // TODO: Do something
            panic!("http 5XX")
        } else if response.status().is_client_error() {
            // TODO: Do something
            panic!("http 4XX")
        } else if !response.status().is_success() {
            // TODO: Do something
            panic!("not 2XX")
        }

        let response_text: Value = response.json().await.unwrap();
        let title = response_text
            .get(game.catalog_id.clone())
            .unwrap()
            .get("title")
            .unwrap();

        println!("{} - {}", game.entitlement_name, title);
    }
}
