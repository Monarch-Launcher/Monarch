use serde::{Serialize, Deserialize};
use serde_json::{value::Value, json};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::fs;
use core::result::Result;
use log::{info, error};
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
            error!("Failed to get collections path! | Message: {:?}", e);
            return Err("Failed to get collections path!".to_string())
        }
    }

    let new_collec: MonarchCollection = MonarchCollection::new(&collection_name, game_ids);

    match get_collections_as_struct() {
        Ok(mut collecs) => {
            collecs.push(new_collec);
            
            if let Err(e) = write_json_content(json!(collecs), path) {
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
        Ok(mut collecs) => {
            match find_collection_index(id, &collecs) {
                Some(index) => {
                    let mut collection: &mut MonarchCollection = collecs.get_mut(index).unwrap();
                    collection.name = new_name.to_string();
                    collection.gameIds = game_ids;
                    println!("{}", json!(collecs));
                    if let Err(e) = write_collection_changes(json!(collecs)) {
                        return Err(e)
                    }
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
            match find_collection_index(id, &collecs) {
                Some(index) => {
                    collecs.remove(index);
                    if let Err(e) = write_collection_changes(json!(collecs)) {
                        return Err(e)
                    }
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
    let path: PathBuf;
    match get_collections_json_path() {
        Ok(json_path) => { path = json_path; }
        Err(e) => {
            error!("Failed to get collections path! | Message: {:?}", e);
            return Err("Failed to get collections path!".to_string())
        }
    }

    match fs::File::open(path.clone()) {
        Ok(file) => {
            match serde_json::from_reader(file) {
                Ok(json_content) => {
                    return Ok(json_content) // Finds existing collections.json

                } Err(e) => {
                    error!("Failed to parse file content to json in get_collections() | Message: {}", e);
                    info!("Attempting to write new empty collection array!");

                    let monarch_collecs: Value = json!(Vec::<MonarchCollection>::new());
                    if let Err(e) = write_json_content(monarch_collecs.clone(), path.clone()) {
                        error!("Failed to write new collections to file: {} | Message: {}", path.display(), e);
                        return Err("Failed to create a new collections.json file!".to_string())
                    }
                    Ok(monarch_collecs) // If it succeeds at creating new collections.json
                }
            }

        } Err(e) => {
            error!("Failed to read json file in get_collections() | Message: {}", e);
            info!("Attempting to create a new empty file! ({})", path.display());

            let monarch_collecs: Value = json!(Vec::<MonarchCollection>::new());
        
            if let Err(e) = fs::File::create(path.clone()) {
                error!("Failed to create empty file: {} | Message: {}", path.display(), e);
                return Err("Failed to create a new collections.json file!".to_string())
            }
            if let Err(e) = write_json_content(monarch_collecs.clone(), path.clone()) {
                error!("Failed to write new collections to file: {} | Message: {}", path.display(), e);
                return Err("Failed to create a new collections.json file!".to_string())
            }
            
            Ok(monarch_collecs) // If it succeeds at creating new collections.json
        }
    }
}

fn write_collection_changes(collections: Value) -> Result<(), String> {
    match get_collections_json_path() {
        Ok(path) => {
            match write_json_content(collections, path) {
                Ok(_) => { 
                    info!("Updated collections.json content!");
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to write changes to collections.json! | Message: {:?}", e);
                    return Err("Failed to write changes to collections.json!".to_string())
                }
            }
        }
        Err(e) => {
            error!("Failed to get path to collections.json! | Message: {:?}", e);
            return Err("Failed to get path to collections.json!".to_string())
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

/// Returns index of MonarchCollection with matching id.
fn find_collection_index(id: &str, collections: &Vec<MonarchCollection>) -> Option<usize> {
    collections.iter().position(|x| *x.id == *id)
}