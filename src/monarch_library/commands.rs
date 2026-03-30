use crate::{
    monarch_games::{commands::get_game_properties, monarchgame::MonarchGame},
    monarch_library::library::get_games,
};

use core::result::Result;
use futures::future::join_all;
use std::cmp::Ordering;
use tracing::error;

/// Returns MonarchGames from library.json
pub async fn get_library() -> Result<Vec<MonarchGame>, String> {
    match get_games().await {
        Ok(games) => Ok(games),
        Err(e) => {
            error!(
                "monarch_games::commands::get_library -> {}",
                e.chain().map(|e| e.to_string()).collect::<String>()
            );
            Err(String::from("Something went wrong getting library!"))
        }
    }
}

/// Simple function for generating suggested games on homescreen, based on
/// recent playtime.
pub async fn get_home_recomendations() -> Result<Vec<MonarchGame>, String> {
    match get_library().await {
        Ok(mut games) => {
            let mut properties_tasks = vec![];
            for game in games.iter_mut() {
                properties_tasks.push(get_game_properties(game));
            }
            join_all(properties_tasks).await;

            games.sort_by(|g1, g2| {
                if g1.properties.last_played > g2.properties.last_played {
                    Ordering::Less
                } else if g1.properties.last_played < g2.properties.last_played {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            });

            if games.len() > 4 {
                Ok(games[0..4].to_vec())
            } else {
                return Ok(games);
            }
        }
        Err(e) => {
            error!("monarch_games::commands::get_home_recomendations() Failed to get recomendations! | Err: {e}");
            Err(String::from("Something went wrong getting library!"))
        }
    }
}

/* 
/// Creates a new collection
pub async fn create_collection(collection_name: String, game_ids: Vec<String>) {
    todo!()
}

/// Updates a collection
pub async fn update_collection(id: String, new_name: String, game_ids: Vec<String>) {
    todo!()
}

/// Deletes a collection
pub async fn delete_collection(id: String) {
    todo!()
}

/// Reads collections
pub async fn get_collections() {
    todo!()
}
*/