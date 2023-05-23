use serde::{Serialize, Deserialize};
use serde_json::{value::Value, json};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::fs;
use core::result::Result;
use log::{info, error};

use crate::monarch_utils::monarch_fs::{write_json_content, get_collections_json_path};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
struct MonarchCollection {
    id: String,
    name: String,
    gameIds: Vec<String>, // camelCase to work better with frontend and json
}

impl MonarchCollection {
    pub fn new(name: &str, games: Vec<String>) -> Self {
        Self { id: generate_hash(&name.to_string()).to_string(), name: name.to_string(), gameIds: games }
    }
}

/// Creates a new collection.
pub fn new_collection(collection_name: String, game_ids: Vec<String>) -> Result<Value, String> {
    let path: String = get_collections_json_path();
    let new_collec: MonarchCollection = MonarchCollection::new(&collection_name, game_ids);

    match get_collections_as_struct() {
        Ok(mut collecs) => {
            collecs.push(new_collec);
            
            if let Err(e) = write_json_content(json!(collecs), &path) {
                error!("Failed to write new collections to collections.json! | Message: {}", e);
                return Err("Failed to write new collection!".to_string())
            }
            get_collections() // Refresh and return to frontend

        } Err(e) => {
            error!("Failed to get collections or parsing to json! | Message: {}", e);
            Err("Failed to get existing collections!".to_string())
        }
    }
}

/// Updates info about a collection.
pub fn update_collections( id: &str, new_name: &str, game_ids: Vec<String>) -> Result<Value, String> {
    match get_collections_as_struct() {
        Ok(collecs) => {
            match find_collection(id, collecs) {
                Some(mut collection) => {
                    collection.name = new_name.to_string();
                    collection.gameIds = game_ids;
                    get_collections()

                } None => {
                    Err("Failed to find specified collection!".to_string())
                }
            }
        } Err(e) => {
            error!("Failed to get collections or parsing to json! | Message: {}", e);
            Err("Failed to get existing collections!".to_string())
        }
    }
}

/// Deletes a specified collection
pub fn delete_collections(id: &str) -> Result<Value, String> {
    match get_collections_as_struct() {
        Ok(mut collecs) => {
            match find_collection_index(id, collecs.clone()) {
                Some(index) => {
                    collecs.remove(index);
                    get_collections()
                } None => {
                    Err("Failed to find specified collection!".to_string())
                }
            }
        } Err(e) => {
            error!("Failed to get collections or parsing to json! | Message: {}", e);
            Err("Failed to get existing collections!".to_string())
        }
    }
}

/// Returns JSON of collections in library
pub fn get_collections() -> Result<Value, String> {
    let path: String = get_collections_json_path();

    match fs::File::open(path.clone()) {
        Ok(file) => {
            match serde_json::from_reader(file) {
                Ok(json_content) => {
                    return Ok(json_content) // Finds existing collections.json

                } Err(e) => {
                    error!("Failed to parse file content to json in get_collections() | Message: {}", e);
                    info!("Attempting to write new empty collection array!");

                    let monarch_collecs: Value = json!(Vec::<MonarchCollection>::new());
                    if let Err(e) = write_json_content(monarch_collecs.clone(), &path) {
                        error!("Failed to write new collections to file: {} | Message: {}", path, e);
                        return Err("Failed to create a new collections.json file!".to_string())
                    }
                    Ok(monarch_collecs) // If it succeeds at creating new collections.json
                }
            }

        } Err(e) => {
            error!("Failed to read json file in get_collections() | Message: {}", e);
            info!("Attempting to create a new empty file! ({})", path);

            let monarch_collecs: Value = json!(Vec::<MonarchCollection>::new());
        
            if let Err(e) = fs::File::create(path.clone()) {
                error!("Failed to create empty file: {} | Message: {}", path, e);
                return Err("Failed to create a new collections.json file!".to_string())
            }
            if let Err(e) = write_json_content(monarch_collecs.clone(), &path) {
                error!("Failed to write new collections to file: {} | Message: {}", path, e);
                return Err("Failed to create a new collections.json file!".to_string())
            }
            
            Ok(monarch_collecs) // If it succeeds at creating new collections.json
        }
    }
}

/// Creates a unique hash for a MonarchCollection currently only based on its name
fn generate_hash<T: Hash>(name: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);

    return hasher.finish()
}

/// Returns a Vec<MonarchCollection> instead of a json value to remove indentation in functions above.
fn get_collections_as_struct() -> Result<Vec<MonarchCollection>, String> {
    match get_collections() {
        Ok(value) => {
            match serde_json::from_value(value) {
                Ok(collections) => {
                    Ok(collections)
                } Err(e) => {
                    error!("Failed to parse from json! | Message: {}", e);
                    Err("Failed to parse from json!".to_string())
                }
            }
        } Err(e) => {
            error!("Failed to get collections! | Message: {}", e);
            Err("Failed to get existing collections!".to_string())
        }
    }
}

/// Returns the correct MonarchCollection from a Vec<MonarchCollection>.
fn find_collection(id: &str, collections: Vec<MonarchCollection>) -> Option<MonarchCollection> {
    for collection in collections {
        if id == collection.id {
            return Some(collection)
        }
    }
    None
}

/// Returns index of MonarchCollection with matching id.
fn find_collection_index(id: &str, collections: Vec<MonarchCollection>) -> Option<usize> {
    collections.iter().position(|x| *x.id == *id)
}