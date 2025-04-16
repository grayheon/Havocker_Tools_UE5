use crate::extract_files;
use std::path::{Path, PathBuf};

pub fn process_dat_files(files: &[PathBuf], destination_path: &str) {
    for dat_file in files.iter().filter(|f| f.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("dat"))) {
        let reference_file = dat_file.with_file_name("reference.dat");

        if dat_file.exists() && reference_file.exists() {
            if dat_file.file_name().map_or(false, |name| name.eq_ignore_ascii_case("data.dat")) {
                extract_files(dat_file, &reference_file, Path::new(destination_path)).ok();
            } else {
                println!("⏭️  Überspringe {:?} (nicht data.dat)", dat_file);
            }
        }
    }
}
pub fn process_ma_files(files: &[PathBuf], destination_path: &str) {
    for file in files {
        let file_name = file.file_name().and_then(|f| f.to_str()).unwrap_or("").to_lowercase();
        if file_name.ends_with(".ma2") {
            let ref_file = file.with_extension("ma1");
            if file.exists() && ref_file.exists() {
                println!("📦 Entpacke {} mit {}", file_name, ref_file.file_name().unwrap().to_string_lossy());
                extract_files(file, &ref_file, Path::new(destination_path)).ok();
            } else {
                println!("⚠️  MA-Dateipaar nicht vollständig vorhanden: {}", file.display());
            }
        }
    }
}

