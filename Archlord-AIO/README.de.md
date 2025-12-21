# 🛠 Archlord Universal Tool

Dieses Repository ist ein Rust-Workspace, der ein modulares Toolset bereitstellt, um Daten aus einer **Archlord**-Client-Installation zu extrahieren, zu entschlüsseln, zu konvertieren, zu validieren und zu analysieren.

> ⚠️ **Hinweis / Disclaimer**  
> Dieses Projekt ist ein inoffizielles Fan- und Reverse-Engineering-Projekt. Es steht in keiner Verbindung zu Rechteinhabern von Archlord und wird von diesen weder unterstützt noch genehmigt.  
> Verwende das Tool nur mit einer legal erworbenen Kopie des Spiels und beachte die EULA und lokale Gesetze.

---

## ✨ Funktionsumfang

- End-to-End-Pipeline zur Verarbeitung eines Archlord-Client-Verzeichnisses
  - Extraktion und Entschlüsselung der Originaldateien
  - Konvertierung proprietärer Texturformate (`.txd`) in gängige Formate (`.dds`, `.png`)
  - Erzeugung von Minimap- und Weltkartenbildern aus Tile-Daten
  - Scan und Validierung von Objektvorlagen und 3D-Ressourcen
- Modulare Architektur: jeder Verarbeitungsschritt ist ein eigenes Binary unter `apps/`
- Geteilte Utilities in `libs/shared_utils` (Datei-Helfer, Formate, Logging usw.)

---

## 🔧 Tool-Übersicht

| App           | Beschreibung |
|---------------|--------------|
| `core_main`   | Zentrale Steuerung, die alle Tools nacheinander (oder in Auswahl) ausführt |
| `extractor`   | Kopiert und entschlüsselt Dateien aus dem Archlord-Client-Verzeichnis |
| `txd_converter` | Konvertiert `.txd`-Dateien in `.dds` und `.png` (benötigt `texconv.exe` in `tools/`) |
| `minimap`     | Erstellt ein zusammenhängendes Weltkartenbild aus Minimap-Tiles |
| `obj_checker` | Prüft Objektvorlagen und erzeugt CSV-/Prüfdaten |
| `dff_scanner` | Durchsucht `.dff`-Dateien nach eingebetteten Textur-Namen / Referenzen |
| `txd_viewer`  | (WIP) Visuelles / interaktives Tool zum Anzeigen von `.txd`-Dateien |

Wenn du nur einen bestimmten Schritt benötigst (z. B. nur Textur-Konvertierung), kannst du das entsprechende Binary direkt starten.

---

## 🧱 Projektstruktur

- `/apps` – Alle Binaries, je ein logischer Verarbeitungsschritt:
  - `/apps/core_main` – High-Level-Controller
  - `/apps/extractor` – Extraktion & Entschlüsselung
  - `/apps/txd_converter` – Textur-Konvertierung
  - `/apps/minimap` – Minimap- / Weltkarten-Generierung
  - `/apps/obj_checker` – Objektvorlagen-Prüfung
  - `/apps/dff_scanner` – DFF-Analyse
  - `/apps/txd_viewer` – Experimenteller Viewer
- `/libs/shared_utils` – Gemeinsame Logik, Datei-Handler, Datenstrukturen
- `/tools` – Externe Tools (z. B. `texconv.exe` für die Textur-Konvertierung)
- `/build.rs` – Kopiert `texconv.exe` während des Builds in das Zielverzeichnis
- `/config.ini` – Projektkonfiguration (Ein-/Ausgabepfade, Optionen pro Tool)

---

## ⚙️ Voraussetzungen

- Rust (stable), z. B. über [rustup](https://rustup.rs/)
- Eine lokale Archlord-Client-Installation (Quelldaten)
- Windows als primäre Zielplattform (wegen `texconv.exe`)

Optional / je nach genutzten Funktionen:

- `texconv.exe` im Verzeichnis `tools/` (für `txd_converter`)

---

## 📦 Workspace bauen

Workspace im Release-Modus bauen:

```bash
cargo build --release
```

Dadurch werden alle Apps unter `apps/` sowie die Libraries unter `libs/` kompiliert.

---

## 🚀 Tools ausführen

### 1. `config.ini` anpassen

Passe vor dem Start die Datei `config.ini` im Projekt-Root an (Beispiel):

```ini
[source]
client_path = "D:/Spiele/Archlord"

[output]
base_path   = "E:/ArchlordOutput"
```

Die genauen Optionen hängen vom aktuellen Stand der Implementierung ab.

### 2. Zentrale Steuerung starten

Typischer Anwendungsfall – komplette Pipeline ausführen:

```bash
cargo run -p core_main --release
```

### 3. Einzelne Tools nutzen

Beispiele:

```bash
# Nur Extraktion & Entschlüsselung
cargo run -p extractor --release

# Nur TXD-Texturen konvertieren
cargo run -p txd_converter --release
```

Mit `--help` kannst du die jeweils verfügbaren Optionen anzeigen:

```bash
cargo run -p extractor -- --help
```

---

## 📄 Lizenz

Trage hier die tatsächliche Lizenz deines Projekts ein, z. B.:

Dieses Projekt steht unter der MIT-Lizenz (siehe `LICENSE`).
