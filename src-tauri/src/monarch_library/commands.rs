use super::collections;
use core::result::Result; // Using different Result type for sending to frontend.
use serde_json::Value;

#[tauri::command]
/// Creates a new collection and writes to JSON
pub async fn create_collection(
    collection_name: String,
    game_ids: Vec<String>,
) -> Result<Value, String> {
    match collections::new_collection(collection_name, game_ids) {
        Ok(result) => Ok(result),
        Err(_) => Err(String::from(
            "Something went wrong while creating a new collection!",
        )),
    }
}

#[tauri::command]
/// Updates a collection and JSON file
pub async fn update_collection(
    id: String,
    new_name: String,
    game_ids: Vec<String>,
) -> Result<Value, String> {
    match collections::update_collections(&id, &new_name, game_ids) {
        Ok(result) => Ok(result),
        Err(_) => Err(String::from(
            "Something went wrong while updating collection!",
        )),
    }
}

#[tauri::command]
/// Deletes a collection and writes to JSON
pub async fn delete_collection(id: String) -> Result<Value, String> {
    match collections::delete_collections(&id) {
        Ok(result) => Ok(result),
        Err(_) => Err(String::from(
            "Something went wrong while deleting collection!",
        )),
    }
}

#[tauri::command]
/// Reads collections from JSON
pub async fn get_collections() -> Result<Value, String> {
    match collections::get_collections() {
        Ok(result) => Ok(result),
        Err(_) => Err(String::from(
            "Something went wrong while getting collections!",
        )),
    }
}
