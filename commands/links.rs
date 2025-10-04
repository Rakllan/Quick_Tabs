// commands/links.rs
use std::collections::HashMap;
use std::fs;
use std::path::{PathBuf, Path};
use serde::{Serialize, Deserialize};
use std::process::Command;
use crate::commands::detect::Browser;
use std::io;

// --- Data Structures ---

#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    pub tag: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkConfig {
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, Copy)]
pub enum LaunchMode {
    Normal,
    Private,
}

// --- LinkConfig Implementation ---

impl LinkConfig {
    pub fn load(path: &Path) -> Self {
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(data) => serde_json::from_str(&data).unwrap_or_else(|e| {
                    eprintln!("⚠️ Failed to parse link config {}: {}", path.display(), e);
                    LinkConfig { links: vec![] }
                }),
                Err(e) => {
                    eprintln!("⚠️ Failed to read link config {}: {}", path.display(), e);
                    LinkConfig { links: vec![] }
                }
            }
        } else {
            LinkConfig { links: vec![] }
        }
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self)?;
        fs::write(path, json)
    }

    pub fn add_link(&mut self, tag: String, url: String) {
        if self.links.iter().any(|l| l.tag == tag) {
            println!("Replacing existing link for tag: {}", tag);
            self.links.retain(|l| l.tag != tag);
        }
        self.links.push(Link { tag, url });
    }

    pub fn get_url(&self, tag: &str) -> Option<String> {
        self.links.iter().find(|l| l.tag == tag).map(|l| l.url.clone())
    }

    pub fn list(&self) {
        if self.links.is_empty() {
            println!("⚠️ No links saved.");
        } else {
            println!("\n📄 Saved links:");
            for l in &self.links {
                println!("  [{}] {}", l.tag, l.url);
            }
        }
    }

    pub fn remove_link(&mut self, tag: &str) -> bool {
        if let Some(pos) = self.links.iter().position(|l| l.tag == tag) {
            self.links.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn open_all(&self, browser: &Browser, mode: LaunchMode) {
        if self.links.is_empty() {
            println!("⚠️ No links to open.");
            return;
        }
        
        // Collect URLs to launch simultaneously (better UX than sequential spawning)
        let urls: Vec<&str> = self.links.iter().map(|l| l.url.as_str()).collect();
        launch_urls_simultaneously(browser, &urls, mode);
    }
}

// --- Launch Logic ---

/// Determines the correct private mode flags based on the browser executable name.
fn get_private_flags(browser_path: &Path) -> &'static [&'static str] {
    let exe_lower = browser_path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();

    if exe_lower.contains("firefox") {
        &["-private-window"]
    } else if exe_lower.contains("msedge") {
        &["--inprivate"]
    } else if exe_lower.contains("brave") || exe_lower.contains("chrome") || exe_lower.contains("chromium") || exe_lower.contains("vivaldi") {
        // Default for Chromium family
        &["--incognito"]
    } else if exe_lower.contains("safari") {
        // Safari must be handled differently, usually via AppleScript, but since we are using
        // direct Command::new(), we might skip specific private mode for Safari on macOS
        // or rely on a user profile method, which is complex. Sticking to common flags.
        &[] 
    } else {
        &[] // Unknown browser or standard launch
    }
}


/// Launch a single URL in the selected browser
pub fn launch_link(browser: &Browser, url: &str, mode: LaunchMode) {
    let mode_str = match mode {
        LaunchMode::Normal => "Normal Mode",
        LaunchMode::Private => "Private Mode",
    };
    println!("🚀 Launching {} in {} ({})", url, browser.path.display(), mode_str);

    let mut command = Command::new(&browser.path);
    
    if let LaunchMode::Private = mode {
        let flags = get_private_flags(&browser.path);
        if flags.is_empty() {
            println!("⚠️ Warning: Private mode flags unknown for this browser. Launching normally.");
        } else {
            command.args(flags);
        }
    }

    if let Err(e) = command.arg(url).spawn() {
        eprintln!("⚠️ Failed to launch browser {}: {}", browser.path.display(), e);
    }
}

/// Launch multiple URLs in the selected browser instance.
pub fn launch_urls_simultaneously(browser: &Browser, urls: &[&str], mode: LaunchMode) {
    let mode_str = match mode {
        LaunchMode::Normal => "Normal Mode",
        LaunchMode::Private => "Private Mode",
    };
    println!("🚀 Launching {} link(s) in {} ({})", urls.len(), browser.path.display(), mode_str);

    let mut command = Command::new(&browser.path);

    if let LaunchMode::Private = mode {
        let flags = get_private_flags(&browser.path);
        if flags.is_empty() {
            println!("⚠️ Warning: Private mode flags unknown for this browser. Launching normally.");
        } else {
            command.args(flags);
        }
    }

    // Add all URLs as arguments
    command.args(urls);

    if let Err(e) = command.spawn() {
        eprintln!("⚠️ Failed to launch browser {}: {}", browser.path.display(), e);
    }
}
