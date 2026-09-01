use std::sync::Arc;

use anyhow::{bail, Context, Result};
use sqlx::SqlitePool;

use crate::monarch_games::monarchgame::MonarchGame;
use crate::monarch_utils::monarch_sql;
use crate::monarch_utils::monarch_state::MONARCH_STATE;

/// Returns games stored in library.db3
pub async fn get_games() -> Result<Vec<MonarchGame>> {
    let pool: Arc<SqlitePool>;
    match MONARCH_STATE.read() {
        Ok(state) => {
            pool = state.get_db_pool_arc();
        }
        Err(e) => {
            bail!("library::get_games() Failed to lock on MONARCH_STATE! | Err: {e}")
        }
    }

    return monarch_sql::get_library(&pool)
        .await
        .with_context(|| "monarch_library::get_games() -> ");
}

/// Functionality for adding a new persistent game that's been installed.
pub async fn add_game(game: &MonarchGame) -> Result<()> {
    let pool: Arc<SqlitePool>;
    match MONARCH_STATE.write() {
        Ok(mut state) => {
            state.push_game(game.clone());
            pool = state.get_db_pool_arc();
        }
        Err(e) => {
            bail!("library::add_game() Failed to lock on MONARCH_STATE | Err: {e}")
        }
    }
    return monarch_sql::insert_game(&pool, game)
        .await
        .with_context(|| "library::add_game() -> ");
}

/// Functionality for persistently removing a game from library
pub async fn remove_game(game: &MonarchGame) -> Result<()> {
    let pool: Arc<SqlitePool>;
    let mut games: Vec<MonarchGame>;
    match MONARCH_STATE.write() {
        Ok(mut state) => {
            games = state.get_library_games();

            for (i, g) in games.iter_mut().enumerate() {
                if g.id == game.id {
                    state.remove_game(i);
                    break;
                }
            }

            pool = state.get_db_pool_arc();
        }
        Err(e) => {
            bail!("library::remove_game() Failed to get write lock on MONARCH_STATE | Err: {e}")
        }
    }

    monarch_sql::remove_game(&pool, game)
        .await
        .with_context(|| "library::remove_game() -> ")
}

/// Marks a game as uninstalled (is_installed = false) in both state and db.
pub async fn mark_game_uninstalled(game: &MonarchGame) -> Result<()> {
    let pool: Arc<SqlitePool>;
    let mut updated: MonarchGame = game.clone();
    updated.is_installed = false;

    match MONARCH_STATE.write() {
        Ok(mut state) => {
            state
                .update_game(updated.clone())
                .with_context(|| "library::mark_game_uninstalled() -> ")?;
            pool = state.get_db_pool_arc();
        }
        Err(e) => {
            bail!("library::mark_game_uninstalled() Failed to lock on MONARCH_STATE | Err: {e}")
        }
    }
    monarch_sql::mark_game_uninstalled(&pool, game)
        .await
        .with_context(|| "library::mark_game_uninstalled() -> ")
}

/// Updates the properties of a game in the library.
pub async fn update_game_properties(game: &MonarchGame) -> Result<()> {
    let pool: Arc<SqlitePool>;
    match MONARCH_STATE.write() {
        Ok(mut state) => {
            state
                .update_game(game.clone())
                .with_context(|| "games_library::update_game_properties() -> ")?;
            pool = state.get_db_pool_arc();
        }
        Err(e) => {
            bail!("library::update_game_properties() Failed to lock on MONARCH_STATE | Err: {e}")
        }
    }
    monarch_sql::update_game(&pool, game)
        .await
        .with_context(|| "library::update_game_properties() -> ")
}

/// Overwrites library games
pub async fn overwrite_games(games: &[MonarchGame]) -> Result<()> {
    let pool: Arc<SqlitePool>;
    match MONARCH_STATE.write() {
        Ok(mut state) => {
            pool = state.get_db_pool_arc();
            state.set_library_games(games);
        }
        Err(e) => {
            bail!("library::overwrite_games() Failed to get MONARCH_STATE write lock! | Err: {e}")
        }
    }
    monarch_sql::overwrite_games(&pool, games)
        .await
        .with_context(|| "library::overwrite_games() -> ")
}
