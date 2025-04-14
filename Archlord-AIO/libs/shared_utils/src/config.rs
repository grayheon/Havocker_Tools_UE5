use std::{fs, io, path::Path};
use std::io::Write;
use configparser::ini::Ini;

#[cfg(target_os = "windows")]
fn open_config_in_editor() {
    std::process::Command::new("notepad")
        .arg("config.ini")
        .spawn()
        .expect("❌ Konnte Notepad nicht starten");
}

pub fn ensure_config_file() -> io::Result<()> {
    let config_path = Path::new("config.ini");
    if !config_path.exists() {
        let mut content = String::new();
        content.push_str("[PATHS]\n");
        content.push_str("SOURCE=D:\\Archlord-EMU\\Webzen\\Archlord\n");
        content.push_str("DESTINATION=D:\\Archlord-EMU\\Rust-Export-Test\n\n");
        content.push_str("[EXPORT]\n");
        content.push_str("CREATE_PROOF=true\n");
        content.push_str("CREATE_CSV=true\n\n");
        content.push_str("[MODES]\n");
        content.push_str("SCAN_DFF=true\n");
        content.push_str("SCAN_TID=true\n");

        fs::write(config_path, content)?;
        println!("📝 config.ini wurde erstellt.");
        println!("📂 Bitte trage die Pfade in der Konfigurationsdatei ein...");
        open_config_in_editor();
        println!("✏️  Ändere die Datei und drücke [ENTER], um fortzufahren...");
        let _ = io::stdout().flush();
        let mut dummy = String::new();
        let _ = io::stdin().read_line(&mut dummy);
    } else {
        println!("✅ config.ini bereits vorhanden.");
    }
    Ok(())
}

pub fn load_config(filename: &str) -> Ini {
    let mut config = Ini::new();
    config
        .load(filename)
        .expect("Fehler beim Laden der INI-Datei");
    config
}

pub fn load_paths_from_config() -> (String, String) {
    let config = load_config("config.ini");
    let source = config.get("PATHS", "SOURCE").expect("Fehler: SOURCE fehlt");
    let dest = config.get("PATHS", "DESTINATION").expect("Fehler: DESTINATION fehlt");
    (source, dest)
}

pub fn prepare_destination(destination_path: &str) -> io::Result<()> {
    if destination_path.trim().is_empty()
        || destination_path.contains(['*', '?', '"', '<', '>', '|'])
    {
        panic!("Fehler: Ungültiges Zielverzeichnis: {}", destination_path);
    }

    let dest_path = Path::new(destination_path);
    if !dest_path.is_absolute() {
        panic!("Fehler: Zielverzeichnis muss absolut sein: {}", destination_path);
    }

    fs::create_dir_all(dest_path)?;
    clear_destination_folder(dest_path)
}

fn clear_destination_folder(destination_path: &Path) -> io::Result<()> {
    if destination_path.exists() && destination_path.read_dir()?.next().is_some() {
        println!("⚠️  Zielordner nicht leer. Lösche: {}", destination_path.display());
        fs::remove_dir_all(destination_path)?;
        println!("✅ Zielordner wurde geleert.");
    } else {
        println!("✅ Zielordner ist leer oder neu.");
    }
    Ok(())
}

pub fn verify_destination_structure(destination_path: &str) -> io::Result<()> {
    let path = Path::new(destination_path);

    if !path.exists() {
        fs::create_dir_all(path)?;
        println!("📁 Zielverzeichnis erstellt: {}", path.display());
    } else {
        println!("📁 Zielverzeichnis vorhanden: {}", path.display());
    }

    Ok(())
}
