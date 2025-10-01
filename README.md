# 🚀 Quick Tabs

**A blazing-fast, cross-platform Rust utility to open multiple browser tabs in a single window.**

![GitHub release](https://img.shields.io/github/v/release/Rakllan/Quick_Tabs)
![License](https://img.shields.io/github/license/Rakllan/Quick_Tabs)
[![Hacktoberfest](https://img.shields.io/badge/Hacktoberfest-friendly-blueviolet)](https://hacktoberfest.com/)

---

## 🌐 What Is Quick Tabs?

**Quick Tabs** is a lightweight command-line tool built in **Rust** that helps you open multiple URLs at once in a single browser window perfect for productivity, research, or daily startup routines.

Whether you're on **Windows, macOS, or Linux**, Quick Tabs automatically detects installed browsers and launches your links in **private/incognito mode** for a clean session.

---

## ✨ Features

- 🔍 **Smart Browser Detection** — Supports Chrome, Brave, Firefox, Edge, Opera, Chromium, and more.
- 🧠 **Default Browser Recognition** — Automatically identifies your system’s default browser.
- 🪟 **Single Window Launch** — Opens all URLs in one incognito/private window.
- 📁 **File-Based Input** — Reads URLs from a simple `links.txt` file.
- ⚡ **Fast Performance** — Parallel detection for snappy startup.
- 📝 **Browser Path Export** — Saves detected browser paths to `browsers.txt`.

---

## 🛠 Installation

### Prerequisites

Ensure you have **Rust** and **Cargo** installed. Get them from [rustup.rs](https://rustup.rs).

### Build Steps

```bash
git clone https://github.com/Rakllan/Quick_Tabs.git
cd Quick_Tabs
cargo build --release
