use crate::utils::err::MonarchEgsError;

use super::user::User;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

static OAUTH_HOST: &str = "account-public-service-prod03.ol.epicgames.com";
static ECOMMERCE_HOST: &str = "ecommerceintegration-public-service-ecomprod02.ol.epicgames.com";
static BASIC_USERNAME: &str = "34a02cf8f4414e29b15921876da36f9a";
static BASIC_PASSWORD: &str = "daafbccc737745039dffe53d94fc76cf";
static VERSION: &str = "15.18.2-29993784+++Portal+Release-Live";
static LABEL: &str = "Live";

pub enum SessionTokenType {
    AuthCode(String),
    RefreshToken(String),
}

#[derive(Debug, Deserialize)]
pub struct GameToken {
    #[serde(rename = "expiresInSeconds")]
    pub expires_in_seconds: u32,
    pub code: String,
    #[serde(rename = "creatingClientId")]
    pub creating_client_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    access_token: String,
    refresh_token: String,
    expires: SystemTime,
    account_id: String,
    client_id: String,
    client_secret: String,
    user_agent: String,
    store_user_agent: String,
    label: String,
}

impl Session {
    pub async fn from_token(token: SessionTokenType, user: &mut User) -> Self {
        Self::authenticate_token(token, Some(user)).await
    }

    pub async fn refresh_session(&mut self) {
        let new_session: Session = Self::authenticate_token(
            SessionTokenType::RefreshToken(self.refresh_token.clone()),
            None,
        )
        .await;

        self.access_token = new_session.access_token;
        self.refresh_token = new_session.refresh_token;
        self.expires = new_session.expires;
    }

    pub fn session_expired(&self) -> bool {
        SystemTime::now() >= self.expires
    }

    pub async fn get_access_token(&mut self) -> String {
        if self.session_expired() {
            self.refresh_session().await;
        }
        self.access_token.clone()
    }

    pub fn get_account_id(&self) -> String {
        self.account_id.clone()
    }

    pub fn get_client_id(&self) -> String {
        self.client_id.clone()
    }

    pub fn get_client_secret(&self) -> String {
        self.client_secret.clone()
    }

    pub fn get_user_agent(&self) -> String {
        self.user_agent.clone()
    }

    pub fn get_store_user_agent(&self) -> String {
        self.store_user_agent.clone()
    }

    pub fn get_label(&self) -> String {
        self.label.clone()
    }

    pub async fn get_game_token(&mut self) -> Result<GameToken, MonarchEgsError> {
        let url: String = format!("https://{OAUTH_HOST}/account/api/oauth/exchange");
        let client: Client = Client::new();

        let response: Response = client
            .get(url)
            .bearer_auth(self.get_access_token().await)
            .send()
            .await
            .map_err(|e| {
                MonarchEgsError::WebRequestError(format!(
                    "Session::get_game_token() Failed to send request! | Err: {e}"
                ))
            })?;

        let token: GameToken = response.json().await.map_err(|e| {
            MonarchEgsError::ParsingError(format!(
                "Session::get_game_token() Failed to parse response in GameToken! | Err: {e}"
            ))
        })?;

        Ok(token)
    }

    pub async fn get_ownership_token(
        &mut self,
        namespace: &str,
        catalog_id: &str,
    ) -> Result<Vec<u8>, MonarchEgsError> {
        let url: String = format!(
            "https://{ECOMMERCE_HOST}/ecommerceintegration/api/public/platforms/EPIC/identities/{}/ownershipToken",
            self.account_id
        );
        let client: Client = Client::new();
        let form: HashMap<&str, String> =
            HashMap::from([("nsCatalogItemId", format!("{namespace}:{catalog_id}"))]);

        let response: Response = client
            .get(url)
            .bearer_auth(self.get_access_token().await)
            .form(&form)
            .send()
            .await
            .unwrap();

        let token: Vec<u8> = response.bytes().await.map_err(|e| {
            MonarchEgsError::ParsingError(format!(
                "Session::get_ownership_token() Failed to parse response in GameToken! | Err: {e}"
            ))
        })?.to_vec();

        Ok(token)
    }

    async fn authenticate_token(token: SessionTokenType, user: Option<&mut User>) -> Self {
        let url: String = format!("https://{OAUTH_HOST}/account/api/oauth/token");

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

        if let Some(u) = user {
            u.set_display_name(&token_resp.display_name);
        }

        token_resp.into()
    }
}

impl Default for Session {
    fn default() -> Self {
        Self {
            access_token: String::new(),
            refresh_token: String::new(),
            expires: SystemTime::now(),
            account_id: String::new(),
            store_user_agent: format!("EpicGamesLauncher//{}", VERSION),
            user_agent: format!("UELauncher/{} Windows/10.0.19041.1.256.64bit", VERSION),
            client_id: String::new(),
            client_secret: String::new(),
            label: LABEL.to_string(),
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
    #[serde(rename = "displayName")]
    display_name: String,
}

impl From<TokenResponse> for Session {
    fn from(value: TokenResponse) -> Self {
        let expire_instant: SystemTime = SystemTime::now()
            .checked_add(Duration::from_secs(value.expires_in))
            .unwrap();

        let mut client_id = value.client_id;
        let mut client_secret = "".to_string();
        if client_id.is_empty() || client_secret.is_empty() {
            client_id = BASIC_USERNAME.to_string();
            client_secret = BASIC_PASSWORD.to_string();
        }

        Self {
            access_token: value.access_token,
            refresh_token: value.refresh_token,
            expires: expire_instant,
            account_id: value.account_id,
            user_agent: format!("UELauncher/{} Windows/10.0.19041.1.256.64bit", VERSION),
            client_id,
            client_secret,
            store_user_agent: format!("EpicGamesLauncher//{}", VERSION),
            label: LABEL.to_string(),
        }
    }
}
