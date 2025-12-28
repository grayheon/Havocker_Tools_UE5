use shared_utils::{ensure_config_file, load_paths_from_config, process_txd_pipeline};

fn main() {
    ensure_config_file().expect("Fehler beim Initialisieren der config.ini");

    if let Err(e) = run() {
        eprintln!("❌ Fehler im TXD-Konverter:\n{e}");
        std::process::exit(1);
    }
}

fn run() -> std::io::Result<()> {
    let (_source_path, destination_path) = load_paths_from_config();
    process_txd_pipeline(&destination_path)?;
    println!("✅ TXD-Konvertierung abgeschlossen.");
    Ok(())
}
