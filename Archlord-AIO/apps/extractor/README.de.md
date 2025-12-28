# extractor

`extractor` ist ein Werkzeug zum Entpacken von Archlord `.dat`-Archivdateien. Es rekonstruiert die interne Dateistruktur aus diesen Archiven in ein Zielverzeichnis.

## Funktionen
- **DAT-Entpacken**: Extrahiert alle Dateien, die im proprietären `.dat`-Format von Archlord enthalten sind.
- **Pfadrekonstruktion**: Behält die ursprüngliche Verzeichnishierarchie bei, die in den Archiven gefunden wurde.
- **Verifizierung**: Prüft und stellt sicher, dass die Zielverzeichnisstruktur vor der Extraktion existiert.

## Funktionsweise
Das Tool identifiziert `.dat`-Dateien im Quellverzeichnis und verarbeitet sie unter Verwendung der in `shared_utils` implementierten Extraktionslogik. Es liest die Dateieinträge aus dem DAT-Header und schreibt die entsprechenden Daten in den Zielpfad, wobei notwendige Unterverzeichnisse on-the-fly erstellt werden.

## Abhängigkeiten
- **Standalone**: Kann als eigenständiges Extraktionsprogramm ausgeführt werden. Erfordert eine `config.ini`, um zu wissen, wo nach Quelldateien gesucht und wohin extrahiert werden soll.
- **Integriert**: Teil der `core_main`-Orchestrierung, die sicherstellt, dass alle erforderlichen Spieldaten extrahiert werden, bevor weitere Verarbeitungswerkzeuge ausgeführt werden.
- **Bibliotheken**: Verwendet `shared_utils` für das Parsen der DAT-Dateien und die Extraktionslogik.

## Benutzung
```bash
cargo run -p extractor
```
Das Tool verwendet die in der `config.ini` definierten Quell- und Zielpfade, um `.dat`-Archive zu lokalisieren und zu entpacken.

