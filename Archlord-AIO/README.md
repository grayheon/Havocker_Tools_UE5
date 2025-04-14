# 🛠 Archlord Universal Tool

This Rust-based workspace provides a modular system to extract, decrypt, convert, validate, and analyze data from an Archlord client installation.

## 🔧 Tools Overview

| App           | Description |
|---------------|-------------|
| `core_main`   | Central coordinator that runs all tools in sequence or in parallel |
| `extractor`   | Copies and decrypts files from the source directory |
| `txd_converter` | Converts `.txd` files to `.dds` and `.png` |
| `minimap`     | Builds a full worldmap image from tile files |
| `obj_checker` | Validates object templates and generates CSV/proof data |
| `dff_scanner` | Scans `.dff` files for embedded texture names |
| `txd_viewer`  | Placeholder or visual tool for `.txd` files |

## 🧱 Project Structure

- `/apps` – All binaries, one for each logical step
- `/libs/shared_utils` – Shared logic, file handlers, formats, etc.
- `/tools` – External tools (e.g. `texconv.exe`)
- `/build.rs` – Copies `texconv.exe` to target folder
- `/config.ini` – Project configuration

## 📦 How to Run

Build the full workspace:
```bash
cargo build --release
```

Run the core controller:
```bash
cargo run -p core_main
```
