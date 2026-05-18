use std::collections::HashMap;

use reqwest::{Client, Response};

use crate::{Session, User, games::Entitlement};

static ENTITLEMENTS_URL: &str = "entitlement-public-service-prod08.ol.epicgames.com";

pub async fn owned_games(user: &User) -> Vec<Entitlement> {
    let mut session: Session = user.session();

    let client: Client = Client::new();
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
