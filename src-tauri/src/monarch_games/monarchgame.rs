use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MonarchGame {
    name: String,
    id: String, // Has to be string instead of u64 to avoid rounding when sent to frontend
    platform: String,
    platform_id: String,
    executable_path: String,
    thumbnail_path: String,
}

impl MonarchGame {
    pub fn new(
        name: &str,
        platform: &str,
        platform_id: &str,
        exec_path: &str,
        thumbnail_path: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            id: generate_hash(&name.to_string(), &platform.to_string(), &platform_id.to_string()).to_string(),
            platform: platform.to_string(),
            platform_id: platform_id.to_string(),
            executable_path: exec_path.to_string(),
            thumbnail_path: thumbnail_path.to_string(),
        }
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
        &self.platform_id
    }

    pub fn get_exec_path(&self) -> &str {
        &self.executable_path
    }

    pub fn get_thumbnail_path(&self) -> &str {
        &self.thumbnail_path
    }
}

/// Creates a unique hash for a MonarchGame based on its name, platform and platform_id
fn generate_hash<T: Hash>(name: &T, platform: &T, platform_id: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    platform.hash(&mut hasher);
    platform_id.hash(&mut hasher);

    return hasher.finish()
}