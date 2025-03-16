@echo off
setlocal enabledelayedexpansion

:: Ziel-Datei für die Ausgabe
set "output=%cd%\Dateien.txt"
del "%output%" 2>nul

:: Temporäre Datei für die Liste
set "tempfile=%cd%\temp_list.txt"
del "%tempfile%" 2>nul

:: Rekursiv nach PNG-Dateien suchen und nur die Dateinamen speichern
for /r "%cd%" %%F in (*.png) do (
    echo %%~nxF >> "%tempfile%"
)

:: Doppelte Einträge entfernen und alphabetisch sortieren
sort "%tempfile%" /unique > "%output%"

:: Temporäre Datei löschen
del "%tempfile%" 2>nul

echo Alphabetische Liste gespeichert in "%output%"
pause
