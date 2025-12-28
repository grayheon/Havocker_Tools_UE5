use shared_utils::{ensure_config_file, load_paths_from_config};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use encoding_rs::EUC_KR;

fn main() {
    ensure_config_file().expect("Fehler beim Initialisieren der config.ini");

    if let Err(e) = run() {
        eprintln!(
            "❌ Fehler beim CSV-Export:
{e}"
        );
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let (_source, destination) = load_paths_from_config();
    let root = Path::new(&destination);

    if !root.exists() {
        eprintln!("❌ Export-Ordner existiert nicht: {}", root.display());
        return Ok(());
    }

    let mut count = 0usize;
    for path in find_text_like_files(root) {
        if let Err(e) = process_text_file(&path) {
            eprintln!("⚠️ Fehler bei {}: {e}", path.display());
        } else {
            count += 1;
        }
    }

    println!(
        "✅ CSV-Prüfung abgeschlossen. {} Dateien verarbeitet.",
        count
    );
    Ok(())
}

fn find_text_like_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_text_files_recursive(root, &mut files);
    files
}

fn collect_text_files_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_text_files_recursive(&path, out);
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext = ext.to_lowercase();
                if ext == "ini" || ext == "txt" {
                    out.push(path);
                }
            }
        }
    }
}

fn decode_text(bytes: &[u8]) -> String {
    // 0. BOM-Erkennung (UTF-8 / UTF-16 LE / UTF-16 BE)
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        // UTF-8 mit BOM: BOM überspringen
        if let Ok(s) = String::from_utf8(bytes[3..].to_vec()) {
            return s;
        }
    } else if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        // UTF-16 LE
        let u16_len = (bytes.len() - 2) / 2;
        let mut buf = Vec::with_capacity(u16_len);
        let mut i = 2;
        while i + 1 < bytes.len() {
            let lo = bytes[i] as u16;
            let hi = bytes[i + 1] as u16;
            buf.push(lo | (hi << 8));
            i += 2;
        }
        if let Ok(s) = String::from_utf16(&buf) {
            return s;
        }
    } else if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        // UTF-16 BE
        let u16_len = (bytes.len() - 2) / 2;
        let mut buf = Vec::with_capacity(u16_len);
        let mut i = 2;
        while i + 1 < bytes.len() {
            let hi = bytes[i] as u16;
            let lo = bytes[i + 1] as u16;
            buf.push((hi << 8) | lo);
            i += 2;
        }
        if let Ok(s) = String::from_utf16(&buf) {
            return s;
        }
    }

    // 1. Versuche echtes UTF-8 ohne BOM / ohne Verlust
    if let Ok(s) = String::from_utf8(bytes.to_vec()) {
        return s;
    }

    // 2. Versuche EUC-KR (typisch für koreanische Archlord-Dateien)
    let (cow, _, had_errors) = EUC_KR.decode(bytes);
    if !had_errors {
        return cow.into_owned();
    }

    // 3. Fallback: lossy UTF-8, um wenigstens Struktur (Tabs/Kommas) zu erhalten
    String::from_utf8_lossy(bytes).into_owned()
}

fn process_text_file(path: &Path) -> io::Result<()> {
    let mut file = fs::File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let buffer = decode_text(&bytes);

    if buffer.is_empty() {
        return Ok(());
    }

    if let Some(delim) = detect_delimiter(&buffer) {
        let csv_path = path.with_extension("csv");

        if csv_path.exists() {
            println!(
                "ℹ️  CSV bereits vorhanden, überspringe: {}",
                csv_path.display()
            );
            return Ok(());
        }

        // Wir wandeln echte Tab-Tabellen in eine Excel-freundliche CSV um.
        // Separator im CSV ist ein Semikolon ';' (üblich bei deutschem Excel).
        //
        // Wichtig: Inhalte innerhalb einer Spalte (auch wenn sie ';', ':' oder
        // ',' enthalten) werden korrekt in Quotes gesetzt. Dadurch bleiben
        // Auflistungen wie "1;2;3" oder "190;5" vollständig erhalten und
        // stören die Spaltenerkennung nicht.

        let mut out = String::new();

        for line in buffer.lines() {
            let trimmed = line.trim_end_matches(['\r', '\n']);
            if trimmed.is_empty() {
                out.push_str("\n");
                continue;
            }

            let cols: Vec<&str> = if delim == '\t' {
                trimmed.split('\t').collect()
            } else {
                // Fallback: falls irgendwann ein anderes Delim genutzt wird,
                // behandeln wir die Zeile als eine einzige Spalte.
                vec![trimmed]
            };

            for (idx, col) in cols.iter().enumerate() {
                if idx > 0 {
                    out.push(';');
                }

                let needs_quotes = col.contains([';', ',', '\n', '\r', '"']);
                if needs_quotes {
                    out.push('"');
                    for ch in col.chars() {
                        if ch == '"' {
                            // CSV-Spezifikation: Quotes werden durch Verdopplung escaped
                            out.push('"');
                        }
                        out.push(ch);
                    }
                    out.push('"');
                } else {
                    out.push_str(col);
                }
            }

            out.push('\n');
        }

        let content = out;

        if let Some(parent) = csv_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&csv_path, content)?;
        println!(
            "📄 CSV-Datei erzeugt: {} (aus {})",
            csv_path.display(),
            path.display()
        );
    }

    Ok(())
}

fn detect_delimiter(content: &str) -> Option<char> {
    // Hinweis: Wir unterstützen hier bewusst **nur Tab** als strukturelles
    // Trennzeichen. Kommas, Semikolons oder Doppelpunkte kommen in den
    // Archlord-INI-Dateien häufig als Listen innerhalb einer Spalte vor
    // (z.B. "1;2;3" oder "190;5"). Diese sollen unverändert bleiben und
    // nicht als CSV-Delimiter gewertet werden.

    let mut tab_lines = 0usize;

    // Wir zählen für jede mögliche Spaltenanzahl, wie oft sie vorkommt,
    // und verwenden danach die häufigste. So werden große Tabellen wie
    // itemdatatable/characterdatatable robuster erkannt, auch wenn
    // einzelne Zeilen abweichende Spaltenanzahlen haben.
    use std::collections::HashMap;
    let mut tab_hist: HashMap<usize, usize> = HashMap::new();

    for line in content.lines().take(200) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('#') || trimmed.starts_with(';') || trimmed.starts_with("//") {
            continue;
        }

        // INI-typische Zeilen (Sektionen, Key-Value) für die Delimiter-Erkennung ignorieren
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            continue;
        }
        if trimmed.contains('=') {
            continue;
        }

        if trimmed.contains('\t') {
            let cols: Vec<&str> = trimmed.split('\t').collect();
            if cols.len() > 1 {
                tab_lines += 1;
                *tab_hist.entry(cols.len()).or_insert(0) += 1;
            }
        }
    }

    let best_tab = tab_hist.values().copied().max().unwrap_or(0);

    if best_tab >= 2 && tab_lines >= 2 {
        Some('\t')
    } else {
        None
    }
}
