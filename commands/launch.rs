use std::process::Command;

pub fn run(browser: String, url: String) {
    println!("🚀 Launching {browser} with {url}");

    if let Err(e) = Command::new(browser).arg(url).spawn() {
        println!("⚠️ Failed to launch: {}", e);
    }
}
