use crate::extract_from_dat;
use std::path::{Path, PathBuf};

pub fn process_dat_files(files: &[PathBuf], destination_path: &str) {
    for dat_file in files.iter().filter(|f| {
        f.extension()
            .map_or(false, |ext| ext.eq_ignore_ascii_case("dat"))
    }) {
        let reference_file = dat_file.with_file_name("reference.dat");

        if dat_file.exists() && reference_file.exists() {
            if dat_file
                .file_name()
                .map_or(false, |name| name.eq_ignore_ascii_case("data.dat"))
            {
                if let Err(e) =
                    extract_from_dat(dat_file, &reference_file, Path::new(destination_path))
                {
                    eprintln!("❌ Fehler beim Extrahieren von {:?}: {e}", dat_file);
                }
            }
        }
    }
}
