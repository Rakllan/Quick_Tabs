use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Known browser executable names per platform
fn browser_candidates() -> Vec<&'static str> {
    vec![
        "chrome.exe", "chromium.exe", "firefox.exe", "brave.exe", "msedge.exe", "opera.exe", // Windows
        "chrome", "chromium", "firefox", "brave-browser", "microsoft-edge", "opera",        // Linux/macOS
    ]
}

/// Candidate directories to search
#[cfg(target_os = "windows")]
fn candidate_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from(r"C:\Program Files"),
        PathBuf::from(r"C:\Program Files (x86)"),
    ];
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        dirs.push(PathBuf::from(local));
    }
    dirs
}

#[cfg(target_os = "linux")]
fn candidate_dirs() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/usr/bin"),
        PathBuf::from("/usr/local/bin"),
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")).join(".local/share/applications"),
    ]
}

#[cfg(target_os = "macos")]
fn candidate_dirs() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/Applications"),
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")).join("Applications"),
        PathBuf::from("/usr/bin"),
        PathBuf::from("/usr/local/bin"),
    ]
}

/// Recursively search for browser executables
fn search_dir(base: &Path, names: &[&str], results: &mut Vec<PathBuf>) {
    if !base.exists() {
        return;
    }

    let entries = match fs::read_dir(base) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            #[cfg(target_os = "windows")]
            {
                let skip = ["Windows", "$Recycle.Bin", "ProgramData"];
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if skip.contains(&name) {
                        continue;
                    }
                }
            }
            search_dir(&path, names, results);
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if names.iter().any(|&b| b.eq_ignore_ascii_case(name)) {
                results.push(path.clone());
            }
        }
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    pub use winreg::enums::*;
    pub use winreg::RegKey;

    /// Detect default browser on Windows via registry
    pub fn detect_default_browser() -> Option<PathBuf> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu.open_subkey(
            r"Software\Microsoft\Windows\Shell\Associations\UrlAssociations\http\UserChoice"
        ).ok()?;
        let prog_id: String = key.get_value("ProgId").ok()?;

        let browser_mapping = [
            ("ChromeHTML", "chrome.exe"),
            ("FirefoxURL", "firefox.exe"),
            ("MSEdgeHTM", "msedge.exe"),
            ("OperaStable", "opera.exe"),
            ("BraveHTML", "brave.exe"),
        ];

        for (id, exe) in browser_mapping {
            if prog_id.eq_ignore_ascii_case(id) {
                // Try to find executable in candidate dirs
                for dir in super::candidate_dirs() {
                    let exe_path = dir.join(exe);
                    if exe_path.exists() {
                        return Some(exe_path);
                    }
                }
            }
        }
        None
    }
}

#[cfg(target_os = "linux")]
fn detect_default_browser() -> Option<PathBuf> {
    let output = Command::new("xdg-settings")
        .args(["get", "default-web-browser"])
        .output()
        .ok()?;
    let default = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let exe = default.split('.').next()?;
    for dir in candidate_dirs() {
        let path = dir.join(exe);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn detect_default_browser() -> Option<PathBuf> {
    // macOS detection could use `defaultbrowser` CLI or AppleScript
    None
}

fn main() {
    println!("🔍 Detecting installed browsers...");

    // Step 1: Search known directories for browser executables
    let mut found: Vec<PathBuf> = Vec::new();
    for dir in candidate_dirs() {
        search_dir(&dir, &browser_candidates(), &mut found);
    }

    found.sort();
    found.dedup();

    // Step 2: Detect default browser
    let default_browser = {
        #[cfg(target_os = "windows")]
        { windows::detect_default_browser() }

        #[cfg(not(target_os = "windows"))]
        { detect_default_browser() }
    };

    // Step 3: Write results to browsers.txt
    let mut file = File::create("browsers.txt").expect("Failed to create browsers.txt");

    if let Some(default) = &default_browser {
        let _ = writeln!(file, "Default = {}", default.display());
    }

    for path in &found {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            let _ = writeln!(file, "{} = {}", name, path.display());
        }
    }

    println!("✅ Browsers detection complete. Results saved to browsers.txt");
    if let Some(default) = default_browser {
        println!("Default browser detected: {}", default.display());
    }
    println!("Total browsers found: {}", found.len());
}
