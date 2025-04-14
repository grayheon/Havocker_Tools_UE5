use shared_utils::{ensure_config_file, load_paths_from_config, prepare_destination, find_files, process_regular_files, process_dat_files};
use std::path::Path;
use std::process::Command;
use std::thread;

fn main() {
    ensure_config_file().expect("❌ Fehler beim Initialisieren der config.ini");

    if let Err(err) = run() {
        eprintln!("❌ Fehler in core_main:\n{err}");
        std::process::exit(1);
    }
}

fn run() -> std::io::Result<()> {
    // 1. Konfiguration und Zielordner vorbereiten
    let (source_path, destination_path) = load_paths_from_config();
    prepare_destination(&destination_path)?;

    // 2. Kopieren und Entschlüsseln
    let files = find_files(Path::new(&source_path));
    process_regular_files(&files, &source_path, &destination_path);
    process_dat_files(&files, &destination_path);

    // 3. Starte parallele Tasks
    let t_minimap = thread::spawn(|| run_subtool("minimap"));
    let t_objcheck = thread::spawn(|| run_subtool("obj_checker"));
    let t_txd = thread::spawn(|| run_subtool("txd_converter"));

    // Warten auf parallele Tasks
    t_minimap.join().unwrap();
    t_objcheck.join().unwrap();
    t_txd.join().unwrap();

    // 4. DFF-Scanner ganz am Ende
    run_subtool("dff_scanner");

    println!("✅ Alle Tasks abgeschlossen.");
    Ok(())
}

/// Startet ein Submodul via `cargo run -p NAME`
fn run_subtool(tool_name: &str) {
    println!("🔧 Starte Tool: {tool_name}");
    let status = Command::new("cargo")
        .args(["run", "-p", tool_name])
        .status();

    match status {
        Ok(exit) if exit.success() => println!("✅ Tool {tool_name} abgeschlossen.\n"),
        Ok(exit) => println!("❌ Tool {tool_name} Fehlercode: {}\n", exit.code().unwrap_or(-1)),
        Err(err) => println!("💥 Fehler beim Start von {tool_name}: {err}"),
    }
}
