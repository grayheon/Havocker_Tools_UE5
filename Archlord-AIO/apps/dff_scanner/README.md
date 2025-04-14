# 🔍 DFF Scanner

Scans `.dff` files for embedded texture names (e.g. for validation).

## Features
- Recursively reads all `.dff` files
- Extracts ASCII texture names (8–9 characters)
- Checks if corresponding PNGs exist
- Writes `.txt` files with valid/missing entries
