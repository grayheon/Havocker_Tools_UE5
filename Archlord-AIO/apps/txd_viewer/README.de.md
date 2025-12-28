# txd_viewer

`txd_viewer` ist ein grafisches Debug-Tool zur Visualisierung der Inhalte von RenderWare `.txd` (Texture Dictionary) Dateien. Es ermöglicht Entwicklern, Texturdaten, Offsets und Verschiebungen in Echtzeit zu untersuchen.

## Funktionen
- **Visuelle Inspektion**: Laden und Betrachten von Texturen direkt aus binären `.txd`-Dateien.
- **Dynamische Anpassungen**: Echtzeit-Schieberegler zum Anpassen von Daten-Offsets, X/Y-Verschiebungen und Spaltensprüngen, um die korrekte Ausrichtung der rohen Texturdaten zu identifizieren.
- **Zoomen**: Detailgenaue Untersuchung von Texturen durch eine Zoom-Funktion.
- **Format-Unterstützung**: Behandelt verschiedene Texturdimensionen und Pixeldaten.

## Funktionsweise
Das Tool basiert auf dem `eframe` (egui) Framework. Es bietet eine Benutzeroberfläche, in der man eine `.txd`-Datei auswählen und anschließend verschiedene Parameter (Offset, X-Shift, Y-Shift etc.) manuell anpassen kann, um zu sehen, wie diese die Rekonstruktion des Bildes aus dem rohen Byte-Stream beeinflussen. Dies ist besonders nützlich für das Reverse-Engineering unbekannter Texturformate oder das Debuggen von Ausrichtungsproblemen.

## Abhängigkeiten
- **Standalone**: Funktioniert als eigenständige GUI-Anwendung.
- **Bibliotheken**: Verwendet `eframe` für die Benutzeroberfläche und `image` für die Bildverarbeitung und Skalierung.

## Benutzung
```bash
cargo run -p txd_viewer
```

