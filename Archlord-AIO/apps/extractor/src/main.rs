use shared_utils::{
    ensure_config_file, find_files, load_paths_from_config, process_dat_files,
    verify_destination_structure,
};

fn main() {
    ensure_config_file().expect("Fehler beim Initialisieren der config.ini");

    if let Err(e) = run() {
        eprintln!("❌ Fehler im Extractor:\n{e}");
        std::process::exit(1);
    }
}

fn run() -> std::io::Result<()> {
    let (source_path, destination_path) = load_paths_from_config();
    verify_destination_structure(&destination_path)?;

    let files = find_files(source_path.as_ref());
    process_dat_files(&files, &destination_path);

    println!("✅ Extraktion abgeschlossen.");
    Ok(())
}
