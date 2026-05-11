use reqwest::{Client, Response, Result};

use crate::auth::session::Session;

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

    pub fn from_config() -> Self {
        User::new()
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
        self.session = Session::from_auth_code(auth_code).await;
        Ok(())
    }
}
