use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, value::Value};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use tracing::{error, info};

use crate::monarch_utils::monarch_fs::write_json_content;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MonarchCollection {
    id: String,
    name: String,
    game_ids: Vec<String>, // camelCase to work better with frontend and json
}

impl MonarchCollection {
    pub fn new(name: &str, games: Vec<String>) -> Self {
        Self {
            id: generate_hash(&name.to_string()).to_string(),
            name: name.to_string(),
            game_ids: games,
        }
    }
}

/// Creates a new collection.
pub fn new_collection(collection_name: String, game_ids: Vec<String>) {
    todo!()
}

/// Updates info about a collection.
pub fn update_collections(id: &str, new_name: &str, game_ids: Vec<String>) {
    todo!()
}

/// Returns JSON of collections in library
pub fn get_collections() {
    todo!()
}

/// Overwrites existing content in collections.json with the new content
fn write_collection_changes(collections: Value) {
    todo!()
}

/// Creates a unique hash for a MonarchCollection currently only based on its name
fn generate_hash<T: Hash>(name: &T) -> u64 {
    let mut hasher: DefaultHasher = DefaultHasher::new();
    name.hash(&mut hasher);

    hasher.finish()
}

/// Returns a Vec<MonarchCollection> instead of a json value to remove indentation in functions above.
fn get_collections_as_struct() {
    todo!()
}

/// Returns index of MonarchCollection with matching id.
fn find_collection_index(id: &str, collections: &[MonarchCollection]) {
    todo!()
}
