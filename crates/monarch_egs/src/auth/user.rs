use reqwest::Result;

use crate::auth::session::{Session, SessionTokenType};

#[derive(Debug, Default)]
pub struct User {
    session: Session,
}

impl User {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub async fn load_stored_user(refresh_token: &str) -> Self {
        Self {
            session: Session::from_token(SessionTokenType::RefreshToken(refresh_token.to_string()))
                .await,
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
        self.session = Session::from_token(SessionTokenType::AuthCode(code.to_string())).await;
        Ok(())
    }

    pub async fn get_access_token(&mut self) -> String {
        if self.session.session_expired() {
            self.session.refresh_session().await;
        }

        self.session.get_access_token()
    }
}
