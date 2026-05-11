use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

static OAUTH_HOST: &str = "https://account-public-service-prod03.ol.epicgames.com";
static BASIC_USERNAME: &str = "34a02cf8f4414e29b15921876da36f9a";
static BASIC_PASSWORD: &str = "daafbccc737745039dffe53d94fc76cf";

pub enum SessionTokenType {
    AuthCode(String),
    RefreshToken(String),
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Session {
    access_token: String,
    refresh_token: String,
}

impl Session {
    pub async fn from(token: SessionTokenType) -> Self {
        let url: String = format!("{OAUTH_HOST}/account/api/oauth/token");

        let form: HashMap<&str, String> = match token {
            SessionTokenType::AuthCode(auth_code) => HashMap::from([
                ("grant_type", "authorization_code".to_string()),
                ("code", auth_code),
                ("token_type", "eg1".to_string()),
            ]),
            SessionTokenType::RefreshToken(refresh_token) => HashMap::from([
                ("grant_type", "refresh_token".to_string()),
                ("refresh_token", refresh_token),
                ("token_type", "eg1".to_string()),
            ]),
        };

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

    pub fn has_access_token(&self) -> bool {
        self.access_token.is_empty()
    }

    pub fn set_new_tokens(&mut self, access_token: String, refresh_token: String) {
        self.access_token = access_token;
        self.refresh_token = refresh_token;
    }

    pub fn refresh_session(&mut self) {}
}
