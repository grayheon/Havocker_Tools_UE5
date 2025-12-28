# csv_exporter

`csv_exporter` is a tool for converting Archlord's proprietary text-based data tables (found in `.ini` and `.txt` files) into standard, Excel-friendly `.csv` files.

## Features
- **Legacy Format Support**: Decodes files using various encodings, including UTF-8 (with/without BOM), UTF-16, and EUC-KR (common for Korean Archlord files).
- **Automatic Delimiter Detection**: Identifies tab-separated values within text files.
- **Excel Compatibility**: Exports data using a semicolon (`;`) as the separator, which is standard for many European versions of Excel.
- **Smart Quoting**: Automatically wraps fields containing special characters (semicolons, quotes, etc.) in double quotes to preserve data integrity.

## How it works
The tool recursively scans the destination directory for `.ini` and `.txt` files. For each file, it attempts to detect the encoding and identifies if it contains a table structure (primarily tab-delimited). If a table is detected, it converts the rows and columns into a CSV format while handling special character escaping according to CSV specifications.

## Dependencies
- **Standalone**: Can be run as a standalone utility to convert text tables.
- **Libraries**: Uses `encoding_rs` for robust character set handling and `shared_utils` for configuration management.

## Usage
```bash
cargo run -p csv_exporter
```
The tool automatically processes all supported files in the destination directory specified in your `config.ini`.
