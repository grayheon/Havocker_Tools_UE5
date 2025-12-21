use std::env;
use std::path::PathBuf;

/// cargo run -p rw_unified_dump -- h0000a00.dff > h0000a00.unified.json
/// cargo run -p rw_unified_dump -- h04e0200.dff > h04e0200.unified.json

/// Dumps an export-ready unified mesh representation.
///
/// # Usage
/// rw_unified_dump <path-to-dff>
///
/// # Output
/// - Deterministic JSON for golden-file testing.
fn main() {
    let mut args = env::args_os();
    let exe = args.next();

    let path = match args.next() {
        Some(p) => PathBuf::from(p),
        None => {
            eprintln!(
                "Usage: {} <path-to-dff>",
                exe.unwrap_or_default().to_string_lossy()
            );
            std::process::exit(1);
        }
    };

    match rw_dff_model::unified_scan::build_unified_report(&path) {
        Ok(report) => {
            let out = serde_json::to_string_pretty(&report).expect("failed to serialize JSON");
            println!("{out}");
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(2);
        }
    }
}
