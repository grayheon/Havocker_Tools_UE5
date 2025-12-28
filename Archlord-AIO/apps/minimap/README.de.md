# minimap

`minimap` ist ein Werkzeug zur Erstellung einer umfassenden Weltkarte aus den Geländedaten von Archlord. Es fügt einzelne Geländekacheln zu einem einzigen, vereinheitlichten Kartenbild zusammen.

## Funktionen
- **Geländezusammenführung**: Kombiniert verstreute Geländeinformationen zu einer kohärenten Karte.
- **Bilderzeugung**: Erzeugt eine visuelle Darstellung der Spielwelt.
- **Tile-Merging**: Führt speziell 2x2 Kacheln (64 kleine Kacheln = 1 vollständiges Kartensegment) zu einer kompletten Weltkarte zusammen.

## Funktionsweise
Das Tool scannt das Zielverzeichnis nach geländebezogenen Daten (z. B. `mapXXXXa/b/c/d.*` Dateien) und verwendet Algorithmen in `shared_utils`, um diese Teile zusammenzufügen. Es übernimmt das Koordinatenmapping der Kacheln, um sicherzustellen, dass die finale Weltkarte räumlich korrekt ist, und exportiert das Ergebnis als PNG oder BMP.

## Abhängigkeiten
- **Standalone**: Kann unabhängig ausgeführt werden, sofern die erforderlichen Geländedaten im konfigurierten Zielpfad verfügbar sind.
- **Integriert**: Wird von `core_main` während der automatisierten Verarbeitungspipeline gestartet.
- **Bibliotheken**: Verlässt sich auf `shared_utils` für die Geländeverarbeitung und die Logik zur Kartenkarten-Erzeugung.

## Benutzung
```bash
cargo run -p minimap
```
Das Tool verarbeitet Geländedaten aus dem in der `config.ini` angegebenen Zielverzeichnis, um die Weltkarte zu generieren.

