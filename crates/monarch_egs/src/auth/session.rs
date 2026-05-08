pub struct Session {
    access_token: String,
}

impl Session {
    pub fn new() -> Self {
        Self {
            access_token: String::new(),
        }
    }

    pub fn has_access_token(&self) -> bool {
        self.access_token.is_empty()
    }

    pub fn set_access_token(&mut self, token: String) {
        self.access_token = token;
    }

    pub fn get_access_token(&self) -> String {
        self.access_token.clone()
    }
}
