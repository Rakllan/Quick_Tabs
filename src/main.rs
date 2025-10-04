mod commands;

use crate::commands::links::{LinkConfig, launch_link, LaunchMode};
use crate::commands::aliases::AliasConfig;
use crate::commands::detect::{run as detect_browsers, Browser};

use std::path::{PathBuf, Path};
use std::env;
use clap::{Parser, Subcommand, CommandFactory}; // <-- ADDED CommandFactory

// --- CLI Structure using Clap ---

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Launch a tag or URL in the detected browser
    Launch {
        target: String,
        /// Open the link in incognito/private mode
        #[arg(short, long)]
        incognito: bool,
    },
    /// Add a new link tag
    AddLink {
        tag: String,
        url: String,
    },
    /// Add a new alias shortcut
    AddAlias {
        tag: String,
        url: String,
    },
    /// Remove a saved link
    RemoveLink {
        tag: String,
    },
    /// Remove a saved alias
    RemoveAlias {
        tag: String,
    },
    /// List saved links and aliases
    ListLinks,
    /// Open all saved links (can use --incognito)
    OpenAllLinks {
        /// Open links in incognito/private mode
        #[arg(short, long)]
        incognito: bool,
    },
    /// Open all saved aliases (can use --incognito)
    OpenAllAliases {
        /// Open aliases in incognito/private mode
        #[arg(short, long)]
        incognito: bool,
    },
    /// Re-detect and select the preferred browser
    Detect,
    /// Print help information
    Help,
}

// --- Main Execution ---

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // 1. Config paths setup
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let link_path = PathBuf::from(format!("{}/.quick_tabs_links.json", home));
    let alias_path = PathBuf::from(format!("{}/.quick_tabs_aliases.json", home));
    
    // 2. Browser Detection (only required for launch/open commands)
    let browser_result = detect_browsers();

    match cli.command {
        // --- Commands requiring Config & Browser ---
        Commands::Launch { target, incognito } => {
            let browser = get_browser_or_exit(browser_result)?;
            let link_cfg = LinkConfig::load(&link_path);
            let alias_cfg = AliasConfig::load(&alias_path);
            
            let mode = if incognito { LaunchMode::Private } else { LaunchMode::Normal };

            let url = alias_cfg.resolve(&target)
                .or_else(|| link_cfg.get_url(&target))
                .unwrap_or_else(|| target);

            launch_link(&browser, &url, mode);
        },

        // --- Commands requiring Config only ---
        Commands::AddLink { tag, url } => {
            let mut link_cfg = LinkConfig::load(&link_path);
            link_cfg.add_link(tag, url);
            link_cfg.save(&link_path)?;
            println!("✅ Link saved!");
        },
        Commands::AddAlias { tag, url } => {
            let mut alias_cfg = AliasConfig::load(&alias_path);
            alias_cfg.add_alias(tag, url);
            alias_cfg.save(&alias_path)?;
            println!("✅ Alias saved!");
        },
        Commands::RemoveLink { tag } => {
            let mut link_cfg = LinkConfig::load(&link_path);
            if link_cfg.remove_link(&tag) {
                link_cfg.save(&link_path)?;
                println!("✅ Link removed!");
            } else {
                println!("⚠️ Link tag '{}' not found.", tag);
            }
        },
        Commands::RemoveAlias { tag } => {
            let mut alias_cfg = AliasConfig::load(&alias_path);
            if alias_cfg.remove_alias(&tag) {
                alias_cfg.save(&alias_path)?;
                println!("✅ Alias removed!");
            } else {
                println!("⚠️ Alias tag '{}' not found.", tag);
            }
        },
        Commands::ListLinks => {
            LinkConfig::load(&link_path).list();
            AliasConfig::load(&alias_path).list();
        },
        
        // --- Commands requiring Config & Browser, and Incognito flag ---
        Commands::OpenAllLinks { incognito } => {
            let browser = get_browser_or_exit(browser_result)?;
            let link_cfg = LinkConfig::load(&link_path);
            let mode = if incognito { LaunchMode::Private } else { LaunchMode::Normal };
            link_cfg.open_all(&browser, mode);
        },
        Commands::OpenAllAliases { incognito } => {
            let browser = get_browser_or_exit(browser_result)?;
            let alias_cfg = AliasConfig::load(&alias_path);
            let mode = if incognito { LaunchMode::Private } else { LaunchMode::Normal };
            alias_cfg.open_all(&browser, mode);
        },

        // --- Browser Commands ---
        Commands::Detect => {
            // detect_browsers returns Option<Browser>, not Result. We ignore the return value.
            let _ = detect_browsers(); 
        },
        Commands::Help => {
            Cli::command().print_help()?;
        }
    }

    Ok(())
}

fn get_browser_or_exit(browser_result: Option<Browser>) -> Result<Browser, Box<dyn std::error::Error>> {
    match browser_result {
        Some(b) => Ok(b),
        None => {
            eprintln!("❌ Error: No browser configured. Run 'quick_tabs detect' or set manually.");
            // We use standard library exit here since we cannot proceed without a browser
            std::process::exit(1); 
        }
    }
}
