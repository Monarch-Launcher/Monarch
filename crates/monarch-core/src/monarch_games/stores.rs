use std::fmt;
use std::sync::{Arc, RwLock};

use crate::{
    monarch_games::{games::SearchResult, monarchgame::MonarchGame},
    monarch_utils::{
        monarch_game_downloader::MonarchDownloader, monarch_settings::Settings,
        monarch_state::MonarchState,
    },
};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreEnum {
    Monarch,
    Steam,
    EpicGames,
    GOG,
    Itch,
    Other(String),
}

#[async_trait]
pub trait StoreType: Send + Sync {
    async fn search_games(
        &self,
        settings_handle: Arc<RwLock<Settings>>,
        name: &str,
        filter: &SearchFilter,
    ) -> Result<Vec<MonarchGame>>;
    async fn install_game(
        &mut self,
        downloader_handle: Arc<RwLock<MonarchDownloader>>,
        game: &mut MonarchGame,
        opts: &DownloadOptions,
    ) -> Result<()>;
    async fn uninstall_game(
        &self,
        settings_handle: Arc<RwLock<Settings>>,
        game: &MonarchGame,
    ) -> Result<()>;
    async fn update_game(
        &mut self,
        settings_handle: Arc<RwLock<Settings>>,
        game: &MonarchGame,
    ) -> Result<()>;
    fn game_is_installed(&self, store_id: &str) -> bool;
    fn store_enabled(&self, settings_handle: Arc<RwLock<Settings>>) -> bool;
    async fn launch_game(
        &mut self,
        state_handle: Arc<RwLock<MonarchState>>,
        game: &MonarchGame,
    ) -> Result<()>;
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchFilter {
    // Stores
    pub steam: bool,
    pub epic: bool,
    pub gog: bool,
    pub itch: bool,

    // Search sources
    pub monarch: bool,
    pub steam_powered: bool,
    pub egs: bool,
}

impl Default for SearchFilter {
    fn default() -> Self {
        Self {
            steam: true,
            epic: true,
            gog: true,
            itch: true,

            monarch: true,
            steam_powered: false,
            egs: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DownloadOptions {
    pub folder: String,
    pub store: StoreEnum,
    pub game_name: String,
    pub game_store_id: String,
    pub os: String,
    pub compatibility: Option<String>,
}

impl StoreEnum {
    pub fn as_str(&self) -> &str {
        match self {
            StoreEnum::Monarch => "monarch",
            StoreEnum::Steam => "steam",
            StoreEnum::EpicGames => "epicgames",
            StoreEnum::GOG => "gog",
            StoreEnum::Itch => "itch",
            StoreEnum::Other(s) => s,
        }
    }
}

impl fmt::Display for StoreEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl PartialEq<&str> for StoreEnum {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl From<&str> for StoreEnum {
    fn from(s: &str) -> Self {
        StoreEnum::from(s.to_string())
    }
}

impl From<String> for StoreEnum {
    fn from(s: String) -> Self {
        match s.as_str() {
            "monarch" => StoreEnum::Monarch,
            "steam" => StoreEnum::Steam,
            "epicgames" => StoreEnum::EpicGames,
            "gog" => StoreEnum::GOG,
            "itch" => StoreEnum::Itch,
            _ => StoreEnum::Other(s),
        }
    }
}

// Stored in SQLite as TEXT, same as String.
impl sqlx::Type<sqlx::Sqlite> for StoreEnum {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'de> sqlx::Decode<'de, sqlx::Sqlite> for StoreEnum {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'de>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        Ok(StoreEnum::from(s))
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for StoreEnum {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <String as sqlx::Encode<'q, sqlx::Sqlite>>::encode_by_ref(&self.to_string(), buf)
    }
}

impl <'de>Deserialize<'de> for StoreEnum {
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let s = <&str>::deserialize(deserializer)?;
        Ok(StoreEnum::from(s.to_string()))
    }
}

impl Serialize for StoreEnum {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let s = match self {
            StoreEnum::Monarch => "monarch",
            StoreEnum::Steam => "steam",
            StoreEnum::EpicGames => "epicgames",
            StoreEnum::GOG => "gog",
            StoreEnum::Itch => "itch",
            StoreEnum::Other(s) => s,
        };
        serializer.serialize_str(s)
    }
}
