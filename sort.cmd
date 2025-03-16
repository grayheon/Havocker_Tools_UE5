@echo off
setlocal enabledelayedexpansion

:: Durchsuche den aktuellen Ordner nach .fbx-Dateien
for %%F in (*.blend) do (
    set "filename=%%~nF"
    set "foldername=!filename!"
    
    :: Überprüfen, ob der Ordner existiert, sonst erstellen
    if not exist "!foldername!" mkdir "!foldername!"
    
    :: Verschiebe die Datei in den entsprechenden Unterordner
    move "%%F" "!foldername!\"
    
    echo Verschoben: %%F -> !foldername!\
)

echo Fertig!
pause
