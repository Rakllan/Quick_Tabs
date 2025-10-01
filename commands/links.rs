use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::process::Command;
use crate::commands::detect::Browser;

#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    pub tag: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkConfig {
    pub links: Vec<Link>,
}

impl LinkConfig {
    pub fn load(path: &PathBuf) -> Self {
        if path.exists() {
            let data = fs::read_to_string(path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or(LinkConfig { links: vec![] })
        } else {
            LinkConfig { links: vec![] }
        }
    }

    pub fn save(&self, path: &PathBuf) {
        let json = serde_json::to_string_pretty(&self).unwrap();
        fs::write(path, json).unwrap();
    }

    pub fn add_link(&mut self, tag: String, url: String) {
        self.links.push(Link { tag, url });
    }

    pub fn get_url(&self, tag: &str) -> Option<String> {
        self.links.iter().find(|l| l.tag == tag).map(|l| l.url.clone())
    }

    pub fn list(&self) {
        if self.links.is_empty() {
            println!("⚠️ No links saved.");
        } else {
            println!("📄 Saved links:");
            for l in &self.links {
                println!("  [{}] {}", l.tag, l.url);
            }
        }
    }
}

/// Launch URL in the selected browser
pub fn launch_link(browser: &Browser, url: &str) {
    println!("🚀 Launching {} in {}", url, browser.path.display());
    if cfg!(target_os = "windows") {
        Command::new(&browser.path)
            .arg(url)
            .spawn()
            .expect("Failed to open browser");
    } else {
        Command::new(&browser.path)
            .arg(url)
            .spawn()
            .expect("Failed to open browser");
    }
}

impl LinkConfig {
    pub fn remove_link(&mut self, tag: &str) -> bool {
        if let Some(pos) = self.links.iter().position(|l| l.tag == tag) {
            self.links.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn open_all(&self, browser: &Browser) {
        for l in &self.links {
            super::links::launch_link(browser, &l.url);
        }
    }
}
