use reqwest::{Client, Response};
use serde::Deserialize;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

static OAUTH_HOST: &str = "https://account-public-service-prod03.ol.epicgames.com";
static BASIC_USERNAME: &str = "34a02cf8f4414e29b15921876da36f9a";
static BASIC_PASSWORD: &str = "daafbccc737745039dffe53d94fc76cf";

pub enum SessionTokenType {
    AuthCode(String),
    RefreshToken(String),
}

#[derive(Debug)]
pub struct Session {
    access_token: String,
    refresh_token: String,
    expires: Instant,
}

impl Session {
    pub async fn from_token(token: SessionTokenType) -> Self {
        Self::authenticate_token(token).await
    }

    pub async fn refresh_session(&mut self) {
        let new_session: Session =
            Self::authenticate_token(SessionTokenType::RefreshToken(self.refresh_token.clone()))
                .await;

        self.access_token = new_session.access_token;
        self.refresh_token = new_session.refresh_token;
        self.expires = new_session.expires;
    }

    pub fn has_access_token(&self) -> bool {
        self.access_token.is_empty()
    }

    pub fn session_expired(&self) -> bool {
        Instant::now() >= self.expires
    }

    pub fn get_access_token(&self) -> String {
        self.access_token.clone()
    }

    async fn authenticate_token(token: SessionTokenType) -> Self {
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
        let token_resp: TokenResponse =
            serde_json::from_str::<TokenResponse>(&response_text).unwrap();

        token_resp.into()
    }
}

impl Default for Session {
    fn default() -> Self {
        Self {
            access_token: String::new(),
            refresh_token: String::new(),
            expires: Instant::now(),
        }
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
    account_id: String,
    client_id: String,
}

impl From<TokenResponse> for Session {
    fn from(value: TokenResponse) -> Self {
        let expire_instant: Instant = Instant::now()
            .checked_add(Duration::from_secs(value.expires_in))
            .unwrap();
        Self {
            access_token: value.access_token,
            refresh_token: value.refresh_token,
            expires: expire_instant,
        }
    }
}
