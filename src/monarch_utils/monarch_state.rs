use crate::monarch_games::monarch_client::get_library;
use crate::monarch_utils::monarch_fs::get_library_db_path;
use crate::monarch_utils::monarch_settings;
use crate::{
    monarch_games::monarchgame::MonarchGame, monarch_library::library::write_games,
    monarch_utils::monarch_settings::Settings,
};
use anyhow::{bail, Context, Result};
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use std::sync::RwLock;
use std::sync::{Arc, LazyLock};

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
    settings: Arc<RwLock<Settings>>,

    library_conn: Option<SqlitePool>,
}

impl Drop for MonarchState {
    fn drop(&mut self) {
        //self.library_conn.close();
    }
}

impl MonarchState {
    pub fn new() -> Self {
        Self {
            library_games: Vec::new(),
            settings: Arc::new(RwLock::new(Settings::new())),
            library_conn: None,
        }
    }

    pub fn init(&mut self) {
        self.library_games = get_library();
        self.settings = Arc::new(RwLock::new(
            monarch_settings::read_settings()
                .expect("monarch_state::init() -> Failed to read settings from disk")
                .try_into()
                .expect("monarch_state::init() -> Failed to convert into Settings"),
        ));
        self.library_conn = Some(SqlitePool::connect_lazy_with(
                SqliteConnectOptions::new()
                    .filename(get_library_db_path())
                    .create_if_missing(true)));
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

    /// Update a game.
    /// Useful when updating game properties and want to let
    /// the backend state know of it.
    pub fn update_game(&mut self, game: &MonarchGame) -> Result<()> {
        for (i, self_game) in self.library_games.iter_mut().enumerate() {
            if self_game.id == game.id {
                self.library_games[i] = game.clone();
                write_games(&self.library_games)
                    .with_context(|| "monarch_state::update_game() -> ")?;
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

    /// Get a copy of the Arc<RwLock<Settings>> contained in MONARCH_STATE
    pub fn get_settings_ptr(&self) -> Arc<RwLock<Settings>> {
        self.settings.clone()
    }

    pub fn get_db_pool_ref(&self) -> &SqlitePool {
        self.library_conn.as_ref().unwrap()
    }
}
