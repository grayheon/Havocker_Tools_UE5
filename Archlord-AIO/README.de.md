# 🛠 Archlord Universal Tool

Dieses auf Rust basierende Workspace-Projekt bietet ein modulares System zur Extraktion, Entschlüsselung, Konvertierung, Validierung und Analyse von Archlord-Clientdaten.

## 🔧 Toolübersicht

| App           | Beschreibung |
|---------------|--------------|
| `core_main`   | Zentrale Steuerung aller Tools in Reihenfolge oder parallel |
| `extractor`   | Kopiert und entschlüsselt Quelldateien aus dem Quellverzeichnis |
| `txd_converter` | Konvertiert `.txd`-Dateien zu `.dds` und `.png` |
| `minimap`     | Erzeugt ein Weltkartenbild aus Minimap-Tiles |
| `obj_checker` | Prüft Objektvorlagen und erzeugt CSV/Prüfdateien |
| `dff_scanner` | Sucht `.dff`-Dateien nach eingebetteten Texturen |
| `txd_viewer`  | Platzhalter oder visuelles Tool für `.txd` |

## 🧱 Projektstruktur

- `/apps` – Alle Binaries, je ein Modul
- `/libs/shared_utils` – Gemeinsame Logik und Helferfunktionen
- `/tools` – Externe Tools (z. B. `texconv.exe`)
- `/build.rs` – Kopiert `texconv.exe` in das Build-Ziel
- `/config.ini` – Projektkonfiguration

## 📦 Ausführen

Workspace bauen:
```bash
cargo build --release
```

Zentrale Steuerung starten:
```bash
cargo run -p core_main
```
