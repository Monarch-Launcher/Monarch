use crate::{
    monarch_games::{
        games::GameType,
        monarchgame::{MonarchGame, MonarchGameProperties},
    }, monarch_utils::{monarch_state::MonarchState, monarch_vdf},
};

#[cfg(target_os = "windows")]
use crate::monarch_games::windows::steam;

#[cfg(target_os = "macos")]
use crate::monarch_games::macos::steam;

#[cfg(target_os = "linux")]
use crate::monarch_games::linux::steam;

use core::result::Result;
use std::{cmp::Ordering, sync::{Arc, RwLock}};
use tracing::error;

/// Simple function for generating suggested games on homescreen, based on
/// recent playtime.
pub async fn get_home_recomendations(state_handle: Arc<RwLock<MonarchState>>) -> Result<Vec<Arc<RwLock<MonarchGame>>>, String> {
    match state_handle.write() {
        Ok(state) => {
            let games = state.get_library_games();

            // Refresh last_played for installed Steam games directly from the
            // local appmanifest files (disk-only, no network) so the sort below
            // reflects actual recent play instead of possibly stale DB values.
            for game_lock in games.iter() {
                if let Ok(mut game) = game_lock.write() {
                    let mut store = game.get_store_name();
                    if store == "steamcmd" {
                        store = "steam".to_string();
                    }

                    if game.is_installed() && store == "steam" {
                        match steam::get_default_libraryfolders_location() {
                            Ok(p) => {
                                // Merge the locally available manifest fields so
                                // cards show correct size/install dir even when
                                // the DB row was never fully enriched.
                                let props: MonarchGameProperties =
                                    monarch_vdf::get_game_properties_from_manifest(&game, &p).into();
                                game.properties.last_played = props.last_played;
                                if !props.install_dir.is_empty() {
                                    game.properties.install_dir = props.install_dir;
                                }
                                if props.size_on_disk > 0 {
                                    game.properties.size_on_disk = props.size_on_disk;
                                }
                            }
                            Err(e) => {
                                error!("monarch_library::get_home_recomendations() Failed to get path to Steams libraryfolders.vdf! | Err: {e}");
                            }
                        }
                    }
                }
                
            }

            // Sort by last_played, newest first. Timestamps are stored as unix
            // epoch strings, so compare them numerically.
            let mut cloned_vec: Vec<Arc<RwLock<MonarchGame>>> = games.to_vec();
            cloned_vec.sort_by(|g1, g2| {
                let t1 = g1.read().unwrap().properties.last_played.parse::<i64>().unwrap_or(0);
                let t2 = g2.read().unwrap().properties.last_played.parse::<i64>().unwrap_or(0);
                if t1 > t2 {
                    Ordering::Less
                } else if t1 < t2 {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            });

            if cloned_vec.len() > 4 {
                Ok(cloned_vec[0..4].to_vec())
            } else {
                return Ok(cloned_vec);
            }
        }
        Err(e) => {
            error!("monarch_games::commands::get_home_recomendations() Failed to get recomendations! | Err: {e}");
            Err(String::from("Something went wrong getting library!"))
        }
    }
}

/// Creates a new collection
pub async fn create_collection(_collection_name: String, _game_ids: Vec<String>) {
    unimplemented!()
}

/// Updates a collection
pub async fn update_collection(_id: String, _new_name: String, _game_ids: Vec<String>) {
    unimplemented!()
}

/// Deletes a collection
pub async fn delete_collection(_id: String) {
    unimplemented!()
}

/// Reads collections
pub async fn get_collections() {
    unimplemented!()
}
