use shared_utils::{ensure_config_file, load_paths_from_config, scan_dff_textures};
use std::path::Path;

fn main() {
    ensure_config_file().expect("Fehler beim Initialisieren der config.ini");

    if let Err(e) = run() {
        eprintln!("❌ Fehler im DFF-Scanner:\n{e}");
        std::process::exit(1);
    }
}

fn run() -> std::io::Result<()> {
    let (_source, destination) = load_paths_from_config();
    scan_dff_textures(Path::new(&destination))?;
    println!("✅ DFF Texture Scan abgeschlossen.");
    Ok(())
}
