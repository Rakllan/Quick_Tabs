// commands/aliases.rs
use std::collections::HashMap;
use std::fs;
use std::path::{PathBuf, Path};
use crate::commands::detect::Browser;
use crate::commands::links::{launch_link, LaunchMode, launch_urls_simultaneously};
use serde::{Serialize, Deserialize};
use std::io;

#[derive(Debug, Serialize, Deserialize)]
pub struct AliasConfig {
    pub aliases: HashMap<String, String>,
}

impl AliasConfig {
    pub fn load(path: &Path) -> Self {
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(data) => serde_json::from_str(&data).unwrap_or_else(|e| {
                    eprintln!("⚠️ Failed to parse alias config {}: {}", path.display(), e);
                    AliasConfig { aliases: HashMap::new() }
                }),
                Err(e) => {
                    eprintln!("⚠️ Failed to read alias config {}: {}", path.display(), e);
                    AliasConfig { aliases: HashMap::new() }
                }
            }
        } else {
            AliasConfig { aliases: HashMap::new() }
        }
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self)?;
        fs::write(path, json)
    }

    pub fn add_alias(&mut self, tag: String, url: String) {
        self.aliases.insert(tag, url);
    }

    pub fn resolve(&self, tag: &str) -> Option<String> {
        self.aliases.get(tag).cloned()
    }

    pub fn remove_alias(&mut self, tag: &str) -> bool {
        self.aliases.remove(tag).is_some()
    }

    pub fn list(&self) {
        if self.aliases.is_empty() {
            println!("⚠️ No aliases saved.");
        } else {
            println!("\n✨ Saved aliases:");
            for (tag, url) in &self.aliases {
                println!("  [{}] -> {}", tag, url);
            }
        }
    }
    
    pub fn open_all(&self, browser: &Browser, mode: LaunchMode) {
        if self.aliases.is_empty() {
            println!("⚠️ No aliases to open.");
            return;
        }

        let urls: Vec<&str> = self.aliases.values().map(|url| url.as_str()).collect();
        launch_urls_simultaneously(browser, &urls, mode);
    }
}
