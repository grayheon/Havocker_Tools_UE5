use image::{self, ImageFormat};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

pub fn convert_txd_to_dds(root_path: &Path) -> io::Result<()> {
    println!(
        "🟦 Starte TXD → DDS Konvertierung in: {}",
        root_path.display()
    );

    for entry in WalkDir::new(root_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            let path = e.path();
            path.is_file()
                && path.extension().map_or(false, |ext| {
                    ext.eq_ignore_ascii_case("txd") || ext.eq_ignore_ascii_case("tx1")
                })
                && path.ancestors().any(|p| {
                    p.file_name()
                        .map_or(false, |n| n.eq_ignore_ascii_case("TXD"))
                })
        })
    {
        let path = entry.path();
        if let Err(e) = process_single_txd(path) {
            eprintln!("❌ Fehler bei {}: {}", path.display(), e);
        }
    }

    Ok(())
}

pub fn process_single_txd(txd_path: &Path) -> io::Result<()> {
    println!("▶️ Verarbeite TXD-Datei: {}", txd_path.display());

    let data = fs::read(txd_path)?;
    let file_len = data.len();
    if file_len < 0x90 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("TXD-Datei zu klein ({file_len} Bytes) - benötige mindestens 0x90"),
        ));
    }

    let header = &data[..0x90];

    let offset = 0x80;
    let dxt_bytes = &header[offset..offset + 4];
    let dxt_str = String::from_utf8_lossy(dxt_bytes).to_string();

    let dxt = u32::from_le_bytes([dxt_bytes[0], dxt_bytes[1], dxt_bytes[2], dxt_bytes[3]]);
    let width = u16::from_le_bytes([header[offset + 4], header[offset + 5]]);
    let height = u16::from_le_bytes([header[offset + 6], header[offset + 7]]);
    let mip_count = header[offset + 9].max(1);

    println!("   • Format: {dxt_str}, Größe: {width}x{height}, Mips: {mip_count}");

    if !matches!(dxt_str.as_str(), "DXT1" | "DXT3" | "DXT5") {
        // Versuch: PNG extrahieren (PNG-Signatur suchen)
        const PNG_SIG: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
        if let Some(pos) = data.windows(8).position(|w| w == PNG_SIG) {
            let png_bytes = &data[pos..];
            let img =
                image::load_from_memory_with_format(png_bytes, ImageFormat::Png).map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("PNG-Decode fehlgeschlagen: {e}"),
                    )
                })?;

            let txd_dir = txd_path.parent().unwrap_or_else(|| Path::new("."));
            let png_dir = txd_dir.with_file_name("PNG");
            fs::create_dir_all(&png_dir)?;

            let file_stem = txd_path
                .file_stem()
                .and_then(|f| f.to_str())
                .unwrap_or("unknown");
            let out_path = png_dir.join(format!("{file_stem}.png"));
            img.save(&out_path).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("PNG speichern fehlgeschlagen: {e}"),
                )
            })?;

            println!("✅ PNG extrahiert (embedded): {}", out_path.display());
            return Ok(());
        }

        // Fallback: heuristische RAW-RGBA (legacy)
        let mut raw_data = data.get(60..).unwrap_or(&[]).to_vec();
        let file_name = txd_path
            .file_stem()
            .and_then(|f| f.to_str())
            .unwrap_or("unknown");
        let first_char = file_name.chars().next().unwrap_or('x').to_ascii_lowercase();

        let (width_guess, height_guess) = match first_char {
            'm' | 'q' => (32, 32),
            _ => (50, 50),
        };
        const PIXEL_SIZE: usize = 4;
        let expected = width_guess * height_guess * PIXEL_SIZE;

        if raw_data.len() < expected {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Bilddaten zu klein für RAW-PNG-Erzeugung",
            ));
        }
        raw_data.truncate(expected);

        let txd_dir = txd_path.parent().unwrap_or_else(|| Path::new("."));
        let png_dir = txd_dir.with_file_name("PNG");
        fs::create_dir_all(&png_dir)?;

        let file_stem = txd_path
            .file_stem()
            .and_then(|f| f.to_str())
            .unwrap_or("unknown");
        let out_path = png_dir.join(format!("{file_stem}.png"));

        let mut img = image::ImageBuffer::new(width_guess as u32, height_guess as u32);
        for y in 0..height_guess {
            for x in 0..width_guess {
                let idx = (y * width_guess + x) * PIXEL_SIZE;
                let pixel = image::Rgba([
                    raw_data[idx],
                    raw_data[idx + 1],
                    raw_data[idx + 2],
                    raw_data[idx + 3],
                ]);
                img.put_pixel(x as u32, y as u32, pixel);
            }
        }

        img.save(&out_path).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("PNG-Speicherfehler: {e}"))
        })?;

        println!("✅ PNG (RAW) gespeichert: {}", out_path.display());
        return Ok(());
    }

    if width == 0 || height == 0 || width > 8192 || height > 8192 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Ungültige Bildgröße: {}x{}", width, height),
        ));
    }

    let block_size = if dxt_str == "DXT1" { 8 } else { 16 };

    // Alle Mipmaps einlesen (Daten liegen direkt nach 0x90 hintereinander)
    let mut mip_data: Vec<Vec<u8>> = Vec::new();
    let mut level_w = width as usize;
    let mut level_h = height as usize;
    let mut cursor = 0x90;
    for _ in 0..mip_count {
        let blocks_w = (level_w.max(1) + 3) / 4;
        let blocks_h = (level_h.max(1) + 3) / 4;
        let linear_size = blocks_w * blocks_h * block_size;
        if file_len < cursor + linear_size {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "TXD zu klein für alle Mipmaps",
            ));
        }
        mip_data.push(data[cursor..cursor + linear_size].to_vec());
        cursor += linear_size;
        level_w = (level_w / 2).max(1);
        level_h = (level_h / 2).max(1);
    }

    // DDS speichern
    let txd_dir = txd_path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Kein gültiger TXD-Pfad"))?;
    let dds_dir = txd_dir.with_file_name("DDS");
    fs::create_dir_all(&dds_dir)?;

    let file_stem = txd_path
        .file_stem()
        .and_then(|f| f.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Ungültiger Dateiname"))?;

    let dds_path = dds_dir.join(format!("{file_stem}.dds"));

    let mut dds_header = vec![0u8; 128];
    dds_header[0..4].copy_from_slice(b"DDS ");
    dds_header[4..8].copy_from_slice(&0x7C_u32.to_le_bytes());
    dds_header[8..12].copy_from_slice(&(0x0002100A_u32 | 0x20000).to_le_bytes()); // flags + mipmap flag
    dds_header[12..16].copy_from_slice(&(height as u32).to_le_bytes());
    dds_header[16..20].copy_from_slice(&(width as u32).to_le_bytes());
    dds_header[24..28].copy_from_slice(&(mip_count as u32).to_le_bytes());
    dds_header[76..80].copy_from_slice(&0x20_u32.to_le_bytes());
    dds_header[80..84].copy_from_slice(&0x04_u32.to_le_bytes());
    dds_header[84..88].copy_from_slice(&dxt.to_le_bytes());

    // Caps: texture + mipmaps
    dds_header[108..112].copy_from_slice(&0x00001008_u32.to_le_bytes()); // DDSCAPS_TEXTURE | DDSCAPS_COMPLEX
    dds_header[112..116].copy_from_slice(&0x00004000_u32.to_le_bytes()); // DDSCAPS2_MIPMAP

    let mut out_file = File::create(&dds_path)?;
    out_file.write_all(&dds_header)?;
    for level in &mip_data {
        out_file.write_all(level)?;
    }
    drop(out_file); // sicherstellen, dass texconv ohne File-Lock auf die DDS zugreifen kann

    println!("✅ DDS gespeichert (mit Mips): {}", dds_path.display());

    // Nach dem DDS-Export eine PNG-Version per texconv erzeugen
    let png_dir = txd_dir.with_file_name("PNG");
    fs::create_dir_all(&png_dir)?;
    if let Err(e) = dds_to_png(&dds_path, &png_dir) {
        eprintln!(
            "⚠️ PNG-Konvertierung für {} fehlgeschlagen: {}",
            dds_path.display(),
            e
        );
    }
    Ok(())
}

/// Konvertiert eine DDS per texconv in eine PNG-Datei im Zielordner.
/// Falls texconv nicht gefunden wird, gibt es nur eine Warnung.
fn dds_to_png(dds_path: &Path, out_dir: &Path) -> io::Result<()> {
    let texconv = find_texconv().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "texconv.exe nicht gefunden (tools/ oder neben der Binary erwartet)",
        )
    })?;

    let status = Command::new(&texconv)
        .args([
            "-ft",
            "PNG",
            "-o",
            out_dir.to_str().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "ungültiger PNG-Pfad")
            })?,
            dds_path.to_str().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "ungültiger DDS-Pfad")
            })?,
        ])
        .status()?;

    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("texconv exit code {}", status.code().unwrap_or(-1)),
        ));
    }

    Ok(())
}

/// Sucht texconv.exe an typischen Stellen (neben dem Binary oder im tools/-Ordner).
fn find_texconv() -> Option<PathBuf> {
    // 1) Neben der aktuell laufenden Binary (build.rs kopiert texconv dorthin)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let cand = dir.join("texconv.exe");
            if cand.exists() {
                return Some(cand);
            }
        }
    }

    // 2) Repo-Tools-Ordner
    let tools = Path::new("tools").join("texconv.exe");
    if tools.exists() {
        return Some(tools);
    }

    // 3) PATH-Fallback
    Some(PathBuf::from("texconv.exe"))
}
