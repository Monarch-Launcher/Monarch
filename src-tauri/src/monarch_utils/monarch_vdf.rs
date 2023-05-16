/*
    This file is for parsing Valve's .vdf (Valve Data Format) format.
    It is used for reading content related to steam such as the users installed library, library locations in the filesystem, etc.
*/

use std::fs;
use keyvalues_parser::Vdf;

pub fn parse_vdf_file(path: &str) {
    let content = fs::read_to_string(path).unwrap();    
    let vdf_content = Vdf::parse(&content).unwrap();

    for values in vdf_content.value.unwrap_obj().values() {
        println!("{:?}", values)
    }
}