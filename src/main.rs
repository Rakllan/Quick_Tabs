use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::Command;
use std::path::Path;

/// Read list of links from links.txt
fn read_links() -> Vec<String> {
    let file = File::open("links.txt").expect("Could not open links.txt");
    let reader = BufReader::new(file);
    reader
        .lines()
        .filter_map(Result::ok)
        .filter(|line| !line.trim().is_empty())
        .collect()
}

/// Read browsers.txt and return first browser path
fn read_browser() -> Option<String> {
    let path = Path::new("browsers.txt");
    if !path.exists() {
        eprintln!("browsers.txt not found! Run `cargo run --bin find_browsers` first.");
        return None;
    }

    let file = File::open(path).expect("Could not open browsers.txt");
    let reader = BufReader::new(file);

    for line in reader.lines().flatten() {
        if let Some((_, exe_path)) = line.split_once(" = ") {
            return Some(exe_path.to_string());
        }
    }
    None
}

fn main() {
    let links = read_links();
    if links.is_empty() {
        eprintln!("No links found in links.txt");
        return;
    }

    let browser = match read_browser() {
        Some(path) => path,
        None => {
            eprintln!("No browser found in browsers.txt");
            return;
        }
    };

    println!("Launching {} with {} tabs...", browser, links.len());

    let mut cmd = Command::new(&browser);

    // Arguments for private/incognito mode
    if browser.to_lowercase().contains("firefox") {
        cmd.arg("-private-window");
    } else {
        cmd.arg("--incognito");
    }

    cmd.args(&links);

    match cmd.spawn() {
        Ok(_) => println!("✅ Browser launched successfully."),
        Err(e) => eprintln!("❌ Failed to launch browser: {}", e),
    }
}
