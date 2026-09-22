use std::sync::{Arc, RwLock};

use anyhow::{bail, Context, Result};
use sqlx::SqlitePool;

use crate::monarch_games::monarchgame::MonarchGame;
use crate::monarch_utils::monarch_sql;
use crate::monarch_utils::monarch_state::MonarchState;

/// Returns games stored in library.db3
pub async fn get_games(pool: Arc<SqlitePool>) -> Result<Vec<MonarchGame>> {
    return monarch_sql::get_library(&pool)
        .await
        .with_context(|| "monarch_library::get_games() -> ");
}

/// Functionality for adding a new persistent game that's been installed.
pub async fn add_game(pool: Arc<SqlitePool>, game: &MonarchGame) -> Result<()> {
    return monarch_sql::insert_game(&pool, game)
        .await
        .with_context(|| "library::add_game() -> ");
}

/// Functionality for persistently removing a game from library
pub async fn remove_game(
    state_handle: Arc<RwLock<MonarchState>>,
    game: &MonarchGame,
) -> Result<()> {
    let pool: Arc<SqlitePool>;

    match state_handle.write() {
        Ok(mut state) => {
            pool = state.get_db_pool_arc();
            let games: Vec<Arc<RwLock<MonarchGame>>> = state.get_library_games().to_vec();

            for (i, g_lock) in games.iter().enumerate() {
                if let Ok(g) = g_lock.read() {
                    if g.id == game.id {
                        state.remove_game(i);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            bail!("library::remove_game() Failed to acquire lock on MonarchState! | Err: {e}")
        }
    };

    monarch_sql::remove_game(&pool, game)
        .await
        .with_context(|| "library::remove_game() -> ")
}

/// Marks a game as uninstalled (is_installed = false) in both state and db.
pub async fn mark_game_uninstalled_in_db(pool: Arc<SqlitePool>, game: &MonarchGame) -> Result<()> {
    monarch_sql::mark_game_uninstalled(&pool, game)
        .await
        .with_context(|| "library::mark_game_uninstalled() -> ")
}

/// Updates the properties of a game in the library.
pub async fn update_game_properties_in_db(pool: Arc<SqlitePool>, game: &MonarchGame) -> Result<()> {
    monarch_sql::update_game(&pool, game)
        .await
        .with_context(|| "library::update_game_properties() -> ")
}

/// Overwrites library games
pub async fn overwrite_games(
    state_handle: Arc<RwLock<MonarchState>>,
    games: &[MonarchGame],
) -> Result<()> {
    let pool: Arc<SqlitePool>;
    match state_handle.write() {
        Ok(mut state) => {
            pool = state.get_db_pool_arc();
            let game_ptrs: Vec<Arc<RwLock<MonarchGame>>> = games
                .iter()
                .map(|g| Arc::new(RwLock::new(g.clone())))
                .collect();
            state.set_library_games(&game_ptrs);
        }
        Err(e) => {
            bail!("library::overwrite_games() Failed to get MONARCH_STATE write lock! | Err: {e}")
        }
    }

    monarch_sql::overwrite_games(&pool, games)
        .await
        .with_context(|| "library::overwrite_games() -> ")
}
