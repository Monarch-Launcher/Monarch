/*
    This file is for parsing Valve's .vdf (Valve Data Format) format.
    It is used for reading content related to steam such as the users installed library, library locations in the filesystem, etc.
*/

use std::fs;

struct VdfObject {
    key: String,
    values: Vec<Value>
}

impl VdfObject {
    pub fn new(key: &str, values: Vec<Value>) -> Self {
        return Self { key: key.to_string(), values: values }
    }

    pub fn get_values(&self) -> &Vec<Value> {
        return &self.values
    }

    pub fn get_key(&self) -> &str {
        return &self.key
    }
}

enum Value {
    Str(String),
    Obj(VdfObject),
}

pub fn parse_library_file(path: &str) -> Vec<String> {
    let games: Vec<String> = Vec::new();

    if let Ok(content) = fs::read_to_string(path) {
        
        println!("{:?}", content.split("{"))

    }

    return games
}

