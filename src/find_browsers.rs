use rayon::prelude::*;
use serde::Serialize;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

use dirs::home_dir;
use shellexpand::tilde;

/// Browser entry with normalized fields
#[derive(Debug, Clone, Serialize)]
pub struct Browser {
    pub name: String,
    pub path: String,
}

impl Browser {
    pub fn new(name: &str, path: PathBuf) -> Self {
        Self {
            name: name.to_string(),
            path: normalize_path(&path),
        }
    }
}

fn normalize_path(p: &Path) -> String {
    // to_string_lossy and canonicalize when possible
    match fs::canonicalize(p) {
        Ok(cp) => cp.to_string_lossy().to_string(),
        Err(_) => p.to_string_lossy().to_string(),
    }
}

/// Common executable names (both windows and unix-like)
fn candidate_executables() -> Vec<&'static str> {
    vec![
        "chrome.exe",
        "google-chrome-stable",
        "google-chrome",
        "chromium.exe",
        "chromium",
        "firefox.exe",
        "firefox",
        "brave.exe",
        "brave-browser",
        "msedge.exe",
        "microsoft-edge",
        "opera.exe",
        "opera",
        "vivaldi.exe",
        "vivaldi",
    ]
}

/// Candidate full-path locations to check quickly (do not recurse deeply)
#[cfg(target_os = "windows")]
fn candidate_paths_quick() -> Vec<PathBuf> {
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


/// Quick path checks + PATH probing (parallel)
fn probe_quick() -> Vec<Browser> {
    let mut found = Vec::new();
    let exes = candidate_executables();

    // Check exact common paths
    let quick_paths = candidate_paths_quick();
    quick_paths.into_par_iter().for_each(|p| {
        if p.exists() {
            // derive name from filename
            let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "browser".to_string());
            // write out to vector via file? we'll gather after
            // we return via channel; but simpler: collect in thread-safe vec using Mutex? We'll return via iterator.
        }
    });

    // We'll do a simple approach: check quick paths synchronously (fast) first
    for p in candidate_paths_quick() {
        if p.exists() {
            let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "browser".to_string());
            found.push(Browser::new(&name, p));
        }
    }

    // Check PATH for candidate executables in parallel
    let paths_from_env: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths).collect()
        })
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

    // combine and dedup by path
    let mut set = HashSet::new();
    let mut out = Vec::new();

    for b in found.into_iter().chain(path_found.into_iter()) {
        if set.insert(b.path.clone()) {
            out.push(b);
        }
    }

    out
}

/// On Windows: detect browsers from registry (StartMenuInternet)
#[cfg(target_os = "windows")]
fn probe_registry() -> Vec<Browser> {
    let mut result = Vec::new();
    let hk_local = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

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

    // deduplicate by path
    let mut set = std::collections::HashSet::new();
    result.into_iter().filter(|b| set.insert(b.path.clone())).collect()
}

/// On linux: try xdg-settings for default (not exhaustive)
#[cfg(target_os = "linux")]
fn detect_default_linux() -> Option<Browser> {
    if let Ok(out) = std::process::Command::new("xdg-settings").args(["get", "default-web-browser"]).output() {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                // typical value: firefox.desktop or google-chrome.desktop
                let exe = s.split('.').next().unwrap_or(&s);
                // search PATH
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
    // macOS default detection is messy; fallback to common quick probes
    None
}

/// Main exported function: detect browsers quickly, then fallback to deeper search if necessary
pub fn detect_all() -> Vec<Browser> {
    // quick probe
    let mut found = probe_quick();

    // windows registry probe
    #[cfg(target_os = "windows")]
    {
        let reg = probe_registry();
        for r in reg {
            if !found.iter().any(|b| b.path == r.path) {
                found.push(r);
            }
        }
    }

    // linux default check
    #[cfg(target_os = "linux")]
    {
        if let Some(d) = detect_default_linux() {
            if !found.iter().any(|b| b.path == d.path) {
                found.insert(0, d);
            }
        }
    }

    // mac default stub skipped

    // optionally write outputs for external use
    write_outputs(&found).ok();

    found
}

/// Write JSON and text outputs
fn write_outputs(found: &Vec<Browser>) -> std::io::Result<()> {
    // json
    let json = serde_json::to_string_pretty(found).unwrap_or_else(|_| "[]".to_string());
    fs::write("browsers.json", json)?;

    // text style
    let mut file = File::create("browsers.txt")?;
    for b in found {
        writeln!(file, "{} = {}", b.name, b.path)?;
    }

    Ok(())
}
