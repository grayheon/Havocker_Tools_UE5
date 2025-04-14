use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Ergebnisstruktur für den Texturnamen-Scan
struct TextureScanResult {
    valid: Vec<String>,
    invalid: Vec<String>,
}

/// Hauptfunktion: scannt DFFs und schreibt _textures.txt-Dateien mit validierten Texturnamen
pub fn scan_dff_textures(scan_root: &Path) -> std::io::Result<()> {
    let dff_files = find_dff_files(scan_root)?;
    let known_textures = collect_png_texture_names(scan_root)?;

    println!("🔍 {} DFF-Dateien gefunden.", dff_files.len());
    println!("🔎 {} .png Texturnamen referenzierbar.\n", known_textures.len());

    for dff_path in dff_files {
        let stem = dff_path.file_stem().unwrap_or_default().to_string_lossy();
        let txt_path = dff_path.with_file_name(format!("{stem}_textures.txt"));

        let scan_result = extract_texture_names_from_file(&dff_path, &known_textures)?;

        let mut writer = BufWriter::new(File::create(&txt_path)?);
        for name in &scan_result.valid {
            writeln!(writer, "{name}")?;
        }

        println!("✅ {} → {} Texturnamen gespeichert", txt_path.display(), scan_result.valid.len());

        if !scan_result.invalid.is_empty() {
            println!("⚠️  {} ungültige / nicht vorhandene Texturnamen ignoriert:", scan_result.invalid.len());
            for name in &scan_result.invalid {
                println!("   • {name}");
            }
        }

        println!();
    }

    Ok(())
}

fn find_dff_files(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    Ok(WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                .extension()
                .map_or(false, |ext| ext.eq_ignore_ascii_case("dff"))
        })
        .map(|e| e.into_path())
        .collect())
}

fn collect_png_texture_names(root: &Path) -> std::io::Result<HashSet<String>> {
    let mut result = HashSet::new();
    let texture_root = root.join("texture");

    for entry in WalkDir::new(&texture_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                .extension()
                .map_or(false, |ext| ext.eq_ignore_ascii_case("png"))
        })
    {
        if let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str()) {
            result.insert(stem.to_lowercase());
        }
    }

    Ok(result)
}

fn extract_texture_names_from_file(path: &Path, known_textures: &HashSet<String>) -> std::io::Result<TextureScanResult> {
    let mut file = File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(extract_ascii_texture_names(&data, known_textures))
}

fn extract_ascii_texture_names(data: &[u8], known: &HashSet<String>) -> TextureScanResult {
    let mut valid = HashSet::new();
    let mut invalid = HashSet::new();
    let mut current = Vec::new();

    for &byte in data {
        if byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-' {
            current.push(byte);
        } else {
            if current.len() >= 8 && current.len() <= 9 {
                if let Ok(s) = String::from_utf8(current.clone()) {
                    let s = s.to_lowercase();
                    let first = s.chars().next().unwrap_or('_');
                    if first.is_ascii_alphabetic() && s.chars().all(|c| c.is_ascii_alphanumeric()) {
                        if known.contains(&s) {
                            valid.insert(s);
                        } else {
                            invalid.insert(s);
                        }
                    }
                }
            }
            current.clear();
        }
    }

    TextureScanResult {
        valid: {
            let mut v: Vec<_> = valid.into_iter().collect();
            v.sort();
            v
        },
        invalid: {
            let mut i: Vec<_> = invalid.into_iter().collect();
            i.sort();
            i
        },
    }
}
