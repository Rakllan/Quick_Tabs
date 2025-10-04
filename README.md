# 🚀 Quick Tabs

**A blazing-fast, cross-platform Rust utility to open multiple browser tabs in a single window.**  
![GitHub release](https://img.shields.io/github/v/release/Rakllan/Quick_Tabs)  
![License](https://img.shields.io/github/license/Rakllan/Quick_Tabs?style=flat)
[![Hacktoberfest](https://img.shields.io/badge/Hacktoberfest-friendly-blueviolet)](https://hacktoberfest.com/)

---

## 🌐 What Is Quick Tabs?

Quick Tabs is a lightweight command-line tool built in Rust that opens multiple URLs at once in a single browser window.  
Works on **Windows, macOS, and Linux** with automatic browser detection.

---

## ✨ Features

- Detects Chrome, Firefox, Brave, Edge, Opera, Chromium, and more  
- Opens URLs in one window  
- Reads links from `links.txt` or saved aliases  
- Saves detected browser paths to `browsers.txt` and `browsers.json`  
- Fast detection using parallel processing

---

## 🛠 Installation

Make sure you have Rust and Cargo installed: [rustup.rs](https://rustup.rs)

```bash
git clone https://github.com/Rakllan/Quick_Tabs.git
cd Quick_Tabs
cargo build --release
```

## 💻 CLI Usage

### Commands

| Command              | Description                                         |
|----------------------|-----------------------------------------------------|
| `launch <tag url>`   | Add a link with a tag                              |
| `add-link <tag> <url>`  | Add a link with a tag                            |
| `add-alias <tag> <url>` | Add a shortcut/alias                            |
| `remove-link <tag>`  | Remove a saved link                                |
| `remove-alias <tag>` | Remove a saved alias                               |
| `list-links`         | List all saved links                               |
| `open-all-links`     | Open all saved links                             |
| `open-all-aliases`   | Open all saved aliases                           |

### Examples

```bash
quick_tabs launch google
quick_tabs add-link rust https://www.rust-lang.org
quick_tabs add-alias r https://www.rust-lang.org
quick_tabs remove-link rust
quick_tabs remove-alias r
quick_tabs list-links
quick_tabs open-all-links
quick_tabs open-all-aliases
```

## 📂 Configuration Files

- `~/.quick_tabs_links.json` — saved links  
- `~/.quick_tabs_aliases.json` — saved aliases  
- `browsers.txt` — detected browser paths  
- `browsers.json` — JSON list of detected browsers  

*These files are created automatically on first use.*

## 🤝 Contributing

We welcome contributions!

* This repository is **Hacktoberfest-friendly**!
* Please see the **[CONTRIBUTING.md](CONTRIBUTING.md)** file for guidelines on setting up your development environment, submitting pull requests, and coding standards.

---

## 📜 License

Quick Tabs is distributed under the **GNU GPL License**. See the **[LICENSE](LICENSE)** file for more details.
