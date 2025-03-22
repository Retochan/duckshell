# DuckShell (d-sh)

**DuckShell** is a lightweight command-line shell written in Rust, featuring plugin support, autocompletion, and a built-in plugin manager called `dupi`. Inspired by the simplicity and functionality of classic shells, it adds modern capabilities like installing plugins from `.pfds` files or URLs.

Quack quack! 🦆

## Features
- **REPL**: Interactive command-line interface supporting system commands and plugins.
- **Plugins**: Install and manage plugins via `.pfds` (ZIP archives with `manifest.json` and scripts).
- **dupi**: Built-in plugin manager with commands:
  - `dupi -i <plugin|.pfds>` — Install a plugin.
  - `dupi -re <plugin>` — Remove a plugin.
  - `dupi -ls` — List installed plugins.
  - `dupi -ud` — Update plugins (for those installed via URL).
  - `dupi -d <url>` — Download and install a plugin from a URL.
- **Autocompletion**: Suggestions for built-in commands and plugins (press Tab).
- **Cross-platform**: Works on Unix systems (Linux, macOS), with potential Windows support.
- **Other features in DuckShell Plus**

## Installation

### Requirements
- Rust (latest stable version recommended, install via [rustup](https://rustup.rs/)).
- OpenSSL with development files:
  - Fedora: `sudo dnf install openssl-devel pkg-config`
  - Ubuntu/Debian: `sudo apt install libssl-dev pkg-config`

### Build
1. Clone the repository:
   ```bash
   git clone https://github.com/Retochan/duckshell.git
   cd duckshell
   cargo run
   ```
