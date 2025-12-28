use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Wo liegt die texconv.exe – relativ zum Projekt?
    let texconv_src = PathBuf::from("../../tools/texconv.exe");

    // Zielverzeichnis herausfinden (target/debug oder target/release)
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR nicht gesetzt");
    let target_dir = PathBuf::from(out_dir)
        .ancestors()
        .nth(3) // OUT_DIR = target/debug/build/<crate>/out → wir wollen target/debug
        .expect("Konnte Zielverzeichnis nicht bestimmen")
        .to_path_buf();

    let texconv_dst = target_dir.join("texconv.exe");

    if !texconv_dst.exists() {
        fs::copy(&texconv_src, &texconv_dst).expect("❌ Fehler beim Kopieren von texconv.exe");
        println!("✅ texconv.exe nach {:?} kopiert", texconv_dst);
    } else {
        println!("ℹ️  texconv.exe existiert bereits in {:?}", texconv_dst);
    }
}
