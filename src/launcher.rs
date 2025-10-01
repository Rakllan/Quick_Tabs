use crate::find_browsers::{detect_all, Browser};
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    preferred: Option<String>, // preferred browser path
}

impl Default for Config {
    fn default() -> Self {
        Self { preferred: None }
    }
}

const CONFIG_FILE: &str = "quick_tabs_config.json";
const LINKS_FILE: &str = "links.txt";

fn load_config() -> Config {
    match fs::read_to_string(CONFIG_FILE) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

fn save_config(cfg: &Config) -> io::Result<()> {
    let json = serde_json::to_string_pretty(cfg).unwrap();
    fs::write(CONFIG_FILE, json)
}

fn load_links() -> Vec<String> {
    match fs::read_to_string(LINKS_FILE) {
        Ok(s) => s.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect(),
        Err(_) => vec![],
    }
}

fn save_link(link: &str) -> io::Result<()> {
    let mut file = OpenOptions::new().append(true).create(true).open(LINKS_FILE)?;
    writeln!(file, "{}", link)?;
    Ok(())
}

fn choose_browser_interactive(found: &Vec<Browser>, cfg: &mut Config) -> Option<PathBuf> {
    println!("\nDetected browsers (choose number to set as preferred):");
    for (i, b) in found.iter().enumerate() {
        println!("  {}. {} -> {}", i + 1, b.name, b.path);
    }
    println!("  M. Manually add browser path");
    println!("  K. Keep current preference");
    print!("Choice: ");
    let _ = io::stdout().flush();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok()?;
    let c = choice.trim();

    if c.eq_ignore_ascii_case("M") {
        println!("Enter full path to browser executable:");
        let mut p = String::new();
        io::stdin().read_line(&mut p).ok()?;
        let p = p.trim().to_string();
        if !p.is_empty() {
            cfg.preferred = Some(p.clone());
            let _ = save_config(cfg);
            return Some(PathBuf::from(p));
        }
    } else if c.eq_ignore_ascii_case("K") {
        if let Some(pref) = &cfg.preferred {
            return Some(PathBuf::from(pref));
        }
    } else if let Ok(idx) = c.parse::<usize>() {
        if idx >= 1 && idx <= found.len() {
            let sel = &found[idx - 1];
            cfg.preferred = Some(sel.path.clone());
            let _ = save_config(cfg);
            return Some(PathBuf::from(&sel.path));
        }
    }
    None
}

fn open_links_in_private(browser: &PathBuf, links: &[String]) -> io::Result<()> {
    if links.is_empty() {
        println!("No links to open.");
        return Ok(());
    }

    let exe_lower = browser.to_string_lossy().to_lowercase();
    // pick flags by known browser families
    let mut args_for_private: Vec<&str> = vec!["--incognito"]; // default for chromium family

    if exe_lower.contains("firefox") {
        args_for_private = vec!["-private-window"];
    } else if exe_lower.contains("msedge") {
        args_for_private = vec!["--inprivate"];
    } else if exe_lower.contains("brave") || exe_lower.contains("chrome") || exe_lower.contains("chromium") {
        args_for_private = vec!["--incognito"];
    }

    let mut cmd = Command::new(browser);
    for a in &args_for_private { cmd.arg(a); }
    for link in links { cmd.arg(link); }

    let _ = cmd.spawn()?;
    Ok(())
}

/// Main interactive launcher
pub fn run_launcher() -> Result<(), Box<dyn std::error::Error>> {
    println!("Quick Tabs — intelligent launcher");
    println!("Detecting browsers (fast)...");

    // detect
    let mut found = detect_all();
    println!("Detected {} browser(s).", found.len());

    // show summary
    for (i, b) in found.iter().enumerate() {
        println!("  {}. {} -> {}", i + 1, b.name, b.path);
    }

    // load config
    let mut cfg = load_config();

    // allow user to pick or add
    let chosen = choose_browser_interactive(&found, &mut cfg);

    if chosen.is_none() {
        println!("No browser selected. You can still add links, they will be saved.");
    }

    // main menu loop
    loop {
        println!("\nMenu:");
        println!("  1) Add a link");
        println!("  2) List saved links");
        println!("  3) Open saved links in preferred browser");
        println!("  4) Re-detect browsers");
        println!("  5) Choose preferred browser");
        println!("  6) Quit");

        print!("Select: ");
        io::stdout().flush()?;
        let mut sel = String::new();
        io::stdin().read_line(&mut sel)?;
        match sel.trim() {
            "1" => {
                print!("Enter URL to add: ");
                io::stdout().flush()?;
                let mut url = String::new();
                io::stdin().read_line(&mut url)?;
                let url = url.trim();
                if !url.is_empty() {
                    save_link(url)?;
                    println!("Saved: {}", url);
                }
            }
            "2" => {
                let links = load_links();
                if links.is_empty() {
                    println!("No links saved.");
                } else {
                    println!("Saved links:");
                    for (i, l) in links.iter().enumerate() {
                        println!("  {}. {}", i + 1, l);
                    }
                }
            }
            "3" => {
                let links = load_links();
                if links.is_empty() {
                    println!("No links to open.");
                } else {
                    // pick browser: preferred in config or prompt
                    let browser_path = if let Some(pref) = &cfg.preferred {
                        PathBuf::from(pref)
                    } else if let Some(chosen) = &chosen {
                        chosen.clone()
                    } else if !found.is_empty() {
                        PathBuf::from(&found[0].path)
                    } else {
                        println!("No browser configured or detected.");
                        continue;
                    };

                    match open_links_in_private(&browser_path, &links) {
                        Ok(_) => println!("Opened {} links in {}", links.len(), browser_path.display()),
                        Err(e) => println!("Failed to open links: {}", e),
                    }
                }
            }
            "4" => {
                println!("Re-detecting (fast)...");
                found = detect_all();
                println!("Detected {} browser(s).", found.len());
                for (i, b) in found.iter().enumerate() {
                    println!("  {}. {} -> {}", i + 1, b.name, b.path);
                }
            }
            "5" => {
                if let Some(p) = choose_browser_interactive(&found, &mut cfg) {
                    println!("Preferred browser set to {}", p.display());
                } else {
                    println!("No change to preferred browser.");
                }
            }
            "6" => {
                println!("Goodbye!");
                break;
            }
            other => println!("Unknown option: {}", other),
        }
    }

    Ok(())
}
