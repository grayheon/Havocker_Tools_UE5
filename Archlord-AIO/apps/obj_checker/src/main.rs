use shared_utils::{
    ensure_config_file,
    load_paths_from_config,
    process_obj_templates,
    validate_tids_against_objecttemplate,
    extract_and_check_dff_files,
};

fn main() {
    ensure_config_file().expect("Fehler beim Initialisieren der config.ini");

    if let Err(e) = run() {
        eprintln!("❌ Fehler im OBJ Checker:\n{e}");
        std::process::exit(1);
    }
}

fn run() -> std::io::Result<()> {
    let (_source, destination) = load_paths_from_config();

    process_obj_templates(destination.as_ref())?;
    validate_tids_against_objecttemplate(destination.as_ref())?;
    extract_and_check_dff_files(destination.as_ref())?;

    println!("✅ OBJ-Prüfung abgeschlossen.");
    Ok(())
}
