# 🛠 Archlord Universal Tool

This repository is a Rust workspace that provides a modular toolkit to extract, decrypt, convert, validate, and analyze data from an **Archlord** client installation.

> ⚠️ **Disclaimer**  
> This project is a fan-made reverse engineering / tooling project. It is not affiliated with, endorsed by, or supported by any rights holder of Archlord.  
> Make sure you own a legal copy of the game and comply with local laws and the game’s EULA when using this tool.

---

## ✨ Features

- **End-to-end pipeline** for processing an Archlord client directory
    - Extract and decrypt original client files
    - Convert proprietary texture formats (`.txd`) to common formats (`.dds`, `.png`)
    - Build large **minimap / worldmap** images from tile data
    - Scan and validate object templates and 3D resources
- **Modular architecture**: each step is a separate binary in `apps/`
- **Shared utilities** in `libs/shared_utils` (file I/O helpers, formats, logging, etc.)
- **Scriptable**: run all steps sequentially (or partially) via the `core_main` controller

---

## 🔧 Tools Overview

| App           | Description |
|---------------|-------------|
| `core_main`   | Central controller that orchestrates all tools in sequence (or selected subsets) |
| `extractor`   | Copies and decrypts files from the Archlord client source directory |
| `txd_converter` | Converts `.txd` files to `.dds` and `.png` (requires `texconv.exe` in `tools/`) |
| `minimap`     | Builds a combined worldmap image from minimap tile files |
| `obj_checker` | Validates object templates and generates CSV / proof data |
| `dff_scanner` | Scans `.dff` files for embedded texture names / references |
| `txd_viewer`  | (WIP) Visual / interactive viewer for `.txd` files |

If you only need a specific step (e.g. converting textures), you can run the corresponding binary directly.

---

## 🧱 Project Structure

At the top level, this repository is a Cargo workspace:

- `/apps` – All binaries, one for each logical step in the pipeline:
    - `/apps/core_main` – High-level controller
    - `/apps/extractor` – Extraction & decryption
    - `/apps/txd_converter` – Texture conversion
    - `/apps/minimap` – Minimap / worldmap generation
    - `/apps/obj_checker` – Object template checks
    - `/apps/dff_scanner` – DFF analysis
    - `/apps/txd_viewer` – Viewer (experimental / WIP)
- `/libs/shared_utils` – Shared logic, file handlers, common data structures
- `/tools` – External tools (e.g. `texconv.exe` for texture conversion)
- `/build.rs` – Copies `texconv.exe` into the target directory during build
- `/config.ini` – Project configuration (input/output paths, options per tool)

---

## ⚙️ Requirements

- **Rust** (stable) – recommended via [rustup](https://rustup.rs/)
- A **local Archlord client installation** (source data)
- **Windows** is currently the primary target (because `texconv.exe` is used). Other platforms may require adjustments.

Optional / depending on features:

- `texconv.exe` in the `tools/` directory (for `txd_converter`)

---

## 📦 Building the Workspace

Clone the repository and build the full workspace in release mode:

```bash
git clone https://github.com/<your-account>/Archlord-AIO.git
cd Archlord-AIO

cargo build --release
```

This will compile all apps in `apps/` and shared libraries in `libs/`.

---

## 🚀 Running the Tools

### 1. Configure `config.ini`

Before running the tools, adjust `config.ini` in the repository root, for example:

```ini
[source]
client_path = "D:/Games/Archlord"    # Path to your Archlord client installation

[output]
base_path   = "E:/ArchlordOutput"    # Where extracted / processed data will be stored
```

The exact options depend on the current implementation; check the source or comments in `config.ini` for details.

### 2. Run the core controller

Typical use case – run the whole pipeline:

```bash
cargo run -p core_main --release
```

### 3. Run individual tools

You can also start single steps directly, for example:

```bash
# Only extraction & decryption
cargo run -p extractor --release

# Only TXD texture conversion
cargo run -p txd_converter --release
```

Show available options for a tool with:

```bash
cargo run -p extractor -- --help
```

---

## 🧪 Development

Run tests (if present) with:

```bash
cargo test
```

Contributions are welcome via pull requests (new format support, better validation, tooling improvements, etc.).

---

## 📄 License

Add the actual license information for the project here, for example:

This project is licensed under the MIT License (see `LICENSE`).