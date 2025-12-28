# core_main

## Deutsch
**Beschreibung**  
`core_main` orchestriert die Archlord-AIO-Toolchain: Es liest `config.ini`, bereitet das Ziel-Layout vor, kopiert reguläre Dateien, entpackt `.dat`-Archive und startet die wichtigsten Subtools (`minimap`, `txd_converter`, `dff2gltf`, optional `obj_checker`).

**Ablauf**  
- `source_path` und `destination_path` aus `config.ini` laden.  
- Dateien kopieren und `.dat`-Archive im Ziel entpacken.  
- Subtools per `cargo run -p <tool>` anstoßen (wo möglich parallel).  
- Abschlussmeldung ausgeben.

**Verwendung**  
```bash
cargo run -p core_main
```
`config.ini` muss im Workspace-Root mit gültigen Pfaden liegen.

## English
**Description**  
`core_main` orchestrates the Archlord-AIO toolchain: reads `config.ini`, prepares the destination layout, copies regular files, unpacks `.dat` archives, and launches the key subtools (`minimap`, `txd_converter`, `dff2gltf`, optionally `obj_checker`).

**Flow**  
- Load `source_path` and `destination_path` from `config.ini`.  
- Copy files and unpack `.dat` archives into the destination.  
- Trigger subtools via `cargo run -p <tool>` (parallel where possible).  
- Emit a final summary.

**Usage**  
```bash
cargo run -p core_main
```
`config.ini` must sit in the workspace root with valid paths.
