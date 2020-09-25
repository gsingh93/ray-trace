use std::fs::File;
use std::io::Read;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub scene_config: String,
    pub output_file: String,
    pub width: u32,
    pub height: u32,
    pub supersampling: u32,
    pub reflection_depth: u16,
}

impl Config {
    pub fn new(filename: &str) -> Self {
        let mut toml_str = String::new();
        File::open(filename)
            .unwrap()
            .read_to_string(&mut toml_str)
            .unwrap();

        toml::from_str(&toml_str).unwrap()
    }
}
