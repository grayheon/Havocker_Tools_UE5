# obj_checker

`obj_checker` ist ein Validierungswerkzeug, das die Konsistenz und Integrität von Objekt-Templates und deren zugehörigen Modellen (DFF) und Texturen (TID) in den Archlord-Daten sicherstellt.

## Funktionen
- **Objekt-Template-Verarbeitung**: Analysiert und validiert die Struktur der `objecttemplate.ini`.
- **TID-Validierung**: Gleicht Texture-ID (TID) Einträge mit den Objekt-Templates ab.
- **DFF-Konsistenzprüfung**: Extrahiert und verifiziert `.dff`-Modelldateien, um sicherzustellen, dass sie den Template-Definitionen entsprechen.
- **Referenzintegrität**: Erkennt fehlende oder falsch zugeordnete Verknüpfungen zwischen Templates, Modellen und Texturen.

## Funktionsweise
Das Tool lädt die verarbeiteten Spieldaten und führt eine Reihe von Prüfungen unter Verwendung der Validierungslogik aus `shared_utils` durch. Es iteriert durch die Objekt-Templates, folgt den Referenzen zu Modell- und Texturdateien und prüft deren Existenz sowie grundlegende Eigenschaften. Dies hilft bei der Identifizierung von Datenkorruption oder fehlenden Assets, die im Spiel-Client Probleme verursachen würden.

## Abhängigkeiten
- **Standalone**: Kann unabhängig ausgeführt werden, wenn die verarbeiteten Daten im Zielpfad verfügbar sind.
- **Integriert**: Wird automatisch von `core_main` während des Hauptverarbeitungsflusses ausgelöst.
- **Bibliotheken**: Verlässt sich stark auf `shared_utils` für das Parsen von Templates und Validierungsroutinen.

## Benutzung
```bash
cargo run -p obj_checker
```
Das Tool validiert die Konsistenz von Objekten und Modellen innerhalb des in der `config.ini` angegebenen Zielverzeichnisses.

