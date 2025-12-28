# extractor

## Deutsch
**Beschreibung**  
Entpackt Archlord-`.dat`-Archive in das konfigurierte Ziel und stellt die Ordnerstruktur wieder her.

**Ablauf**  
- `source_path`/`destination_path` aus `config.ini` lesen.  
- `.dat`-Dateien finden, Zielstruktur prüfen und Einträge mit `shared_utils` extrahieren.  
- Unterordner bei Bedarf anlegen und Dateien schreiben.

**Verwendung**  
```bash
cargo run -p extractor
```
`config.ini` muss im Workspace-Root liegen.

## English
**Description**  
Unpacks Archlord `.dat` archives into the configured destination and recreates the original directory layout.

**Flow**  
- Read `source_path`/`destination_path` from `config.ini`.  
- Locate `.dat` files, validate the destination, and extract entries using `shared_utils`.  
- Create subfolders as needed while writing files.

**Usage**  
```bash
cargo run -p extractor
```
Requires a valid `config.ini` in the workspace root.
