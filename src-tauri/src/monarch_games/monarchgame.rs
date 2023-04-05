pub struct MonarchGame {
    name: String,
    id: String,
    platform: String,
}

impl MonarchGame {
    pub fn new(name: &str, id: &str, platform: &str) -> Self {
        MonarchGame { name: name.to_string(), id: id.to_string(), platform: platform.to_string() }
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
}