# Quick Tabs

**A Fast, Cross-Platform Rust Utility to Open Multiple Browser Tabs in a Single Window.**

![GitHub release (latest by date)](https://img.shields.io/github/v/release/Rakllan/Quick_Tabs)
![License](https://img.shields.io/github/license/Rakllan/Quick_Tabs)
[![Hacktoberfest](https://img.shields.io/badge/Hacktoberfest-friendly-blueviolet)](https://hacktoberfest.com/)

---

## 🚀 Overview

**Quick Tabs** is a **cross-platform utility** written in **Rust** designed to simplify opening numerous URLs simultaneously. It can quickly detect installed browsers on your system or even a mounted system and launch multiple links concurrently in a single, dedicated window.

### Key Features

* **Broad Browser Detection:** Detects and supports a wide range of browsers, including **Chrome, Brave, Edge, Firefox, Opera, Chromium**, and more.
* **Default Browser Check:** Automatically identifies the system's default browser.
* **Single Window Launch:** Opens all URLs in **one browser window (currently limited to private/incognito mode).**
* **Cross-Platform:** Full support for **Windows, Linux, and macOS**.
* **Input File:** Reads URLs easily from a user-defined `links.txt` file.
* **Performance:** Utilizes parallel checks for **fast browser detection**.
* **Configuration Output:** Saves detected browser paths to a `browsers.txt` file for easy reference.

---

## ⬇️ Installation

### Prerequisites

You must have **Rust and Cargo** installed on your system to build the utility.

### Steps

1.  **Clone the Repository**
    ```bash
    git clone https://github.com/Rakllan/Quick_Tabs.git
    cd Quick_Tabs
    ```

2.  **Build the Project**
    Use Cargo to build the project in release mode for optimization.

    ```bash
    cargo build --release
    ```

    The final executable will be located in the `target/release/` directory.

---

## 💻 Usage

To use Quick Tabs, you need a list of URLs, and then you simply run the executable.

1.  **Prepare Your Links**
    Create a file named **`links.txt`** in the root directory of the project. Each URL must be on its own line.

    ```text
    # links.txt example
    [https://www.google.com](https://www.google.com)
    [https://github.com/Rakllan/Quick_Tabs](https://github.com/Rakllan/Quick_Tabs)
    [https://doc.rust-lang.org/](https://doc.rust-lang.org/)
    ```

2.  **Run the Utility**
    Execute the compiled binary (`open_private`) using Cargo.

    ```bash
    cargo run --bin open_private
    ```

    The program will first detect your installed browsers. It will then prompt you to select a browser and open all the links from `links.txt` in a single **private/incognito window** of your choice.

---

## 🤝 Contributing

We welcome contributions!

* This repository is **Hacktoberfest-friendly**!
* Please see the **[CONTRIBUTING.md](CONTRIBUTING.md)** file for guidelines on setting up your development environment, submitting pull requests, and coding standards.

---

## 📜 License

Quick Tabs is distributed under the **MIT License**. See the **[LICENSE](LICENSE)** file for more details.
