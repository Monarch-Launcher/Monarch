use anyhow::Result;
use async_trait::async_trait;
use tracing::{error, info};

use monarch_egs::{Session, User};

use crate::monarch_games::games::SearchResult;
use crate::monarch_games::monarchgame::{GameImageType, MonarchGame, MonarchWebApiGame};
use crate::monarch_games::stores::DownloadOptions;
use crate::monarch_games::stores::SearchFilter;
use crate::monarch_utils::monarch_fs::generate_cache_image_path;
use crate::monarch_utils::monarch_settings::get_settings;

use super::stores::StoreType;

pub struct EgsClient {}

impl EgsClient {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn login(&self) -> Result<()> {
        info!("Logging in to Epic Games...");

        let mut egs_user: User = User::new();
        egs_user.start_auth();

        use std::io;
        use std::io::Read;
        println!("Please enter authorization code: ");
        let mut input = String::new().into_bytes();
        io::stdin().read_to_end(&mut input).unwrap();

        unsafe {
            egs_user
                .finish_auth(String::from_utf8_unchecked(input).as_str())
                .await
                .unwrap();
        }

        info!("Login complete!");

        Ok(())
    }
}

#[async_trait]
impl StoreType for EgsClient {
    async fn search_games(&self, name: &str, _filter: &SearchFilter) -> Vec<Box<dyn SearchResult>> {
        let monarch_url: &'static str = std::env!("MONARCH_URL");
        let search_term: String =
            format!("{}/api/games?search={}?store=epicgames", monarch_url, name,);

        let response = match reqwest::get(search_term).await {
            Ok(resp) => resp,
            Err(e) => {
                error!(
                    "monarch_client::search_games() reqwest::get() failed! | Err: {}",
                    e
                );
                return Vec::new();
            }
        };

        let resp_content = match response.text().await {
            Ok(content) => content,
            Err(e) => {
                error!(
                    "monarch_client::search_games() response.text() failed! | Err: {}",
                    e
                );
                return Vec::new();
            }
        };

        let mut web_games: Vec<Box<MonarchWebApiGame>> =
            match serde_json::from_str::<Vec<MonarchWebApiGame>>(&resp_content) {
                Ok(games) => games.into_iter().map(Box::new).collect(),
                Err(e) => {
                    error!(
                        "monarch_client::search_games() serde_json::from_str() failed! | Err: {}",
                        e
                    );
                    return Vec::new();
                }
            };

        for game in web_games.iter_mut() {
            let thumbnail_path = String::from(
                generate_cache_image_path(&game.name.clone(), GameImageType::Cover)
                    .to_str()
                    .unwrap(),
            );
            game.thumbnail_path = thumbnail_path;
        }

        web_games
            .into_iter()
            .map(|g| g as Box<dyn SearchResult>)
            .collect()
    }

    async fn install_game(&self, game: &MonarchGame, opts: &DownloadOptions) -> Result<()> {
        unimplemented!()
    }

    async fn uninstall_game(&self, game: &MonarchGame) -> Result<()> {
        unimplemented!()
    }

    async fn update_game(&self, game: &MonarchGame) -> Result<()> {
        unimplemented!()
    }

    fn game_is_installed(&self, _store_id: &str) -> bool {
        unimplemented!()
    }

    fn store_enabled(&self) -> bool {
        let settings_lock = match get_settings() {
            Ok(lock) => lock,
            Err(e) => {
                error!("egs::store_enabled() get_settings() failed! | Err: {}", e);
                return false;
            }
        };
        let settings = match settings_lock.read() {
            Ok(settings) => settings,
            Err(e) => {
                error!(
                    "egs::store_enabled() settings_lock.read() failed! | Err: {}",
                    e
                );
                return false;
            }
        };

        settings.epic.manage
    }

    async fn launch_game(&self, game: &MonarchGame) -> Result<()> {
        unimplemented!()
    }
}
