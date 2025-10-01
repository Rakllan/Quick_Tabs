use std::fs;
use std::path::PathBuf;
use dirs::config_dir;

fn storage_file() -> PathBuf {
    let mut path = config_dir().unwrap();
    path.push("quick_tabs.json");
    path
}

pub fn run(kind: String, value: String) {
    let file = storage_file();
    let mut data: serde_json::Value = if file.exists() {
        serde_json::from_str(&fs::read_to_string(&file).unwrap()).unwrap()
    } else {
        serde_json::json!({ "browsers": [], "links": [] })
    };

    match kind.as_str() {
        "browser" => {
            data["browsers"].as_array_mut().unwrap().push(serde_json::json!(value));
            println!("✅ Browser added: {}", value);
        }
        "link" => {
            data["links"].as_array_mut().unwrap().push(serde_json::json!(value));
            println!("✅ Link added: {}", value);
        }
        _ => println!("⚠️ Invalid type. Use `browser` or `link`."),
    }

    fs::write(&file, serde_json::to_string_pretty(&data).unwrap()).unwrap();
}
