# core_main

`core_main` is the central orchestration tool of the Archlord-AIO toolchain. It manages the overall workflow of data processing, from initial file copying and extraction to the execution of specialized sub-tools.

## Features
- **Configuration Management**: Ensures `config.ini` exists and loads source/destination paths.
- **File Organization**: Copies regular files and processes `.dat` archives from the source to the destination directory.
- **Sub-tool Orchestration**: Executes several sub-modules to process the extracted data:
  - `minimap`: Generates world maps.
  - `obj_checker`: Validates object templates and models.
  - `txd_converter`: Converts textures to modern formats.
  - `dff_scanner`: Scans and validates model textures (runs after the others).

## How it works
The tool reads paths from a `config.ini`. It then scans the source directory, prepares the destination, and starts the processing pipeline. Most specialized tasks are delegated to other binaries within the workspace using `cargo run`.

## Dependencies
- **Integrated**: This tool acts as a runner for `minimap`, `obj_checker`, `txd_converter`, and `dff_scanner`.
- **Standalone**: It can be run independently but requires a valid `config.ini` and the source Archlord data.
- **Libraries**: Uses `shared_utils` for core logic and file handling.

## Usage
```bash
cargo run -p core_main
```

