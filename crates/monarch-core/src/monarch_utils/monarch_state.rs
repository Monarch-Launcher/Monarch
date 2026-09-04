use crate::monarch_utils::monarch_game_downloader::MonarchDownloader;
use crate::monarch_utils::monarch_fs::get_library_db_path;
use crate::monarch_utils::monarch_settings;
use crate::{
    monarch_games::monarchgame::MonarchGame, monarch_games::updates::MonarchGameUpdate,
    monarch_utils::monarch_settings::Settings,
};
use anyhow::{bail, Result};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;
use std::sync::RwLock;
use std::sync::{Arc, LazyLock};
use tracing::{error, warn};

/// Global app state of app logic.
/// Initialises a blank MonarchState to avoid RwLock deadlock on init.
/// Therefore it uses new() here and can call MonarchState::init() in the main.rs init() function.
pub static MONARCH_STATE: LazyLock<RwLock<MonarchState>> =
    LazyLock::new(|| RwLock::new(MonarchState::new()));

/// A struct for storing some sort of global state that
/// the backend can access to recieve relevant info.
#[derive(Debug)]
pub struct MonarchState {
    library_games: Vec<MonarchGame>,
    /// Updates found by the latest update check for games managed by Monarch.
    available_updates: Vec<MonarchGameUpdate>,
    settings: Arc<RwLock<Settings>>,
    downloader: Arc<RwLock<MonarchDownloader>>,

    library_conn: Option<Arc<SqlitePool>>, // Using Arc<> allows for copying the 'ptr' across async threads
}

impl Drop for MonarchState {
    fn drop(&mut self) {
        futures::executor::block_on(self.library_conn.as_ref().unwrap().close());
    }
}

impl MonarchState {
    pub fn new() -> Self {
        Self {
            library_games: Vec::new(),
            available_updates: Vec::new(),
            settings: Arc::new(RwLock::new(Settings::new())),
            downloader: Arc::new(RwLock::new(MonarchDownloader::new())),
            library_conn: None,
        }
    }

    pub async fn init(&mut self) {
        self.settings = Arc::new(RwLock::new(
            monarch_settings::read_settings()
                .expect("monarch_state::init() -> Failed to read settings from disk")
                .try_into()
                .expect("monarch_state::init() -> Failed to convert into Settings"),
        ));

        match self.settings.write() {
            Ok(mut settings) => {
                settings.fix_settings();
            }
            Err(e) => {
                error!(
                    "monarch_state::init() Failed to lock on setting when verifying! | Err: {e}"
                );
            }
        }

        // Push the persisted download speed limit into the downloader so it
        // applies from launch without any UI interaction.
        let max_speed_bps = match self.settings.read() {
            Ok(settings) => settings.monarch.max_download_speed_bps(),
            Err(e) => {
                error!(
                    "monarch_state::init() Failed to lock on settings for speed limit! | Err: {e}"
                );
                0
            }
        };
        if let Ok(mut downloader) = self.downloader.write() {
            downloader.set_max_download_speed_bps(max_speed_bps);
        }

        self.library_conn = Some(Arc::new(SqlitePool::connect_lazy_with(
            SqliteConnectOptions::new()
                .filename(get_library_db_path())
                .create_if_missing(true),
        )));
    }

    /// Returns what the backend thinks is the users library.
    pub fn get_library_games(&self) -> Vec<MonarchGame> {
        self.library_games.clone()
    }

    /// For setting known library games.
    /// Should probably only be run when refreshing library.
    pub fn set_library_games(&mut self, games: &[MonarchGame]) {
        self.library_games = games.to_vec();
    }

    /// Simple abstraction for pushing new game into MONARCH_STATE
    pub fn push_game(&mut self, game: MonarchGame) {
        self.library_games.push(game);
    }

    /// Simple abstraction for removing a game at index
    pub fn remove_game(&mut self, index: usize) {
        self.library_games.remove(index);
    }

    /// Update a game.
    /// Useful when updating game properties and want to let
    /// the backend state know of it.
    pub fn update_game(&mut self, game: MonarchGame) -> Result<()> {
        for (i, self_game) in self.library_games.iter_mut().enumerate() {
            if self_game.id == game.id {
                self.library_games[i] = game;
                return Ok(());
            }
        }
        bail!("monarch_state::update_game() No matching game found!")
    }

    /// Returns a library game with matching id.
    /// Useful when you might need some properties of a game.
    pub fn get_game(&self, id: &str) -> Option<MonarchGame> {
        for game in self.library_games.iter() {
            if game.id == id {
                return Some(game.clone());
            }
        }
        None
    }

    /// Check against the current list of games if an id is already in use.
    /// Used when generating ids for locally imported game binaries.
    pub fn binary_game_id_collision(&self, id: &str) -> bool {
        for game in self.library_games.iter() {
            if game.id == id {
                return true;
            }
        }
        false
    }

    /// Updates found by the latest update check for games managed by Monarch.
    pub fn get_available_updates(&self) -> Vec<MonarchGameUpdate> {
        self.available_updates.clone()
    }

    /// For storing the result of an update check.
    pub fn set_available_updates(&mut self, updates: Vec<MonarchGameUpdate>) {
        self.available_updates = updates;
    }

    /// Get a copy of the Arc<RwLock<Settings>> contained in MONARCH_STATE
    pub fn get_settings_ptr(&self) -> Arc<RwLock<Settings>> {
        self.settings.clone()
    }

    /// Get a copy of the Arc<RwLock<MonarchDownloader>> contained in MONARCH_STATE
    pub fn get_downloader_ptr(&self) -> Arc<RwLock<MonarchDownloader>> {
        self.downloader.clone()
    }

    pub fn get_db_pool_arc(&self) -> Arc<SqlitePool> {
        self.library_conn.as_ref().unwrap().clone()
    }

    /// Attempt to fix RwLock of Settings if error occurs
    /// Could be useful in future
    pub fn _clear_settings_poison(&self) {
        if self.settings.is_poisoned() {
            warn!(
                "monarch_state::clear_settings_poison() detected poisoned state for settings lock! Clearing poison."
            );
            self.settings.clear_poison();
        } else {
            error!("monarch_state::clear_settings_poison() Settings is not poisoned! Nothing to clear.")
        }
    }
}
