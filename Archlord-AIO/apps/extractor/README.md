# extractor

## English
**Description**  
Unpacks Archlord `.dat` archives into the configured destination and recreates the original directory layout.

**How it works**  
- Reads `source_path`/`destination_path` from `config.ini`.  
- Finds `.dat` files, verifies the destination structure, and extracts entries with `shared_utils`.  
- Rebuilds folders on the fly while writing files.

**Usage**  
```bash
cargo run -p extractor
```
Requires a valid `config.ini` in the workspace root.

## Deutsch
**Beschreibung**  
Entpackt Archlord-`.dat`-Archive ins konfigurierte Ziel und stellt die Verzeichnisstruktur wieder her.

**Ablauf**  
- `source_path`/`destination_path` aus `config.ini` laden.  
- `.dat`-Dateien finden, Zielstruktur prüfen und Einträge mit `shared_utils` extrahieren.  
- Unterordner bei Bedarf anlegen und Dateien schreiben.

**Verwendung**  
```bash
cargo run -p extractor
```
Benötigt eine gültige `config.ini` im Workspace-Root.
