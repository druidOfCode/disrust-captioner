use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    speaker_names: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            speaker_names: HashMap::new(),
        }
    }

    pub fn load(path: &str) -> Self {
        if Path::new(path).exists() {
            let mut file = File::open(path).expect("Failed to open config file");
            let mut contents = String::new();
            file.read_to_string(&mut contents).expect("Failed to read config file");
            serde_json::from_str(&contents).expect("Failed to parse config file")
        } else {
            Config::new()
        }
    }

    pub fn save(&self, path: &str) {
        let contents = serde_json::to_string_pretty(self).expect("Failed to serialize config");
        let mut file = File::create(path).expect("Failed to create config file");
        file.write_all(contents.as_bytes()).expect("Failed to write config file");
    }

    pub fn set_speaker_name(&mut self, id: String, name: String) {
        self.speaker_names.insert(id, name);
    }

    pub fn get_speaker_name(&self, id: &str) -> Option<&String> {
        self.speaker_names.get(id)
    }
}
