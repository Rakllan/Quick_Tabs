use rayon::prelude::*; // For parallel iteration
use serde::Serialize; // For serializing structs
use std::collections::HashSet; // For deduplicating browser paths
use std::fs::{self, File}; // For file operations
use std::io::Write; // For writing to files
use std::path::{Path, PathBuf}; // For handling file paths

#[cfg(target_os = "windows")]
use winreg::enums::*; // Windows registry enums
#[cfg(target_os = "windows")]
use winreg::RegKey; // Windows registry access

use dirs::home_dir; // Get user's home directory
use shellexpand::tilde; // Expand ~ in paths

#[derive(Debug, Clone, Serialize)]
pub struct Browser {
    pub name: String, // Browser name
    pub path: String, // Executable path
}

impl Browser {
    pub fn new(name: &str, path: PathBuf) -> Self {
        Self {
            name: name.to_string(),
            path: normalize_path(&path), // Normalize path
        }
    }
}

fn normalize_path(p: &Path) -> String {
    // Canonicalize path or fallback to lossy string
    match fs::canonicalize(p) {
        Ok(cp) => cp.to_string_lossy().to_string(),
        Err(_) => p.to_string_lossy().to_string(),
    }
}

fn candidate_executables() -> Vec<&'static str> {
    // Common browser executable names
    vec![
        "chrome.exe", "google-chrome-stable", "google-chrome", "chromium.exe", "chromium",
        "firefox.exe", "firefox", "brave.exe", "brave-browser", "msedge.exe", "microsoft-edge",
        "opera.exe", "opera", "vivaldi.exe", "vivaldi",
    ]
}

#[cfg(target_os = "windows")]
fn candidate_paths_quick() -> Vec<PathBuf> {
    // Known Windows install paths for browsers
    let mut v: Vec<PathBuf> = Vec::new();
    let pf = std::env::var("PROGRAMFILES").ok();
    let pfx = std::env::var("PROGRAMFILES(X86)").ok();
    let local = std::env::var("LOCALAPPDATA").ok();

    if let Some(ref p) = pf { v.push(PathBuf::from(format!("{}\\Google\\Chrome\\Application\\chrome.exe", p))); }
    if let Some(ref p) = pfx { v.push(PathBuf::from(format!("{}\\Google\\Chrome\\Application\\chrome.exe", p))); }
    if let Some(ref l) = local { v.push(PathBuf::from(format!("{}\\Chromium\\Application\\chrome.exe", l))); }
    if let Some(ref p) = pf { v.push(PathBuf::from(format!("{}\\BraveSoftware\\Brave-Browser\\Application\\brave.exe", p))); }
    if let Some(ref p) = pf { v.push(PathBuf::from(format!("{}\\Mozilla Firefox\\firefox.exe", p))); }
    if let Some(ref p) = pfx { v.push(PathBuf::from(format!("{}\\Microsoft\\Edge\\Application\\msedge.exe", p))); }

    v
}

fn probe_quick() -> Vec<Browser> {
    // Fast detection via known paths and PATH
    let mut found = Vec::new();
    let exes = candidate_executables();

    for p in candidate_paths_quick() {
        if p.exists() {
            let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or("browser".to_string());
            found.push(Browser::new(&name, p));
        }
    }

    let paths_from_env: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).collect())
        .unwrap_or_default();

    let path_found: Vec<Browser> = exes.par_iter()
        .flat_map(|exe| {
            paths_from_env.par_iter().filter_map(move |dir| {
                let candidate = dir.join(exe);
                if candidate.exists() {
                    Some(Browser::new(exe, candidate))
                } else {
                    None
                }
            })
        })
        .collect();

    let mut set = HashSet::new();
    let mut out = Vec::new();

    for b in found.into_iter().chain(path_found.into_iter()) {
        if set.insert(b.path.clone()) {
            out.push(b);
        }
    }

    out
}

#[cfg(target_os = "windows")]
fn probe_registry() -> Vec<Browser> {
    // Detect browsers from Windows registry
    let mut result = Vec::new();

    for hive in &[HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER] {
        if let Ok(key) = RegKey::predef(*hive).open_subkey("SOFTWARE\\Clients\\StartMenuInternet") {
            for browser_name in key.enum_keys().flatten() {
                let subpath = format!("SOFTWARE\\Clients\\StartMenuInternet\\{}", browser_name);
                if let Ok(sub) = RegKey::predef(*hive).open_subkey(&subpath) {
                    if let Ok(cmd) = sub.open_subkey("shell\\open\\command") {
                        if let Ok(val) = cmd.get_value::<String, _>("") {
                            let cleaned = val.split_whitespace().next().unwrap_or(&val).trim_matches('"').to_string();
                            let exe_name = Path::new(&cleaned).file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or(browser_name.clone());
                            result.push(Browser::new(&exe_name, PathBuf::from(cleaned)));
                        }
                    }
                }
            }
        }
    }

    let mut set = HashSet::new();
    result.into_iter().filter(|b| set.insert(b.path.clone())).collect()
}

#[cfg(target_os = "linux")]
fn detect_default_linux() -> Option<Browser> {
    // Use xdg-settings to find default browser
    if let Ok(out) = std::process::Command::new("xdg-settings").args(["get", "default-web-browser"]).output() {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                let exe = s.split('.').next().unwrap_or(&s);
                if let Some(p) = which::which(exe).ok() {
                    return Some(Browser::new(exe, p));
                }
            }
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn detect_default_macos() -> Option<Browser> {
    None // macOS detection not implemented
}

pub fn detect_all() -> Vec<Browser> {
    // Detect all browsers across OS methods
    let mut found = probe_quick();

    #[cfg(target_os = "windows")]
    {
        let reg = probe_registry();
        for r in reg {
            if !found.iter().any(|b| b.path == r.path) {
                found.push(r);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(d) = detect_default_linux() {
            if !found.iter().any(|b| b.path == d.path) {
                found.insert(0, d);
            }
        }
    }

    write_outputs(&found).ok();
    found
}

fn write_outputs(found: &Vec<Browser>) -> std::io::Result<()> {
    // Write browser list to JSON and TXT files
    let json = serde_json::to_string_pretty(found).unwrap_or("[]".to_string());
    fs::write("browsers.json", json)?;

    let mut file = File::create("browsers.txt")?;
    for b in found {
        writeln!(file, "{} = {}", b.name, b.path)?;
    }

    Ok(())
}
