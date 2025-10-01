use std::fs;
use dirs::config_dir;

pub fn run() {
    let mut path = config_dir().unwrap();
    path.push("quick_tabs.json");

    if !path.exists() {
        println!("⚠️ No saved data yet.");
        return;
    }

    let data: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();

    println!("🌐 Browsers:");
    for b in data["browsers"].as_array().unwrap() {
        println!(" - {}", b);
    }

    println!("\n🔗 Links:");
    for l in data["links"].as_array().unwrap() {
        println!(" - {}", l);
    }
}
