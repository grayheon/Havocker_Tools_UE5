use rw_dff::ids;
use rw_dff::parse_file;
use rw_dff::tree::RwChunkNode;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};
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
    println!(
        "🔎 {} .dds Texturnamen referenzierbar.\n",
        known_textures.len()
    );

    for dff_path in dff_files {
        let stem = dff_path.file_stem().unwrap_or_default().to_string_lossy();
        let txt_path = dff_path.with_file_name(format!("{stem}_textures.txt"));

        let scan_result = extract_texture_names_from_file(&dff_path, &known_textures)?;

        let mut writer = BufWriter::new(File::create(&txt_path)?);
        for name in &scan_result.valid {
            writeln!(writer, "{name}")?;
        }

        println!(
            "✅ {} → {} Texturnamen gespeichert",
            txt_path.display(),
            scan_result.valid.len()
        );

        if !scan_result.invalid.is_empty() {
            println!(
                "⚠️  {} ungültige / nicht vorhandene Texturnamen ignoriert:",
                scan_result.invalid.len()
            );
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
                && e.path().extension().map_or(false, |ext| {
                    ext.eq_ignore_ascii_case("dff") || ext.eq_ignore_ascii_case("ecl")
                })
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
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("dds"))
        })
    {
        if let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str()) {
            result.insert(stem.to_lowercase());
        }
    }

    Ok(result)
}

fn extract_texture_names_from_file(
    path: &Path,
    known_textures: &HashSet<String>,
) -> std::io::Result<TextureScanResult> {
    let root_node =
        parse_file(path).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let file_bytes = std::fs::read(path)?;

    let mut names = HashSet::new();
    collect_texture_names_from_node(&root_node, &file_bytes, &mut names);

    let mut valid = HashSet::new();
    let mut invalid = HashSet::new();

    for name in names {
        let name_lower = name.to_lowercase();
        if known_textures.contains(&name_lower) {
            valid.insert(name_lower);
        } else {
            invalid.insert(name_lower);
        }
    }

    Ok(TextureScanResult {
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
    })
}

fn collect_texture_names_from_node(
    node: &RwChunkNode,
    file_bytes: &[u8],
    out: &mut HashSet<String>,
) {
    if node.header.id == ids::RW_TEXTURE {
        if let Ok(info) = rw_dff_model::texture::decode_texture_node(node, file_bytes) {
            if let Some(name) = info.name {
                if !name.is_empty() {
                    out.insert(name);
                }
            }
        }
    }

    for child in &node.children {
        collect_texture_names_from_node(child, file_bytes, out);
    }
}
