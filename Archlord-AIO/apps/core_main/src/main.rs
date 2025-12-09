use shared_utils::{ensure_config_file, find_files, load_paths_from_config, prepare_destination, process_dat_files, process_regular_files};
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
    let (source_path, destination_path) = load_paths_from_config();
    prepare_destination(&destination_path)?;

    let files = find_files(Path::new(&source_path));
    process_regular_files(&files, &source_path, &destination_path);
    process_dat_files(&files, &destination_path);

    let handles = [
        thread::spawn(|| run_subtool("minimap")),
        thread::spawn(|| run_subtool("obj_checker")),
        thread::spawn(|| run_subtool("txd_converter")),
    ];

    for handle in handles {
        if let Err(e) = handle.join() {
            eprintln!("💥 Subtool-Thread ist gepanict: {:?}", e);
        }
    }

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
