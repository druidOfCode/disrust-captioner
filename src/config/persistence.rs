use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)] // Add Debug for logging
pub struct Config {
    pub speaker_names: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            speaker_names: HashMap::new(),
        }
    }

    pub fn load(path: &str) -> Self {
        if Path::new(path).exists() {
            let mut file = match File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Failed to open config file: {}", e);
                    return Config::new();
                }
            };
            let mut contents = String::new();
            if let Err(e) = file.read_to_string(&mut contents) {
                eprintln!("Failed to read config file: {}", e);
                return Config::new();
            }
            match serde_json::from_str(&contents) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to parse config file: {}", e);
                    Config::new()
                }
            }
        } else {
            Config::new()
        }
    }

    pub fn save(&self, path: &str) {
        match serde_json::to_string_pretty(self) {
            Ok(contents) => {
                let mut file = match File::create(path) {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("Failed to create config file: {}", e);
                        return;
                    }
                };
                if let Err(e) = file.write_all(contents.as_bytes()) {
                    eprintln!("Failed to write config file: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to serialize config: {}", e);
            }
        }
    }

    pub fn set_speaker_name(&mut self, id: String, name: String) {
        self.speaker_names.insert(id, name);
    }

    pub fn get_speaker_name(&self, id: &str) -> Option<&String> {
        self.speaker_names.get(id)
    }
}