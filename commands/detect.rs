use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use std::process::Command;
use serde::{Serialize, Deserialize};
use which::which;

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

pub fn run() -> Option<Browser> {
    let config_path = get_config_path();

    if let Some(browser) = load_saved_browser(&config_path) {
        println!("⚡ Using saved browser: {}", browser.path.display());
        return Some(browser);
    }

    let mut detected = detect_all_browsers();

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

fn detect_all_browsers() -> Vec<Browser> {
    println!("🔍 Searching for installed browsers...\n");

    let browsers = vec![
        ("Google Chrome", "chrome"),
        ("Mozilla Firefox", "firefox"),
        ("Brave", "brave"),
        ("Microsoft Edge", "msedge"),
        ("Opera", "opera"),
        ("Chromium", "chromium"),
        ("Safari", "safari"),
    ];

    let mut found = vec![];

    for (name, exec) in browsers {
        found.extend(detect_browser(name, exec));
    }

    if !found.is_empty() {
        println!("✨ Found {} browsers:", found.len());
        for (i, b) in found.iter().enumerate() {
            let ver = b.version.clone().unwrap_or_else(|| "unknown".to_string());
            println!("  [{}] {} (version: {}, path: {})", i + 1, b.name, ver, b.path.display());
        }
    }

    found
}

fn detect_browser(name: &str, exec: &str) -> Vec<Browser> {
    let mut found = vec![];

    let exec_name = get_executable_name(exec);

    if let Ok(path) = which(&exec_name) {
        found.push(Browser {
            name: name.to_string(),
            path: path.clone(),
            version: get_version(&path),
        });
    }

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

fn get_executable_name(base: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}

fn get_version(path: &PathBuf) -> Option<String> {
    Command::new(path)
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| {
            let version_str = String::from_utf8_lossy(&output.stdout);
            Some(version_str.trim().to_string())
        })
}

fn choose_browser_interactively(found: &mut [Browser]) -> Option<Browser> {
    println!("\nSelect a browser:");
    println!("  [M] Manual entry");

    print!("Enter choice [1-{} or M]: ", found.len());
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
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

    println!("⚠️ Invalid choice. Falling back to manual entry.");
    manual_select()
}

pub fn manual_select() -> Option<Browser> {
    println!("\n🖊️ Enter full path to browser executable:");
    print!("Path: ");
    io::stdout().flush().unwrap();

    let mut path = String::new();
    io::stdin().read_line(&mut path).unwrap();
    let path = PathBuf::from(path.trim());

    if path.exists() {
        println!("✅ Browser added: {}", path.display());
        Some(Browser {
            name: "Custom Browser".to_string(),
            path: path.clone(),
            version: get_version(&path),
        })
    } else {
        println!("❌ Invalid path.");
        None
    }
}

fn common_paths(exec: &str) -> Vec<PathBuf> {
    let mut paths = vec![];

    if cfg!(target_os = "windows") {
        let pf = env::var("ProgramFiles").unwrap_or_default();
        let pf_x86 = env::var("ProgramFiles(x86)").unwrap_or_default();
        let local = env::var("LOCALAPPDATA").unwrap_or_default();
        let candidates = vec![
            format!("{pf}\\{exec}\\Application\\{exec}"),
            format!("{pf_x86}\\{exec}\\Application\\{exec}"),
            format!("{local}\\Programs\\{exec}\\{exec}"),
        ];
        paths.extend(candidates.into_iter().map(PathBuf::from));
    } else if cfg!(target_os = "macos") {
        paths.push(PathBuf::from(format!("/Applications/{}.app/Contents/MacOS/{}", exec, exec)));
        paths.push(PathBuf::from(format!("/System/Applications/{}.app/Contents/MacOS/{}", exec, exec)));
    } else {
        paths.push(PathBuf::from(format!("/usr/bin/{exec}")));
        paths.push(PathBuf::from(format!("/usr/local/bin/{exec}")));
        paths.push(PathBuf::from(format!("/snap/bin/{exec}")));
        paths.push(PathBuf::from(format!("/opt/{exec}/{exec}")));
    }

    paths
}

fn get_config_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        let local = env::var("LOCALAPPDATA").unwrap_or(".".to_string());
        let folder = Path::new(&local).join("quick_tabs");
        fs::create_dir_all(&folder).ok();
        folder.join("config.json")
    } else {
        let home = env::var("HOME").unwrap_or(".".to_string());
        Path::new(&home).join(".quick_tabs.json")
    }
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
        fs::write(config_path, json).ok();
    }
    println!("💾 Saved browser to config: {}", config_path.display());
}
