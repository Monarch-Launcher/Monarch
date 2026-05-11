use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

static OAUTH_HOST: &str = "https://account-public-service-prod03.ol.epicgames.com";
static BASIC_USERNAME: &str = "34a02cf8f4414e29b15921876da36f9a";
static BASIC_PASSWORD: &str = "daafbccc737745039dffe53d94fc76cf";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Session {
    access_token: String,
    refresh_token: String,
}

impl Session {
    pub async fn from_auth_code(auth_code: &str) -> Self {
        let url: String = format!("{OAUTH_HOST}/account/api/oauth/token");

        let form: HashMap<&str, &str> = HashMap::from([
            ("grant_type", "authorization_code"),
            ("code", auth_code),
            ("token_type", "eg1"),
        ]);

        let client: Client = Client::new();
        let response: Response = client
            .post(url)
            .form(&form)
            .basic_auth(BASIC_USERNAME, Some(BASIC_PASSWORD))
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
        serde_json::from_str::<Session>(&response_text).unwrap()
    }

    pub fn from_refresh_token(refresh_token: &str) -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn has_access_token(&self) -> bool {
        self.access_token.is_empty()
    }

    pub fn set_new_tokens(&mut self, access_token: String, refresh_token: String) {
        self.access_token = access_token;
        self.refresh_token = refresh_token;
    }

    pub fn refresh_session(&mut self) {}
}
