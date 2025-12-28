# dff_scanner

`dff_scanner` ist ein Werkzeug zur Validierung von Texturreferenzen in RenderWare `.dff`-Modelldateien. Es stellt sicher, dass alle von einem Modell referenzierten Texturen in der Zielumgebung tatsächlich verfügbar sind.

## Funktionen
- **Texturextraktion**: Parsed rekursiv den RenderWare-Chunk-Tree, um `RW_TEXTURE`-Chunks zu finden.
- **Validierung**: Vergleicht extrahierte Texturnamen mit einem Satz bekannter verfügbarer `.png`-Texturen.
- **Berichterstattung**: Generiert für jede verarbeitete `.dff`-Datei eine `<modellname>_textures.txt`, in der gültige und ungültige/fehlende Texturreferenzen aufgelistet sind.

## Funktionsweise
Das Tool verwendet die `rw_dff`-Bibliothek, um die binäre Struktur von `.dff`-Dateien zu parsen. Es sucht gezielt nach Texturnamen-Strings innerhalb der RenderWare-Hierarchie. Durch den Abgleich dieser Namen mit tatsächlichen Bilddateien auf der Festplatte können fehlerhafte Referenzen identifiziert werden, die im Spiel zu fehlenden Texturen führen würden.

## Abhängigkeiten
- **Standalone**: Kann unabhängig ausgeführt werden, verlässt sich jedoch auf eine `config.ini`, um den Zielpfad zu definieren, in dem sich die `.dff`-Dateien und Texturen befinden.
- **Integriert**: Wird normalerweise von `core_main` als Teil der vollständigen Verarbeitungspipeline aufgerufen.
- **Bibliotheken**: Abhängig von `rw_dff` für das Parsing und `rw_dff_model` für die Dekodierung der Texturinformationen.

## Benutzung
```bash
cargo run -p dff_scanner
```
Das Tool scannt das in der `config.ini` angegebene Verzeichnis nach `.dff`-Dateien und validiert deren Texturreferenzen gegen die in derselben Umgebung gefundenen Texturen.

