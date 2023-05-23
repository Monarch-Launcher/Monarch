use serde_json::Value;

use super::collections;

#[tauri::command]
/// Creates a new collection and writes to JSON
pub async fn create_collection(collection_name: String, game_ids: Vec<String>) -> Result<Value, String> {
    collections::new_collection(collection_name, game_ids)
}

#[tauri::command]
/// Updates a collection and JSON file
pub async fn update_collection( id: String, new_name: String, game_ids: Vec<String>) -> Result<Value, String> {
    collections::update_collections(&id, &new_name, game_ids)
}

#[tauri::command]
/// Deletes a collection and writes to JSON
pub async fn delete_collection(id: String) -> Result<Value, String> {
    collections::delete_collections(&id)
}

#[tauri::command]
/// Reads collections from JSON
pub async fn get_collections() -> Result<Value, String> {
    collections::get_collections()
}