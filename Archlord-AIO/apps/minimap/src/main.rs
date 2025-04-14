use shared_utils::{ensure_config_file, load_paths_from_config, generate_world_map};
use std::path::Path;

fn main() {
    ensure_config_file().expect("Fehler beim Initialisieren der config.ini");

    if let Err(e) = run() {
        eprintln!("❌ Fehler bei Weltkarte:\n{e}");
        std::process::exit(1);
    }
}

fn run() -> std::io::Result<()> {
    let (_source, destination) = load_paths_from_config();
    generate_world_map(Path::new(&destination))?;
    println!("✅ Weltkarte erfolgreich erstellt.");
    Ok(())
}
