# rw_unified_dump

`rw_unified_dump` ist ein exportorientiertes Werkzeug, das eine vereinheitlichte Mesh-Repräsentation aus RenderWare `.dff`-Dateien erzeugt. Es bereitet Modelldaten so auf, dass sie für die Konvertierung in moderne Formate wie glTF optimiert sind.

## Funktionen
- **Vereinheitlichte Mesh-Repräsentation**: Führt disparate RenderWare-Geometriedaten in einer konsistenten Struktur zusammen.
- **Materialbasierte Aufteilung**: Behandelt Submeshes korrekt, indem sie entsprechend ihrer Materialzuordnungen aufgeteilt werden (unter Verwendung von `BinMeshPLG` oder Fallback-Logik).
- **Helfer-Trennung**: Unterscheidet zwischen tatsächlichen Render-Meshes und Helfer-Geometrien (z. B. Kollisionsboxen, Dummy-Knoten).
- **Exportbereit**: Stellt alle notwendigen Daten (Positionen, Normalen, UVs, Indizes) in einem flachen, einfach zu verarbeitenden JSON-Format bereit.

## Funktionsweise
Das Tool nutzt das `unified_scan`-Modul von `rw_dff_model`. Es parst die `.dff`-Datei, identifiziert alle Geometrien und rekonstruiert sie in `UnifiedMesh`-Objekte. Dies umfasst die Neuindizierung von Vertexdaten und die Handhabung der spezifischen Arten von RenderWare, Triangle-Strips oder -Listen zu speichern.

## Abhängigkeiten
- **Standalone**: Kann verwendet werden, um zu untersuchen, wie ein Modell während des Exports strukturiert sein wird.
- **Integriert**: Die zugrunde liegende Logik wird von `dff2gltf` verwendet.
- **Bibliotheken**: Abhängig von `rw_dff` und `rw_dff_model`.

## Benutzung
```bash
cargo run -p rw_unified_dump -- <pfad-zu-dff>
```

### Parameter
- `<pfad-zu-dff>`: Pfad zur RenderWare `.dff`-Datei, die analysiert werden soll.

### Beispiel
```bash
cargo run -p rw_unified_dump -- h0000a00.dff > h0000a00.unified.json
```
Dieser Befehl erzeugt eine vereinheitlichte JSON-Repräsentation von `h0000a00.dff` und leitet die Ausgabe in eine Datei namens `h0000a00.unified.json` um.
