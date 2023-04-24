use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct MonarchGame {
    name: String,
    id: String,
    platform: String,
    platform_id: String,
    executable_path: String,
    thumbnail_path: String,
}

impl MonarchGame {
    pub fn new(name: &str, platform: &str, platform_id: &str,exec_path: &str, thumbnail_path: &str) -> Self {
        Self { name: name.to_string(), 
               id: generate_uuid(), 
               platform: platform.to_string(),
               platform_id: platform_id.to_string(),
               executable_path: exec_path.to_string(),
               thumbnail_path: thumbnail_path.to_string() }
    }
    
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_platform(&self) -> &str {
        &self.platform
    }

    pub fn get_platform_id(&self) -> &str {
        &self.platform
    }

    pub fn get_exec_path(&self) -> &str {
        &self.executable_path
    }

    pub fn get_thumbnail_path(&self) -> &str {
        &self.thumbnail_path
    }
}

/// Returns a UUID as String because uuid::Uuid does not support Serialize, Deserialize
fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}