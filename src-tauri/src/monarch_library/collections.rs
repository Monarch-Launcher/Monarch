use anyhow::{anyhow, Context, Result};
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::{json, value::Value};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use crate::monarch_utils::monarch_fs::{get_collections_json_path, write_json_content};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
struct MonarchCollection {
    id: String,
    name: String,
    gameIds: Vec<String>, // camelCase to work better with frontend and json
}

impl MonarchCollection {
    pub fn new(name: &str, games: Vec<String>) -> Self {
        Self {
            id: generate_hash(&name.to_string()).to_string(),
            name: name.to_string(),
            gameIds: games,
        }
    }
}

/// Creates a new collection.
pub fn new_collection(collection_name: String, game_ids: Vec<String>) -> Result<Value> {
    let path: PathBuf = get_collections_json_path().with_context(|| -> String {"collections::new_collection failed! Cannot get path to collections.json! | Err".to_string()})?;
    let new_collec: MonarchCollection = MonarchCollection::new(&collection_name, game_ids);

    let mut collecs: Vec<MonarchCollection> =
        get_collections_as_struct().with_context(|| -> String {"collections::new_collection() failed! Failed to get collections/parse json! | Err".to_string()})?;

    collecs.push(new_collec);
    write_json_content(json!(collecs), &path).context("collections::new_collection() failed! Error writing new collections to collections.json! | Err")?;
    get_collections()
}

/// Updates info about a collection.
pub fn update_collections(id: &str, new_name: &str, game_ids: Vec<String>) -> Result<Value> {
    let mut collecs: Vec<MonarchCollection> = get_collections_as_struct().with_context(|| 
        -> String {"collections::update_collections() failed! Failed to get collections/parse json! | Err".to_string()})?;

    if let Some(index) = find_collection_index(id, &collecs) {
        let collection: &mut MonarchCollection = collecs.get_mut(index).with_context(|| 
            -> String {"collections::update_collections() failed! No collection found for index! | Err".to_string()})?;

        collection.name = new_name.to_string();
        collection.gameIds = game_ids;

        write_collection_changes(json!(collecs)).context("collections::update_collections() failed! Error writing updates to collections.json! | Err")?;
        return get_collections();
    }

    Err(anyhow!(
        "collections::update_collections() failed! No index found for collection: {id}"
    ))
}

/// Deletes a specified collection
pub fn delete_collections(id: &str) -> Result<Value> {
    let mut collecs = get_collections_as_struct().with_context(|| -> String {"collections::delete_collections() failed! Failed to get collections/parse json! | Err".to_string()})?;

    if let Some(index) = find_collection_index(id, &collecs) {
        collecs.remove(index);

        write_collection_changes(json!(collecs)).context("collections::delete_collections() failed! Error writing updates to collections.json! | Err")?;
        return get_collections();
    }

    Err(anyhow!("collections::delete_collections() failed! No index found for collection: {id}"))
}

/// Returns JSON of collections in library
pub fn get_collections() -> Result<Value> {
    let path: PathBuf = get_collections_json_path().with_context(|| -> String {"collections::get_collections() failed! Cannot get collections path! | Err".to_string()})?;

    match fs::File::open(&path) {
        Ok(file) => {
            serde_json::from_reader(file).context("collections::get_collections() failed! Failed to parse file content to json! Possibly reading an empty file. | Err")

        } Err(e) => {
            error!("collections::get_collections() failed! Could not open: {file} | Error: {e}", file = path.display());
            info!("Attempting to create a new empty file: {file}", file = path.display());

            let monarch_collecs: Value = json!(Vec::<MonarchCollection>::new());
        
            fs::File::create(&path)
                .context(format!("collections::get_collections() failed! Failed to create: {file} | Error: {e}", file = path.display()))?;
            
            write_json_content(monarch_collecs.clone(), &path)
                .context(format!("collections::get_collections() failed! Error while writing collections to : {file} | Error: {e}", file = path.display()))?;
            
            Ok(monarch_collecs) // If it succeeds at creating new collections.json
        }
    }
}

/// Overwrites existing content in collections.json with the new content
fn write_collection_changes(collections: Value) -> Result<()> {
    let path: PathBuf = get_collections_json_path().with_context(|| 
        -> String {"collections::write_collection_changes() failed! Cannot get path to collections.json! | Err".to_string()})?;

    write_json_content(collections, &path).context("collections::write_collection_changes() failed! Error while writing changes to collections.json! | Err")?;
    info!("Updated collections.json content!");

    Ok(())
}

/// Creates a unique hash for a MonarchCollection currently only based on its name
fn generate_hash<T: Hash>(name: &T) -> u64 {
    let mut hasher: DefaultHasher = DefaultHasher::new();
    name.hash(&mut hasher);

    hasher.finish()
}

/// Returns a Vec<MonarchCollection> instead of a json value to remove indentation in functions above.
fn get_collections_as_struct() -> Result<Vec<MonarchCollection>> {
    let collecs_json = get_collections().with_context(|| -> String {"collections::get_collections_as_struct() failed! Failed to get collections! | Err".to_string()})?;
    serde_json::from_value::<Vec<MonarchCollection>>(collecs_json).context("collections::get_collections_as_struct() failed! Error while parsing from json to Vec<MonarchGame>! | Err")
}

/// Returns index of MonarchCollection with matching id.
fn find_collection_index(id: &str, collections: &[MonarchCollection]) -> Option<usize> {
    collections.iter().position(|x| *x.id == *id)
}
