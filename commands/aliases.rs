use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use crate::commands::detect::Browser;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AliasConfig {
    pub aliases: HashMap<String, String>,
}

impl AliasConfig {
    pub fn load(path: &PathBuf) -> Self {
        if path.exists() {
            let data = fs::read_to_string(path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or(AliasConfig { aliases: HashMap::new() })
        } else {
            AliasConfig { aliases: HashMap::new() }
        }
    }

    pub fn save(&self, path: &PathBuf) {
        let json = serde_json::to_string_pretty(&self).unwrap();
        fs::write(path, json).unwrap();
    }

    pub fn add_alias(&mut self, tag: String, url: String) {
        self.aliases.insert(tag, url);
    }

    pub fn resolve(&self, tag: &str) -> Option<String> {
        self.aliases.get(tag).cloned()
    }
}
impl AliasConfig {
    pub fn remove_alias(&mut self, tag: &str) -> bool {
        self.aliases.remove(tag).is_some()
    }

    pub fn open_all(&self, browser: &Browser) {
        for (_, url) in &self.aliases {
            super::links::launch_link(browser, url);
        }
    }
}
