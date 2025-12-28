use crate::conversion::convert_txd_to_dds;
use std::{fs, path::Path};
use walkdir::WalkDir;

pub fn process_txd_pipeline(destination_path: &str) -> std::io::Result<()> {
    let path = Path::new(destination_path);
    organize_txd_structure(path)?;
    println!("🟡 Starte TXD → DDS Konvertierung...");
    convert_txd_to_dds(path)?;
    println!("🟢 TXD → DDS Konvertierung beendet.");
    Ok(())
}

pub fn organize_txd_structure(destination_root: &Path) -> std::io::Result<()> {
    let skip_folders = ["TXD", "PNG", "DDS"];
    let mut last_dir: Option<std::path::PathBuf> = None;
    let mut moved_count = 0;

    for entry in WalkDir::new(destination_root)
        .into_iter()
        .flatten()
        .filter(|e| {
            let path = e.path();
            let is_file = path.is_file();
            let is_txd = path
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("txd") || ext.eq_ignore_ascii_case("tx1"))
                .unwrap_or(false);
            let in_skipped_folder = path.ancestors().any(|p| {
                p.file_name()
                    .and_then(|f| f.to_str())
                    .map(|n| skip_folders.contains(&n))
                    .unwrap_or(false)
            });

            is_file && is_txd && !in_skipped_folder
        })
    {
        let file_path = entry.path();
        let parent = file_path.parent().unwrap_or(destination_root);

        if Some(parent.to_path_buf()) != last_dir {
            println!("\n📁 Verarbeite Ordner: {}", parent.display());
            last_dir = Some(parent.to_path_buf());
        }

        let txd_folder = parent.join("TXD");
        fs::create_dir_all(&txd_folder)?;

        let file_name = file_path.file_name().unwrap();
        let target_path = parent.join("TXD").join(file_name);

        fs::rename(file_path, &target_path)?;
        println!(
            "↪️  Verschoben: {} → {}",
            file_path.display(),
            target_path.display()
        );
        moved_count += 1;
    }

    println!("\n✅ TXD-Struktur fertig. Insgesamt verschoben: {moved_count} Dateien.");
    Ok(())
}
