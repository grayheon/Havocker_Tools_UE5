use std::env;
use std::path::PathBuf;

/// cargo run -p rw_model_dump -- h0000a00.dff > h0000a00.model.json
///
fn main() {
    let mut args = env::args_os();
    let exe = args.next();

    let path = match args.next() {
        Some(p) => PathBuf::from(p),
        None => {
            eprintln!(
                "Usage: {} <path-to-dff-or-txd>",
                exe.unwrap_or_default().to_string_lossy()
            );
            std::process::exit(1);
        }
    };

    match rw_dff_model::build_report(&path) {
        Ok(report) => {
            let out =
                serde_json::to_string_pretty(&report).expect("failed to serialize report to JSON");
            println!("{}", out);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(2);
        }
    }
}
