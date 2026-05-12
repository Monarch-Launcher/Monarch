use std::collections::HashMap;

use super::Platform;
use reqwest::{Client, Response};

use crate::{Asset, Session, User, games::Entitlement};

static ALL_OFFERS: &str = "launcher-public-service-prod06.ol.epicgames.com";
static OWNED_OFFERS: &str = "entitlement-public-service-prod08.ol.epicgames.com";
static METADATA_OFFERS: &str = "catalog-public-service-prod06.ol.epicgames.com";

pub async fn owned_games(user: &User, platform: Platform) -> Vec<Entitlement> {
    let session: Session = user.session();

    let client = Client::new();
    let url: String = format!(
        "https://{}/entitlement/api/account/{}/entitlements",
        OWNED_OFFERS,
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
        .bearer_auth(session.get_access_token())
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

    println!("{:#?}", response_text);

    let assets: Vec<Entitlement> = serde_json::from_str(&response_text).unwrap();

    get_games_metadata(&session, &assets).await;
    assets
}

async fn get_games_metadata(session: &Session, entitlements: &[Entitlement]) {
    let client = Client::new();

    for game in entitlements {
        let url: String = format!(
            "https://{}/catalog/api/shared/namespace/{}/bulk/items",
            METADATA_OFFERS, game.namespace
        );
        let params: HashMap<&str, String> = HashMap::from([
            ("label", session.get_label()),
            ("includeMainGameDetails", "true".to_string()),
            ("id", game.app_id.clone()),
        ]);

        let response: Response = client
            .get(&url)
            .header("User-Agent", session.get_user_agent())
            .bearer_auth(session.get_access_token())
            .query(&params)
            .send()
            .await
            .unwrap();

        println!("{:#?}", response);

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

        println!("{:#?}", response_text);
    }
}
