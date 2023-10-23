use serde::{Serialize, Deserialize};
use serde_json::{value::Value, json};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::fs;
use core::result::Result;
use log::{info, error, warn};
use std::path::PathBuf;

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
    let path: PathBuf;
    match get_collections_json_path() {
        Ok(json_path) => { path = json_path; }
        Err(e) => {
            error!("collections::new_collection failed! Cannot get path to collections.json! | Error: {e}");
            return Err("Failed to get collections path!".to_string())
        }
    }

    let new_collec: MonarchCollection = MonarchCollection::new(&collection_name, game_ids);

    match get_collections_as_struct() {
        Ok(mut collecs) => {
            collecs.push(new_collec);
            
            if let Err(e) = write_json_content(json!(collecs), &path) {
                error!("collections::new_collection() failed! Error writing new collections to collections.json! | Error: {e}");
                return Err("Failed to write new collection!".to_string())
            }
            get_collections() // Refresh and return to frontend

        } Err(e) => {
            error!("collections::new_collection() failed! Failed to get collections/parse json! | Error: {e}");
            Err("Failed to get existing collections!".to_string())
        }
    }
}

/// Updates info about a collection.
pub fn update_collections( id: &str, new_name: &str, game_ids: Vec<String>) -> Result<Value, String> {
    match get_collections_as_struct() {
        Ok(mut collecs) => {
            match find_collection_index(id, &collecs) {
                Some(index) => {
                    let mut collection: &mut MonarchCollection = collecs.get_mut(index).unwrap();
                    collection.name = new_name.to_string();
                    collection.gameIds = game_ids;
                    
                    if let Err(e) = write_collection_changes(json!(collecs)) {
                        return Err(e)
                    }
                    get_collections()

                } None => {
                    warn!("collections::update_collections() Could not find id: {id} in collections.json! Doing nothing.");
                    Err("Failed to find specified collection!".to_string())
                }
            }
        } Err(e) => {
            error!("collections::update_collections() failed! Failed to get collections/parse json! | Error: {e}");
            Err("Failed to get existing collections!".to_string())
        }
    }
}

/// Deletes a specified collection
pub fn delete_collections(id: &str) -> Result<Value, String> {
    match get_collections_as_struct() {
        Ok(mut collecs) => {
            match find_collection_index(id, &collecs) {
                Some(index) => {
                    collecs.remove(index);
                    if let Err(e) = write_collection_changes(json!(collecs)) {
                        return Err(e)
                    }
                    get_collections()
                } None => {
                    warn!("collections::delete_collections() Could not find id: {id} in collections.json! Doing nothing.");
                    Err("Failed to find specified collection!".to_string())
                }
            }
        } Err(e) => {
            error!("collections::delete_collections() failed! Failed to get collections/parse json! | Error: {e}");
            Err("Failed to get existing collections!".to_string())
        }
    }
}

/// Returns JSON of collections in library
pub fn get_collections() -> Result<Value, String> {
    let path: PathBuf;
    match get_collections_json_path() {
        Ok(json_path) => { path = json_path; }
        Err(e) => {
            error!("collections::get_collections() failed! Cannot get collections path! | Error: {e}");
            return Err("Failed to get collections path!".to_string())
        }
    }

    match fs::File::open(&path) {
        Ok(file) => {
            match serde_json::from_reader(file) {
                Ok(json_content) => {
                    return Ok(json_content) // Finds existing collections.json

                } Err(e) => {
                    error!("collections::get_collections() failed! Failed to parse file content to json! | Error: {e}");
                    info!("Attempting to write new empty collection array!");

                    let monarch_collecs: Value = json!(Vec::<MonarchCollection>::new());
                    if let Err(e) = write_json_content(monarch_collecs.clone(), &path) {
                        error!("collections::get_collections() failed! Erro while writing collections to: {file_path} | Error: {e}", file_path = path.display());
                        return Err("Failed to create a new collections.json file!".to_string())
                    }
                    Ok(monarch_collecs) // If it succeeds at creating new collections.json
                }
            }

        } Err(e) => {
            error!("collections::get_collections() failed! Could not open: {file} | Error: {e}", file = path.display());
            info!("Attempting to create a new empty file: {}", path.display());

            let monarch_collecs: Value = json!(Vec::<MonarchCollection>::new());
        
            if let Err(e) = fs::File::create(&path) {
                error!("collections::get_collections() failed! Failed to create: {file} | Error: {e}", file = path.display());
                return Err("Failed to create a new collections.json file!".to_string())
            }
            if let Err(e) = write_json_content(monarch_collecs.clone(), &path) {
                error!("collections::get_collections() failed! Error while writing collections to : {file} | Error: {e}", file = path.display());
                return Err("Failed to create a new collections.json file!".to_string())
            }
            
            Ok(monarch_collecs) // If it succeeds at creating new collections.json
        }
    }
}

/// Overwrites existing content in collections.json with the new content
fn write_collection_changes(collections: Value) -> Result<(), String> {
    match get_collections_json_path() {
        Ok(path) => {
            match write_json_content(collections, &path) {
                Ok(_) => { 
                    info!("Updated collections.json content!");
                    Ok(())
                }
                Err(e) => {
                    error!("collections::write_collection_changes() failed! Error while writing changes to collections.json! | Error: {e}");
                    return Err("Failed to write changes to collections.json!".to_string())
                }
            }
        }
        Err(e) => {
            error!("collections::write_collection_changes() failed! Cannot get path to collections.json! | Error: {e}");
            return Err("Failed to get path to collections.json!".to_string())
        }
    }
}

/// Creates a unique hash for a MonarchCollection currently only based on its name
fn generate_hash<T: Hash>(name: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);

    hasher.finish()
}

/// Returns a Vec<MonarchCollection> instead of a json value to remove indentation in functions above.
fn get_collections_as_struct() -> Result<Vec<MonarchCollection>, String> {
    match get_collections() {
        Ok(value) => {
            match serde_json::from_value(value) {
                Ok(collections) => {
                    Ok(collections)
                } Err(e) => {
                    error!("collections::get_collections_as_struct() failed! Error while parsing from json! | Error: {e}");
                    Err("Failed to parse from json!".to_string())
                }
            }
        } Err(e) => {
            error!("collections::get_collections_as_struct() failed! Failed to get collections! | Error: {e}");
            Err("Failed to get collections!".to_string())
        }
    }
}

/// Returns index of MonarchCollection with matching id.
fn find_collection_index(id: &str, collections: &Vec<MonarchCollection>) -> Option<usize> {
    collections.iter().position(|x| *x.id == *id)
}