# rw_model_dump

`rw_model_dump` ist ein spezialisiertes Werkzeug zum Extrahieren einer High-Level-Zusammenfassung von Modelldaten aus RenderWare-Dateien. Es konzentriert sich auf den semantischen Inhalt des Modells, wie Geometrien und Materialien.

## Funktionen
- **Modell-Zusammenfassung**: Dekodiert RenderWare `Geometry`- und `Material`-Chunks in ein lesbares Format.
- **Detaillierte Berichterstattung**: Enthält Informationen über Vertices, Dreiecke, Texturreferenzen und verwendete Plugins.
- **Deterministische Ausgabe**: Erzeugt JSON, das für vergleichende Analysen und automatisierte Tests geeignet ist.

## Funktionsweise
Das Tool verwendet die `rw_dff_model`-Bibliothek, um einen umfassenden Bericht über eine RenderWare-Datei zu erstellen. Es parst den Chunk-Tree und wendet anschließend Dekodierungslogik an, um aussagekräftige Daten aus den einzelnen Payloads zu extrahieren. Das Ergebnis ist ein strukturiertes JSON-Objekt, das die visuellen und strukturellen Eigenschaften des Modells beschreibt.

## Abhängigkeiten
- **Standalone**: Kann als eigenständiges Analysetool für Modelldateien verwendet werden.
- **Bibliotheken**: Abhängig von `rw_dff` für das Tree-Parsing und `rw_dff_model` für die modellspezifische Dekodierung.

## Benutzung
```bash
cargo run -p rw_model_dump -- <pfad-zur-datei>
```

### Parameter
- `<pfad-zur-datei>`: Pfad zur RenderWare `.dff`- oder `.txd`-Datei, die analysiert werden soll.

### Beispiel
```bash
cargo run -p rw_model_dump -- h0000a00.dff > h0000a00.model.json
```
Dieser Befehl erzeugt einen High-Level-Modellbericht für `h0000a00.dff` und speichert die Ausgabe in `h0000a00.model.json`.
