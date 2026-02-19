
# MLVM - Multi Language Version Manager

**MLVM** is a blazing fast, cross-platform command line tool written in Rust that allows you to manage multiple versions of your favorite programming languages locally.

If you find this project useful, please drop a ✨ star on the repo! It means a lot :)

https://github.com/user-attachments/assets/79176462-d41b-4423-b2e8-4128d3f7c6f8

## Features

- **Blazing Fast:** Written in Rust for maximum performance.
- **Single Binary:** No complex dependencies, just one binary to rule them all.
- **Secure & Clean:** Uses symlinks instead of messy environment variable hacks.
- **Cross-Platform:** Works on Windows, macOS, and Linux.

## Supported Languages

Currently, `mlvm` supports the following languages (with more coming soon!):

- **Node.js**
- **Python** (Standalone builds)
- **Go**
- **Bun**

## Installation

### Build From Source

1. Clone the repository:
   ```bash
   git clone https://github.com/sanchit-4/mlvm.git
   cd mlvm
   ```

2. Build the project using Cargo:
    ``` bash
    cargo build --release
    ```
The binary will be located at ./target/release/mlvm (or mlvm.exe on Windows).

### Initial Setup (Important!)

For mlvm to work, you must add the language directories to your system PATH.

Windows (PowerShell):
Add the following to your $PROFILE (or Environment Variables via System Settings):
``` Powershell
$env:PATH = "C:\Users\YOUR_USER\.mlvm\node\current;" + $env:PATH
$env:PATH = "C:\Users\YOUR_USER\.mlvm\python\current;" + $env:PATH
$env:PATH = "C:\Users\YOUR_USER\.mlvm\go\current\bin;" + $env:PATH
$env:PATH = "C:\Users\YOUR_USER\.mlvm\bun\current;" + $env:PATH
```

Linux / macOS (.bashrc or .zshrc):
```Bash

export PATH="$HOME/.mlvm/node/current/bin:$PATH"
export PATH="$HOME/.mlvm/python/current/bin:$PATH"
export PATH="$HOME/.mlvm/go/current/bin:$PATH"
export PATH="$HOME/.mlvm/bun/current/bin:$PATH"
```

Restart your terminal after updating the path.

## Usage

The general syntax is:
```Bash

mlvm <language> <command> [args]
```
Node.js
```code Bash
mlvm node list-remote       # List available online versions
mlvm node install 18.17.0   # Install a specific version
mlvm node use 18.17.0       # Switch to this version
mlvm node list              # List installed versions
```
Python
```code Bash
mlvm python list-remote     # List available standalone versions
mlvm python install 3.10.11 # Install Python 3.10.11
mlvm python use 3.10.11     # Switch to Python 3.10.11
```
Go
```code Bash
mlvm go list-remote         # List available Go versions
mlvm go install 1.21.5      # Install Go 1.21.5
mlvm go use 1.21.5          # Switch to Go 1.21.5
```
Bun
```code Bash
mlvm bun list-remote        # List Bun tags
mlvm bun install 1.0.25     # Install Bun
mlvm bun use 1.0.25         # Switch Bun version
```

## Tech Stack

    Language: Rust 🦀

    CLI Framework: Clap

    HTTP Client: Reqwest

    Async Runtime: Tokio

    Decompression: Zip, Tar, Flate2, Zstd

## Contributing

Contributions are welcome! Please read our Code of Conduct before contributing.

    Fork the repository

    Create your feature branch (git checkout -b feature/amazing-feature)

    Commit your changes (git commit -m 'feat: add amazing feature')

    Push to the branch (git push origin feature/amazing-feature)

    Open a Pull Request

## License

Distributed under the MIT License. See LICENSE for more information.

Made with ❤️ by Sanchit   
