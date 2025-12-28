# dff2gltf

`dff2gltf` ist ein Konverter, der RenderWare `.dff`-Modelldateien in das moderne glTF 2.0-Format umwandelt. Dies ermöglicht die Nutzung von Archlord-Assets in modernen 3D-Engines und Werkzeugen wie Blender.

## Funktionen
- **glTF 2.0 Export**: Erzeugt Standard `.gltf`- und `.bin`-Dateien.
- **Vereinheitlichte Mesh-Verarbeitung**: Verwendet eine vereinheitlichte Mesh-Repräsentation, um einen konsistenten Geometrie-Export zu gewährleisten.
- **Material-Unterstützung**: Erzeugt glTF-Primitive, die den Material-Splits von RenderWare entsprechen.
- **Textur-Koordination**: Gibt mehrere UV-Sets (texcoord_0..4) aus und ordnet sie den korrekten Materialien zu.
- **Automatische Organisation**: Platziert die exportierten Dateien und referenzierten Texturen in einem vorgesehenen Ausgabeordner.

## Funktionsweise
Das Tool erstellt zunächst mit `rw_dff_model` einen vereinheitlichten Mesh-Bericht aus der Eingabedatei (`.dff`). Anschließend übergibt es diesen Bericht an die `gltf_writer`-Bibliothek, welche die glTF-JSON-Struktur und den zugehörigen Binärpuffer konstruiert. Zudem wird versucht, die notwendigen Texturen zu lokalisieren und zu verknüpfen.

## Abhängigkeiten
- **Standalone**: Kann als eigenständiger Kommandozeilen-Konverter verwendet werden.
- **Bibliotheken**: Abhängig von `rw_dff`, `rw_dff_model` und `gltf_writer`.

## Benutzung
```bash
cargo run -p dff2gltf -- <input.dff> [zielverzeichnis]
```

### Parameter
- `<input.dff>`: Pfad zur RenderWare `.dff`-Modelldatei, die konvertiert werden soll.
- `[zielverzeichnis]` (Optional): Das Verzeichnis, in dem die resultierenden `.gltf`- und `.bin`-Dateien gespeichert werden sollen. Wenn weggelassen, werden die Dateien im selben Verzeichnis wie die Eingabedatei abgelegt.

### Beispiel
```bash
cargo run -p dff2gltf -- models/h0000a00.dff ./export
```
Dieser Befehl konvertiert `h0000a00.dff` und speichert `h0000a00.gltf` sowie `h0000a00.bin` im Ordner `./export`. Das Tool versucht außerdem, die benötigten Texturen zu finden und in denselben Ordner zu kopieren.
