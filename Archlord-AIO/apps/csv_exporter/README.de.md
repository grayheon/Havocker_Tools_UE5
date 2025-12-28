# csv_exporter

`csv_exporter` ist ein Werkzeug zur Konvertierung von Archlords proprietären textbasierten Datentabellen (zu finden in `.ini`- und `.txt`-Dateien) in Standard-, Excel-freundliche `.csv`-Dateien.

## Funktionen
- **Unterstützung für Legacy-Formate**: Dekodiert Dateien unter Verwendung verschiedener Kodierungen, einschließlich UTF-8 (mit/ohne BOM), UTF-16 und EUC-KR (üblich für koreanische Archlord-Dateien).
- **Automatische Delimiter-Erkennung**: Identifiziert tabulatorgetrennte Werte innerhalb von Textdateien.
- **Excel-Kompatibilität**: Exportiert Daten mit einem Semikolon (`;`) als Trennzeichen, was Standard für viele europäische Excel-Versionen ist.
- **Intelligentes Quoting**: Setzt Felder, die Sonderzeichen enthalten (Semikolons, Anführungszeichen usw.), automatisch in doppelte Anführungszeichen, um die Datenintegrität zu wahren.

## Funktionsweise
Das Tool scannt das Zielverzeichnis rekursiv nach `.ini`- und `.txt`-Dateien. Für jede Datei versucht es, die Kodierung zu erkennen und festzustellen, ob sie eine Tabellenstruktur enthält (primär Tab-getrennt). Wenn eine Tabelle erkannt wird, konvertiert es die Zeilen und Spalten in ein CSV-Format, wobei die Maskierung von Sonderzeichen gemäß den CSV-Spezifikationen erfolgt.

## Abhängigkeiten
- **Standalone**: Kann als eigenständiges Dienstprogramm zur Konvertierung von Texttabellen ausgeführt werden.
- **Bibliotheken**: Verwendet `encoding_rs` für eine robuste Handhabung von Zeichensätzen und `shared_utils` für das Konfigurationsmanagement.

## Benutzung
```bash
cargo run -p csv_exporter
```
Das Tool verarbeitet automatisch alle unterstützten Dateien im Zielverzeichnis, das in der `config.ini` angegeben ist.
