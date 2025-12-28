# core_main

`core_main` ist das zentrale Orchestrierungswerkzeug der Archlord-AIO Toolchain. Es verwaltet den gesamten Workflow der Datenverarbeitung, vom initialen Kopieren und Extrahieren der Dateien bis hin zur Ausführung spezialisierter Sub-Tools.

## Funktionen
- **Konfigurationsmanagement**: Stellt sicher, dass die `config.ini` existiert und lädt Quell- und Zielpfade.
- **Dateiorganisation**: Kopiert reguläre Dateien und verarbeitet `.dat`-Archive vom Quell- in das Zielverzeichnis.
- **Sub-Tool-Orchestrierung**: Führt mehrere Sub-Module parallel aus, um die extrahierten Daten zu verarbeiten:
  - `minimap`: Generiert Weltkarten.
  - `obj_checker`: Validiert Objekt-Templates und Modelle.
  - `txd_converter`: Konvertiert Texturen in moderne Formate.
  - `dff_scanner`: Scannt und validiert Modell-Texturen (wird nach den anderen ausgeführt).

## Funktionsweise
Das Tool liest Pfade aus einer `config.ini`. Anschließend scannt es das Quellverzeichnis, bereitet das Zielverzeichnis vor und startet die Verarbeitungspipeline. Die meisten spezialisierten Aufgaben werden an andere Binaries innerhalb des Workspaces delegiert, indem `cargo run` verwendet wird.

## Abhängigkeiten
- **Integriert**: Dieses Tool fungiert als Runner für `minimap`, `obj_checker`, `txd_converter` und `dff_scanner`.
- **Standalone**: Es kann unabhängig ausgeführt werden, erfordert jedoch eine gültige `config.ini` und die originalen Archlord-Daten.
- **Bibliotheken**: Verwendet `shared_utils` für die Kernlogik und Dateihandhabung.

## Benutzung
```bash
cargo run -p core_main
```

