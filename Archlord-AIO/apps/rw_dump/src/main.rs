use rw_dff::json::to_json;
use rw_dff::parse_file;
use std::env;
use std::path::PathBuf;

/// cargo run -p rw_dump -- h0000a00.dff > h0000a00.tree.json

/// Simple RenderWare chunk tree dumper.
///
/// # Usage
/// ```text
/// rw_dump <path-to-dff-or-txd>
/// ```
///
/// # Output
/// - Writes a deterministic JSON representation of the RW chunk tree to stdout.
/// - Intended for golden-file generation and debugging.
///
/// # Philosophy
/// - No guessing
/// - No semantic interpretation
/// - If parsing fails, fail loudly
fn main() {
    let mut args = env::args_os();
    let exe = args.next();

    let path = match args.next() {
        Some(p) => PathBuf::from(p),
        None => {
            eprintln!(
                "Usage: {} <path-to-file>",
                exe.unwrap_or_default().to_string_lossy()
            );
            std::process::exit(1);
        }
    };

    match parse_file(&path) {
        Ok(root) => {
            let json = to_json(&root);
            let out = serde_json::to_string_pretty(&json)
                .expect("failed to serialize chunk tree to JSON");
            println!("{}", out);
        }
        Err(e) => {
            eprintln!("Error parsing {:?}: {}", path, e);
            std::process::exit(2);
        }
    }
}
