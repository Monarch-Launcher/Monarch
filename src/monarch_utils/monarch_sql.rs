use anyhow::{bail, Context, Result};
use sqlx::SqlitePool;

use crate::monarch_games::monarchgame::MonarchGame;

pub async fn get_library(pool: &SqlitePool) -> Result<Vec<MonarchGame>> {
    let mut conn = pool
        .acquire()
        .await
        .with_context(|| "monarch_sql::get_library() Failed to acquire connection! | Err: ")?;

    // Using the macro enables compile-time stuff
    let rows = sqlx::query_as::<_, MonarchGame>("SELECT * FROM library")
        .fetch_all(pool)
        .await;

    Ok(Vec::new())
}
