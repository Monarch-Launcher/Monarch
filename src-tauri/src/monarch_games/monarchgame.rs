use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct MonarchGame {
    name: String,
    id: String,
    platform: String,
    executable_path: String,
    thumbnail_path: String,
}

impl MonarchGame {
    pub fn new(name: &str, id: &str, platform: &str, exec_path: &str, thumbnail_path: &str) -> Self {
        Self { name: name.to_string(), 
               id: id.to_string(), 
               platform: platform.to_string(),
               executable_path: exec_path.to_string(),
               thumbnail_path: thumbnail_path.to_string() }
    }

    /* 
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_platform(&self) -> &str {
        &self.platform
    }

    pub fn get_exec_path(&self) -> &str {
        &self.executable_path
    }

    pub fn get_thumbnail_path(&self) -> &str {
        &self.thumbnail_path
    }
    */
}