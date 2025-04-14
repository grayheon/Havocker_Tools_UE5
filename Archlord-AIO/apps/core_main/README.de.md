# 🧠 Core Main

Dies ist das zentrale Steuermodul, das alle Verarbeitungsschritte in definierter Reihenfolge und teils parallel ausführt.

## Funktionen
- Liest Konfigurationsdateien
- Führt nacheinander:
  - Extraktion
  - TXD-Konvertierung
  - Minimap-Erzeugung
  - Objektprüfung
  - DFF-Scanner
- Erkennt Fehler und meldet diese konsistent
- Steuert parallele Ausführung und wartet auf Abschluss

