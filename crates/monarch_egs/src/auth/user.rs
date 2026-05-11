use std::collections::HashMap;

use reqwest::{Client, Response, Result};
use serde_json::Value;

use crate::auth::session::Session;

static OAUTH_HOST: &str = "https://account-public-service-prod03.ol.epicgames.com";
static BASIC_USERNAME: &str = "34a02cf8f4414e29b15921876da36f9a";
static BASIC_PASSWORD: &str = "daafbccc737745039dffe53d94fc76cf";

pub struct User {
    session: Session,
}

impl User {
    pub fn new() -> Self {
        Self {
            session: Session::new(),
        }
    }

    pub fn start_auth(&self) {
        let url: &str = std::env!("EPICLOGIN_URL");
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open").arg(url).spawn();
        }
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("cmd.exe")
                .arg("/C")
                .arg(format!("start {}", url))
                .spawn();
        }
    }

    pub async fn finish_auth(&mut self, auth_code: &str) -> Result<()> {
        let code: &str = auth_code.trim();
        self.login_with_auth_code(code).await
    }

    async fn login_with_auth_code(&mut self, auth_code: &str) -> Result<()> {
        let form: HashMap<&str, &str> = HashMap::from([
            ("grant_type", "authorization_code"),
            ("code", auth_code),
            ("token_type", "eg1"),
        ]);

        let url: String = format!("{}/account/api/oauth/token", OAUTH_HOST);

        let client: Client = Client::new();
        let response: Response = client
            .post(url)
            .form(&form)
            .basic_auth(BASIC_USERNAME, Some(BASIC_PASSWORD))
            .send()
            .await?;

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

        let response_text: String = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_text).unwrap();
        let token: String = response_json["access_token"].to_string();
        self.session.set_access_token(token);

        Ok(())
    }
}
