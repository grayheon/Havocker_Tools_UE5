# rw_dump

`rw_dump` ist ein Diagnosewerkzeug zur Untersuchung der internen Struktur von RenderWare-Dateien (`.dff`, `.txd`). Es bietet eine Rohansicht des RenderWare-Chunk-Trees.

## Funktionen
- **Chunk-Tree-Visualisierung**: Parst die verschachtelte Struktur von RenderWare-Chunks.
- **JSON-Ausgabe**: Erzeugt eine deterministische JSON-Darstellung der Dateistruktur.
- **Keine Interpretation**: Konzentriert sich auf Rohdaten und Hierarchie, ohne spielspezifische Semantik anzuwenden.

## Funktionsweise
Das Tool nimmt einen Pfad zu einer RenderWare-Datei als Eingabe, verwendet die `rw_dff`-Bibliothek zum Parsen der binären Chunk-Hierarchie und serialisiert den resultierenden Baum nach JSON, das dann auf stdout ausgegeben wird. Es wird primär für das Debugging und die Erstellung von "Golden Files" für Regressionstests verwendet.

## Abhängigkeiten
- **Standalone**: Funktioniert als einfaches Kommandozeilen-Utility.
- **Bibliotheken**: Verlässt sich auf `rw_dff` für die Kern-Parsing-Logik und `serde_json` für die Serialisierung.

## Benutzung
```bash
cargo run -p rw_dump -- <pfad-zur-datei>
```

### Parameter
- `<pfad-zur-datei>`: Pfad zur RenderWare `.dff`- oder `.txd`-Datei, die analysiert werden soll.

### Beispiel
```bash
cargo run -p rw_dump -- h0000a00.dff > h0000a00.tree.json
```
Dieser Befehl parst den Chunk-Tree von `h0000a00.dff` und speichert die resultierende JSON-Struktur in `h0000a00.tree.json`.
