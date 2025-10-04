// commands/detect.rs
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use std::process::Command;
use serde::{Serialize, Deserialize};
use which::which;

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

// --- Data Structures ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Browser {
    pub name: String,
    pub path: PathBuf,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    browser: Browser,
}

// --- Public Entry Point ---

pub fn run() -> Option<Browser> {
    let config_path = get_app_config_path();

    if let Some(browser) = load_saved_browser(&config_path) {
        println!("⚡ Using saved browser: {}", browser.path.display());
        return Some(browser);
    }

    let mut detected = detect_all_browsers();
    write_outputs(&detected).ok(); // Write full list to CWD

    let selected = match detected.len() {
        0 => {
            println!("⚠️ No browsers detected. Please enter manually.");
            manual_select()
        }
        1 => {
            let b = detected.remove(0);
            println!("✅ Auto-selected: {}", b.name);
            Some(b)
        }
        _ => choose_browser_interactively(&mut detected),
    };

    if let Some(ref b) = selected {
        save_browser(&config_path, b);
    }

    selected
}

// --- Detection Logic ---

fn detect_all_browsers() -> Vec<Browser> {
    println!("🔍 Searching for installed browsers...");

    let known_browsers = vec![
        ("Google Chrome", "chrome"),
        ("Mozilla Firefox", "firefox"),
        ("Brave", "brave"),
        ("Microsoft Edge", "msedge"),
        ("Opera", "opera"),
        ("Chromium", "chromium"),
    ];

    let mut found = vec![];

    // 1. Check PATH and common installation directories
    for (name, exec) in known_browsers.iter() {
        found.extend(detect_browser(name, exec));
    }
    
    // 2. Check Windows Registry (most reliable method on Windows)
    #[cfg(target_os = "windows")]
    {
        found.extend(probe_registry());
    }

    // Deduplicate by path
    let mut unique_paths = std::collections::HashSet::new();
    let unique_found: Vec<Browser> = found.into_iter()
        .filter(|b| unique_paths.insert(b.path.clone()))
        .collect();

    if !unique_found.is_empty() {
        println!("✨ Found {} unique browsers:", unique_found.len());
        for (i, b) in unique_found.iter().enumerate() {
            let ver = b.version.clone().unwrap_or_else(|| "unknown".to_string());
            println!("  [{}] {} (version: {}, path: {})", i + 1, b.name, ver, b.path.display());
        }
    } else {
        println!("⚠️ Did not find any known browsers.");
    }

    unique_found
}

fn detect_browser(name: &str, base_exec: &str) -> Vec<Browser> {
    let mut found = vec![];
    let exec_name = get_executable_name(base_exec);

    // Check PATH
    if let Ok(path) = which(&exec_name) {
        found.push(Browser {
            name: name.to_string(),
            path: path.clone(),
            version: get_version(&path),
        });
    }

    // Check common platform-specific paths
    for candidate in common_paths(&exec_name) {
        if candidate.exists() && !found.iter().any(|b| b.path == candidate) {
            found.push(Browser {
                name: name.to_string(),
                path: candidate.clone(),
                version: get_version(&candidate),
            });
        }
    }

    found
}

#[cfg(target_os = "windows")]
fn probe_registry() -> Vec<Browser> {
    let mut result = Vec::new();

    // Paths where default browser commands are stored
    let hives = [HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER];
    const SUBKEY: &str = "SOFTWARE\\Clients\\StartMenuInternet";

    for &hive in &hives {
        if let Ok(key) = RegKey::predef(hive).open_subkey(SUBKEY) {
            for browser_name in key.enum_keys().flatten() {
                let subpath = format!("{SUBKEY}\\{browser_name}");
                if let Ok(sub) = RegKey::predef(hive).open_subkey(&subpath) {
                    if let Ok(cmd) = sub.open_subkey("shell\\open\\command") {
                        if let Ok(val) = cmd.get_value::<String, _>("") {
                            // Clean up path: remove quotes and arguments
                            let cleaned = val.split_whitespace().next().unwrap_or(&val).trim_matches('"').to_string();
                            let path = PathBuf::from(&cleaned);

                            if path.exists() {
                                let exe_name = path.file_stem()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or(browser_name.clone());

                                result.push(Browser {
                                    name: exe_name,
                                    path,
                                    version: get_version(&PathBuf::from(cleaned)),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    result
}

// --- Utility Functions ---

fn get_executable_name(base: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}

fn get_version(path: &PathBuf) -> Option<String> {
    // Note: --version flag is highly common but not universal.
    Command::new(path)
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| {
            let version_str = String::from_utf8_lossy(&output.stdout);
            // Typically version is the last word or first line. Clean it up.
            Some(version_str.lines().next().unwrap_or(&version_str).trim().to_string())
        })
}

fn common_paths(exec: &str) -> Vec<PathBuf> {
    let mut paths = vec![];

    if cfg!(target_os = "windows") {
        let pf = env::var("ProgramFiles").unwrap_or_default();
        let pf_x86 = env::var("ProgramFiles(x86)").unwrap_or_default();
        let local = env::var("LOCALAPPDATA").unwrap_or_default();
        
        let candidates = vec![
            format!("{pf}\\Google\\Chrome\\Application\\{exec}"),
            format!("{pf_x86}\\Google\\Chrome\\Application\\{exec}"),
            format!("{pf}\\Mozilla Firefox\\{exec}"),
            format!("{pf_x86}\\Microsoft\\Edge\\Application\\{exec}"),
            format!("{pf}\\BraveSoftware\\Brave-Browser\\Application\\{exec}"),
            format!("{local}\\Programs\\{exec}"),
        ];
        paths.extend(candidates.into_iter().map(PathBuf::from));
    } else if cfg!(target_os = "macos") {
        // macOS executable paths within .app bundles
        let base_name = exec.replace(".exe", "");
        paths.push(PathBuf::from(format!("/Applications/{base_name}.app/Contents/MacOS/{base_name}")));
        // Handle common variations
        if base_name == "chrome" {
             paths.push(PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"));
        }
    } else { // Linux/Unix
        paths.push(PathBuf::from(format!("/usr/bin/{exec}")));
        paths.push(PathBuf::from(format!("/usr/local/bin/{exec}")));
        paths.push(PathBuf::from(format!("/snap/bin/{exec}")));
        paths.push(PathBuf::from(format!("/opt/{exec}/{exec}")));
    }

    paths
}

// --- Interaction and Configuration Saving ---

fn choose_browser_interactively(found: &mut [Browser]) -> Option<Browser> {
    println!("\nSelect a browser:");
    println!("  [M] Manual entry");

    print!("Enter choice [1-{} or M]: ", found.len());
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return None;
    }
    let choice = input.trim();

    if choice.eq_ignore_ascii_case("m") {
        return manual_select();
    }

    if let Ok(index) = choice.parse::<usize>() {
        if index > 0 && index <= found.len() {
            let b = found[index - 1].clone();
            println!("✅ Selected: {}", b.name);
            return Some(b);
        }
    }

    println!("⚠️ Invalid choice. Retrying manual entry.");
    manual_select()
}

pub fn manual_select() -> Option<Browser> {
    println!("\n🖊️ Enter full path to browser executable:");
    print!("Path: ");
    let _ = io::stdout().flush();

    let mut path = String::new();
    if io::stdin().read_line(&mut path).is_err() {
        println!("❌ Read error.");
        return None;
    }
    let path = PathBuf::from(path.trim());

    if path.exists() {
        println!("✅ Browser added: {}", path.display());
        Some(Browser {
            name: "Custom Browser".to_string(),
            path: path.clone(),
            version: get_version(&path),
        })
    } else {
        println!("❌ Invalid path: path does not exist.");
        None
    }
}

// --- File Storage Handlers ---

/// Gets the application configuration path (~/.config/quick_tabs/config.json)
fn get_app_config_path() -> PathBuf {
    let mut config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    config_dir.push("quick_tabs");
    fs::create_dir_all(&config_dir).ok();
    config_dir.join("browser_config.json")
}

fn load_saved_browser(config_path: &Path) -> Option<Browser> {
    if config_path.exists() {
        if let Ok(data) = fs::read_to_string(config_path) {
            if let Ok(cfg) = serde_json::from_str::<Config>(&data) {
                if cfg.browser.path.exists() {
                    return Some(cfg.browser);
                }
            }
        }
    }
    None
}

fn save_browser(config_path: &Path, browser: &Browser) {
    let cfg = Config {
        browser: browser.clone(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&cfg) {
        if fs::write(config_path, json).is_ok() {
            println!("💾 Saved preferred browser to config: {}", config_path.display());
        } else {
            eprintln!("⚠️ Could not save browser config to {}", config_path.display());
        }
    }
}

/// Write JSON and text outputs to the Current Working Directory (CWD)
fn write_outputs(found: &[Browser]) -> std::io::Result<()> {
    // 1. JSON output (browsers.json)
    let json_path = PathBuf::from("browsers.json");
    let json = serde_json::to_string_pretty(found).unwrap_or_else(|_| "[]".to_string());
    fs::write(&json_path, json)?;
    println!("📄 Saved full browser list to {}", json_path.display());


    // 2. Text output (browsers.txt)
    let text_path = PathBuf::from("browsers.txt");
    let mut content = String::new();
    for b in found {
        content.push_str(&format!("{} = {}\n", b.name, b.path.display()));
    }
    fs::write(&text_path, content)?;
    println!("📄 Saved full browser list to {}", text_path.display());

    Ok(())
}
