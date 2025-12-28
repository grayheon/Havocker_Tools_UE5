# csv_exporter

## English
**Description**  
Scans the extracted Archlord data for text-like files (`.ini`, `.txt`), detects tabular content, normalizes encoding (EUC-KR/UTF-8), and emits semicolon-separated CSV copies.

**How it works**  
- Reads `destination_path` from `config.ini`.  
- Recursively visits text files, detects tab-delimited rows, quotes cells safely, and writes a `.csv` next to the source file (folders are created as needed).  
- Leaves non-tab files untouched.

**Usage**  
```bash
cargo run -p csv_exporter
```
`config.ini` must exist in the workspace root.

## Deutsch
**Beschreibung**  
Durchsucht die extrahierten Archlord-Daten nach Textdateien (`.ini`, `.txt`), erkennt tabellarische Inhalte, normalisiert die Kodierung (EUC-KR/UTF-8) und legt Semikolon-separierte CSV-Kopien an.

**Ablauf**  
- Liest `destination_path` aus `config.ini`.  
- Läuft rekursiv über Textdateien, erkennt Tab-getrennte Zeilen, quotet sicher und schreibt eine `.csv` neben die Quelldatei (Ordner werden bei Bedarf angelegt).  
- Nicht-tabellarische Dateien bleiben unverändert.

**Verwendung**  
```bash
cargo run -p csv_exporter
```
`config.ini` muss im Workspace-Root vorhanden sein.
