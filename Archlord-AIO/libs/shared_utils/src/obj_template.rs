use configparser::ini::Ini;
use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn load_ini_euckr(path: &Path) -> std::io::Result<Ini> {
    let file = File::open(path)?;
    let mut reader = DecodeReaderBytesBuilder::new()
        .encoding(Some(EUC_KR))
        .build(file);

    let mut ini = Ini::new();
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    ini.read(content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(ini)
}

pub fn process_obj_templates(destination_root: &Path) -> std::io::Result<()> {
    let object_template_path = find_objecttemplate(destination_root)?;
    let object_template = load_ini_euckr(&object_template_path)?;
    let all_dff_files = collect_all_dff_names(destination_root)?;

    for entry in WalkDir::new(destination_root)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            if name.starts_with("obj") && name.ends_with(".ini") && name.len() == 12 {
                println!("📄 Verarbeite Datei: {}", path.display());

                let base = get_base_name(path).unwrap_or_else(|| "output".to_string());
                let target_dir = path.parent().unwrap_or(Path::new(".")).join("OBJ");
                fs::create_dir_all(&target_dir)?;

                let csv_path = target_dir.join(format!("{base}.csv"));
                let proof_path = target_dir.join(format!("{base}_DFF_Proof_SUCCESS.txt"));
                let error_path = target_dir.join(format!("{base}_DFF_Proof_ERROR.txt"));
                let ini = load_ini_euckr(path)?;

                let mut csv_file = BufWriter::new(File::create(&csv_path)?);
                let mut proof_entries = Vec::new();
                let mut error_entries = Vec::new();

                writeln!(
                    csv_file,
                    "ID,TID,DFF,COLLISION_DFF,PICK_DFF,SCALE_X,SCALE_Y,SCALE_Z,POS_X,POS_Y,POS_Z,DEGREE_X,DEGREE_Y,BLUEPRINT_CLASS"
                )?;

                let mut written_rows = 0;

                for section in ini.sections() {
                    if let Some(tid) = ini.get(&section, "TID") {
                        let dff = extract_clean_value(object_template.get(&tid, "10").as_ref());
                        let col = extract_clean_value(object_template.get(&tid, "5").as_ref());
                        let pick = extract_clean_value(object_template.get(&tid, "25").as_ref());

                        let mut scalex = String::new();
                        let mut scaley = String::new();
                        let mut scalez = String::new();
                        let mut posx = String::new();
                        let mut posy = String::new();
                        let mut posz = String::new();
                        let degx = ini.get(&section, "DegreeX").unwrap_or_default();
                        let degy = ini.get(&section, "DegreeY").unwrap_or_default();

                        if let Some(scale) = ini.get(&section, "Scale") {
                            let parts: Vec<&str> = scale.split(',').collect();
                            if parts.len() == 3 {
                                scalex = parts[0].to_string();
                                scaley = parts[1].to_string();
                                scalez = parts[2].to_string();
                            }
                        }

                        if let Some(pos) = ini.get(&section, "Position") {
                            let parts: Vec<&str> = pos.split(',').collect();
                            if parts.len() == 3 {
                                posx = parts[0].to_string();
                                posy = parts[1].to_string();
                                posz = parts[2].to_string();
                            }
                        }

                        writeln!(
                            csv_file,
                            "{section},{tid},{dff},{col},{pick},{scalex},{scaley},{scalez},{posx},{posy},{posz},{degx},{degy},"
                        )?;
                        written_rows += 1;

                        let dff_key = dff.to_ascii_uppercase();
                        if all_dff_files.contains(&dff_key) {
                            proof_entries.push(format!("{tid}: {dff} → OK"));
                        } else {
                            error_entries.push(format!("{tid}: {dff} fehlt"));
                        }
                    }
                }

                if !proof_entries.is_empty() {
                    let mut proof_file = BufWriter::new(File::create(&proof_path)?);
                    for entry in proof_entries {
                        writeln!(proof_file, "{}", entry)?;
                    }
                }

                if !error_entries.is_empty() {
                    let mut error_file = BufWriter::new(File::create(&error_path)?);
                    for entry in error_entries {
                        writeln!(error_file, "{}", entry)?;
                    }
                }

                let section_count = ini
                    .sections()
                    .iter()
                    .filter(|s| s.to_ascii_lowercase() != "header" && ini.get(s, "TID").is_some())
                    .count();

                if written_rows != section_count {
                    println!(
                        "⚠️  Warnung: INI-Sektionen: {}, CSV-Zeilen: {} → Unterschied!",
                        section_count, written_rows
                    );
                } else {
                    println!("✅ CSV enthält exakt alle {} Sektionen.", section_count);
                }
            }
        }
    }

    Ok(())
}

pub fn validate_tids_against_objecttemplate(destination_root: &Path) -> std::io::Result<()> {
    let object_template_path = find_objecttemplate(destination_root)?;
    let object_template = load_ini_euckr(&object_template_path)?;
    let mut all_tids = HashSet::new();

    for entry in WalkDir::new(destination_root)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if name.starts_with("obj") && name.ends_with(".ini") && name.len() == 12 {
                    let ini = load_ini_euckr(path)?;
                    for section in ini.sections() {
                        if let Some(tid) = ini.get(&section, "TID") {
                            all_tids.insert(tid);
                        }
                    }
                }
            }
        }
    }

    let mut valid = Vec::new();
    let mut missing = Vec::new();

    for tid in &all_tids {
        if object_template.sections().contains(tid) {
            valid.push(tid.clone());
        } else {
            missing.push(tid.clone());
        }
    }

    valid.sort();
    missing.sort();

    let target_dir = destination_root.join("ini/OBJ");
    fs::create_dir_all(&target_dir)?;

    let mut valid_file = BufWriter::new(File::create(target_dir.join("TID_Valid.txt"))?);
    let mut missing_file = BufWriter::new(File::create(target_dir.join("TID_Missing.txt"))?);

    for v in &valid {
        writeln!(valid_file, "{}", v)?;
    }
    for m in &missing {
        writeln!(missing_file, "{}", m)?;
    }

    println!(
        "✅ TID Prüfung abgeschlossen: {} gültig, {} fehlen",
        valid.len(),
        missing.len()
    );

    Ok(())
}

pub fn extract_and_check_dff_files(destination_root: &Path) -> std::io::Result<()> {
    let object_template_path = find_objecttemplate(destination_root)?;
    let object_template = load_ini_euckr(&object_template_path)?;
    let mut dff_names = HashSet::new();

    for section in object_template
        .sections()
        .iter()
        .filter(|s| s.to_ascii_lowercase() != "header")
    {
        for key in ["5", "10", "25"] {
            if let Some(value) = object_template.get(&section, key) {
                let dff = extract_clean_value(Some(&value));
                if !dff.is_empty()
                    && !["DFF", "COLLISION_DFF", "PICK_DFF"].contains(&dff.as_str())
                    && !dff.to_ascii_uppercase().contains("AGCM")
                    && dff.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                {
                    dff_names.insert(dff.to_ascii_uppercase());
                }
            }
        }
    }

    let found_dffs = collect_all_dff_names(&destination_root.join("object"))?;
    let mut existing = Vec::new();
    let mut missing = Vec::new();

    for dff in &dff_names {
        if found_dffs.contains(dff) {
            existing.push(dff.clone());
        } else {
            missing.push(dff.clone());
        }
    }

    existing.sort();
    missing.sort();

    let target_dir = destination_root.join("ini/OBJ");
    fs::create_dir_all(&target_dir)?;

    let mut found_file = BufWriter::new(File::create(target_dir.join("DFF_Found.txt"))?);
    for f in &existing {
        writeln!(found_file, "{}", f)?;
    }

    if !missing.is_empty() {
        let mut missing_file = BufWriter::new(File::create(target_dir.join("DFF_Missing.txt"))?);
        for m in &missing {
            writeln!(missing_file, "{}", m)?;
        }
    }

    println!("✅ DFF Scan abgeschlossen:");
    println!("   🔹 Einzigartige DFF-Namen: {}", dff_names.len());
    println!("   ✅ Gefunden: {}", existing.len());
    println!("   ❌ Fehlend: {}", missing.len());

    Ok(())
}

// Hilfsfunktionen
fn extract_clean_value(opt: Option<&String>) -> String {
    if let Some(value) = opt {
        let cleaned = value
            .split(|c| c == ':' || c == ' ')
            .last()
            .unwrap_or("")
            .split('.')
            .next()
            .unwrap_or("");
        cleaned.to_string()
    } else {
        String::new()
    }
}

fn get_base_name(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

fn find_objecttemplate(destination_root: &Path) -> std::io::Result<PathBuf> {
    WalkDir::new(destination_root)
        .into_iter()
        .filter_map(Result::ok)
        .find(|e| e.file_type().is_file() && e.file_name() == "objecttemplate.ini")
        .map(|e| e.into_path())
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "objecttemplate.ini nicht gefunden",
            )
        })
}

fn collect_all_dff_names(destination_root: &Path) -> std::io::Result<HashSet<String>> {
    let mut result = HashSet::new();
    for entry in WalkDir::new(destination_root)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext.eq_ignore_ascii_case("dff") || ext.eq_ignore_ascii_case("ecl") {
                    if let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str()) {
                        result.insert(stem.to_ascii_uppercase());
                    }
                }
            }
        }
    }
    Ok(result)
}
