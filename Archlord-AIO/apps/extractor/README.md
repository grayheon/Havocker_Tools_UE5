# extractor

`extractor` is a tool designed to unpack Archlord `.dat` archive files. It reconstructs the internal file structure from these archives into a target directory.

## Features
- **DAT Unpacking**: Extracts all files contained within Archlord's proprietary `.dat` format.
- **Path Reconstruction**: Maintains the original directory hierarchy found within the archives.
- **Verification**: Checks and ensures the destination directory structure exists before extraction.

## How it works
The tool identifies `.dat` files in the source directory and processes them using extraction logic implemented in `shared_utils`. It reads the file entries from the DAT header and writes the corresponding data to the destination path, creating necessary subdirectories on the fly.

## Dependencies
- **Standalone**: Can be run as a standalone extraction utility. It requires a `config.ini` to know where to look for source files and where to extract them.
- **Integrated**: Part of the `core_main` orchestration, which ensures all required game data is extracted before further processing tools are run.
- **Libraries**: Uses `shared_utils` for the DAT parsing and extraction logic.

## Usage
```bash
cargo run -p extractor
```
The tool uses the source and destination paths defined in your `config.ini` to locate and unpack `.dat` archives.
