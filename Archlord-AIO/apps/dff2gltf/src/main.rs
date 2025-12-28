use shared_utils::{ensure_config_file, load_paths_from_config};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// cargo run -p dff2gltf -- h0000a00.dff out/gltf
///
fn main() {
    ensure_config_file().expect("Fehler beim Initialisieren der config.ini");

    let mut args = env::args_os();
    let exe = args.next().unwrap_or_default();

    match args.next() {
        Some(p) => {
            // Single-file mode (CLI usage)
            let in_path = PathBuf::from(p);
            let out_dir = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| in_path.parent().unwrap_or(Path::new(".")).to_path_buf());
            if let Err(e) = convert_one(&in_path, &out_dir) {
                eprintln!("Error: {e}");
                std::process::exit(2);
            }
            return;
        }
        None => batch_from_config(&exe),
    };
}

/// Batch mode: uses config.ini (DESTINATION) and converts all .dff/.ecl below it.
fn batch_from_config(exe: &std::ffi::OsString) {
    let (_source, dest) = load_paths_from_config();
    let root = PathBuf::from(dest);
    println!(
        "Starte Batch glTF Export: {} (aus config.ini)",
        root.display()
    );

    let mut files: Vec<PathBuf> = WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().map_or(false, |ext| {
                    ext.eq_ignore_ascii_case("dff") || ext.eq_ignore_ascii_case("ecl")
                })
        })
        .map(|e| e.into_path())
        .collect();
    files.sort();

    if files.is_empty() {
        eprintln!(
            "Usage: {} <input.dff/ecl> [out_dir]\nKein .dff/.ecl unter DESTINATION gefunden ({}).",
            exe.to_string_lossy(),
            root.display()
        );
        std::process::exit(1);
    }

    let mut ok = 0usize;
    for path in files {
        let out_dir = path.parent().unwrap_or(Path::new("."));
        match convert_one(&path, out_dir) {
            Ok(_) => ok += 1,
            Err(e) => eprintln!("Fehler bei {}: {e}", path.display()),
        }
    }

    println!(
        "Batch abgeschlossen: {} Dateien erfolgreich konvertiert.",
        ok
    );
}

/// Converts one DFF into one glTF (.gltf + .bin + textures if found).
fn convert_one(in_path: &Path, out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // 1) Build a unified report (already splits helpers vs. meshes)
    let report = rw_dff_model::unified_scan::build_unified_report(in_path)?;

    // 2) Determine output names & target folder structure: <stem>/<file>/gltf
    let stem = in_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let gltf_dir = out_dir.join(&stem);
    fs::create_dir_all(&gltf_dir)?;

    let gltf_path = gltf_dir.join(format!("{stem}.gltf"));
    let bin_name = format!("{stem}.bin");
    let bin_path = gltf_dir.join(&bin_name);

    // 3) Convert unified meshes to a glTF document + BIN buffer
    // (Implementation goes into libs/gltf_writer; here we just call it.)
    let (gltf_json, bin_bytes) = gltf_writer::from_unified(&stem, &bin_name, &report, &gltf_dir)?;

    // 4) Write files
    fs::write(&bin_path, bin_bytes)?;
    fs::write(&gltf_path, serde_json::to_vec_pretty(&gltf_json)?)?;

    Ok(())
}
