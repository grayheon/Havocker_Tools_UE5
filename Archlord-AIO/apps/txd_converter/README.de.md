# txd_converter

`txd_converter` ist ein spezialisiertes Werkzeug zur Konvertierung von RenderWare `.txd` (Texture Dictionary) Dateien in moderne Bildformate wie `.png`. Dies ermöglicht das einfache Betrachten und Bearbeiten von Spieltexturen.

## Funktionen
- **TXD-Parsing**: Dekodiert die binären RenderWare-Textur-Dictionaries.
- **Formatkonvertierung**: Konvertiert interne Texturformate (einschließlich DXT1, DXT3, DXT5 und Rohdaten) in Standard-PNG-Bilder.
- **Pipeline-Integration**: Verarbeitet alle Texturen innerhalb einer definierten Verzeichnisstruktur als Teil eines automatisierten Workflows.

## Funktionsweise
Das Tool iteriert durch die Texturverzeichnisse des Spiels, identifiziert `.txd`-Dateien und verwendet die `process_txd_pipeline` aus `shared_utils`. Es extrahiert die rohen oder komprimierten Bilddaten aus den RenderWare-Chunks und konvertiert sie in das PNG-Format, wobei die Transparenz, sofern vorhanden, beibehalten wird.

## Abhängigkeiten
- **Standalone**: Kann als eigenständiges Konvertierungsprogramm verwendet werden.
- **Integriert**: Wird häufig von `core_main` aufgerufen, um sicherzustellen, dass alle Spieltexturen in einem modernen Format für andere Tools oder die Entwicklung verfügbar sind.
- **Bibliotheken**: Verlässt sich auf `shared_utils` für die Dekodierung und Konvertierung der Texturen.

## Benutzung
```bash
cargo run -p txd_converter
```
Das Tool konvertiert alle `.txd`-Dateien im in der `config.ini` angegebenen Zielverzeichnis in das `.png`-Format.

