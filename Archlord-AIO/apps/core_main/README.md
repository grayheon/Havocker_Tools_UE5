# core_main

## English
**Description**  
`core_main` is the orchestrator of the Archlord-AIO toolchain. It reads `config.ini`, prepares the destination layout, copies regular files, extracts `.dat` archives, and launches the important sub-tools (`minimap`, `txd_converter`, `dff2gltf`, optionally `obj_checker`).

**How it works**  
- Load `source_path` and `destination_path` from `config.ini`.  
- Copy regular files and unpack `.dat` archives into the destination.  
- Kick off subtools via `cargo run -p <tool>` (runs in parallel where possible).  
- Print a summary when all tasks are done.

**Usage**  
```bash
cargo run -p core_main
```
`config.ini` must exist in the workspace root with valid `source_path` and `destination_path`.

## Deutsch
**Beschreibung**  
`core_main` ist der Orchestrierer der Archlord-AIO-Toolchain. Es liest `config.ini`, bereitet den Zielpfad vor, kopiert normale Dateien, entpackt `.dat`-Archive und startet die wichtigsten Subtools (`minimap`, `txd_converter`, `dff2gltf`, optional `obj_checker`).

**Ablauf**  
- `source_path` und `destination_path` aus `config.ini` laden.  
- Reguläre Dateien kopieren und `.dat`-Archive ins Ziel entpacken.  
- Subtools per `cargo run -p <tool>` anstoßen (wo möglich parallel).  
- Abschließend eine Zusammenfassung ausgeben.

**Verwendung**  
```bash
cargo run -p core_main
```
`config.ini` muss im Workspace-Root liegen und gültige `source_path`/`destination_path` enthalten.
