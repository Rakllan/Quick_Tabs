mod commands;

use crate::commands::links::{self, LinkConfig};
use crate::commands::aliases::AliasConfig;
use crate::commands::detect::{Browser, run as detect_browsers};

use std::path::PathBuf;
use std::env;

fn main() { let args: Vec<String> = env::args().collect(); 
    if args.len() < 2 { print_help(); return; } 
    // Detect or load browser
     let browser = detect_browsers().expect("No browser available");
   //Config paths
    let home = env::var("HOME").unwrap_or(".".to_string());
     let link_path = PathBuf::from(format!("{}/.quick_tabs_links.json", home));
      let alias_path = PathBuf::from(format!("{}/.quick_tabs_aliases.json", home)); 
      let mut link_cfg = LinkConfig::load(&link_path);
 let mut alias_cfg = AliasConfig::load(&alias_path);

let mut link_cfg = LinkConfig::load(&link_path);
    let mut alias_cfg = AliasConfig::load(&alias_path);

    match args[1].as_str() {
    "launch" => {
        if args.len() < 3 {
            println!("⚠️ Please provide a tag or URL to launch");
            return;
        }
        let target = &args[2];
        let url = alias_cfg.resolve(target)
            .or_else(|| link_cfg.get_url(target))
            .unwrap_or_else(|| target.to_string());
        links::launch_link(&browser, &url);
    },
    "add-link" => {
        if args.len() < 4 {
            println!("⚠️ Usage: add-link <tag> <url>");
            return;
        }
        link_cfg.add_link(args[2].clone(), args[3].clone());
        link_cfg.save(&link_path);
        println!("✅ Link saved!");
    },
    "add-alias" => {
        if args.len() < 4 {
            println!("⚠️ Usage: add-alias <tag> <url>");
            return;
        }
        alias_cfg.add_alias(args[2].clone(), args[3].clone());
        alias_cfg.save(&alias_path);
        println!("✅ Alias saved!");
    },
    "remove-link" => {
        if args.len() < 3 {
            println!("⚠️ Usage: remove-link <tag>");
            return;
        }
        if link_cfg.remove_link(&args[2]) {
            link_cfg.save(&link_path);
            println!("✅ Link removed!");
        } else {
            println!("⚠️ Link not found.");
        }
    },
    "remove-alias" => {
        if args.len() < 3 {
            println!("⚠️ Usage: remove-alias <tag>");
            return;
        }
        if alias_cfg.remove_alias(&args[2]) {
            alias_cfg.save(&alias_path);
            println!("✅ Alias removed!");
        } else {
            println!("⚠️ Alias not found.");
        }
    },
    "list-links" => link_cfg.list(),
    "open-all-links" => link_cfg.open_all(&browser),
    "open-all-aliases" => alias_cfg.open_all(&browser),
    _ => print_help(),
}
}

fn print_help() {
    println!("Quick Tabs CLI - commands:");
    println!("  launch <tag|url>         Launch a link in the detected browser");
    println!("  add-link <tag> <url>     Add a link with a tag");
    println!("  add-alias <tag> <url>    Add a shortcut/alias");
    println!("  remove-link <tag>        Remove a saved link");
    println!("  remove-alias <tag>       Remove a saved alias");
    println!("  list-links               List saved links");
    println!("  open-all-links           Open all saved links");
    println!("  open-all-aliases         Open all aliases");
}
