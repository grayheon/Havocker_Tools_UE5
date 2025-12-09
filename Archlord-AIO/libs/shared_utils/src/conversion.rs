use std::path::Path;
use std::process::Command;
use std::{fs, io};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use walkdir::WalkDir;
use rayon::prelude::*;

pub fn convert_txd_to_dds(root_path: &Path) -> io::Result<()> {
    println!("🔄 Starte TXD → DDS Konvertierung in: {}", root_path.display());

    for entry in WalkDir::new(root_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            let path = e.path();
            path.is_file()
                && path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("txd"))
                && path
                .ancestors()
                .any(|p| p.file_name().map_or(false, |n| n.eq_ignore_ascii_case("TXD")))
        })
    {
        let path = entry.path();
        if let Err(e) = process_single_txd(path) {
            eprintln!("❌ Fehler bei {}: {}", path.display(), e);
        }
    }

    Ok(())
}

pub fn batch_convert_dds_to_png(destination_root: &Path) -> io::Result<()> {
    println!("🟡 Starte DDS → PNG Konvertierung...");

    let tex_conv_path = "texconv.exe";
    if !fs::metadata(tex_conv_path).is_ok() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "❌ texconv.exe nicht gefunden!",
        ));
    }

    // 1. Alle DDS-Dateien sammeln
    let mut dds_files = Vec::new();
    for entry in WalkDir::new(destination_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.path()
                .extension()
                .map_or(false, |ext| ext.eq_ignore_ascii_case("dds"))
        })
    {
        dds_files.push(entry.path().to_owned());
    }

    // 2. Parallel verarbeiten
    let errors: Vec<_> = dds_files
        .par_iter()
        .filter_map(|dds_path| {
            let png_dir = if dds_path
                .ancestors()
                .any(|p| p.file_name().map_or(false, |f| f.eq_ignore_ascii_case("DDS")))
            {
                dds_path
                    .parent()
                    .and_then(|p| p.parent())
                    .map(|txd_dir| txd_dir.join("PNG"))
                    .unwrap_or_else(|| Path::new(".").to_path_buf())
            } else {
                dds_path
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| Path::new(".").to_path_buf())
            };

            if let Err(e) = fs::create_dir_all(&png_dir) {
                return Some(format!(
                    "❌ Fehler beim Erstellen von {}: {}",
                    png_dir.display(),
                    e
                ));
            }

            if let Err(e) = convert_dds_to_png(dds_path, &png_dir, tex_conv_path) {
                return Some(format!(
                    "❌ Fehler bei {}: {}",
                    dds_path.display(),
                    e
                ));
            }

            None::<String>
        })
        .collect();

    for msg in errors {
        eprintln!("{}", msg);
    }

    println!("✅ DDS → PNG Konvertierung abgeschlossen.");
    Ok(())
}

// 👇 Helferfunktionen ↓ (gekürzt aus deiner Datei)
fn convert_dds_to_png(dds_path: &Path, png_target_dir: &Path, tex_conv_path: &str) -> io::Result<()> {
    let filename = dds_path.file_stem().unwrap_or_default().to_string_lossy();
    let output_path = png_target_dir.to_str().ok_or_else(|| {
        io::Error::new(io::ErrorKind::Other, "Ungültiger Zielpfad")
    })?;

    let status = Command::new(tex_conv_path)
        .args(["-ft", "PNG", "-o", output_path, dds_path.to_str().unwrap()])
        .status()
        .map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Fehler beim Aufruf texconv.exe: {e}"))
        })?;

    if status.success() {
        println!("✅ PNG erstellt mit texconv.exe: {}", filename);
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("texconv.exe fehlgeschlagen für {}", dds_path.display()),
        ))
    }
}

pub fn process_single_txd(txd_path: &Path) -> io::Result<()> {
    println!("📄 Verarbeite TXD-Datei: {}", txd_path.display());

    let file_meta = fs::metadata(txd_path)?;
    let file_len = file_meta.len() as usize;
    if file_len < 0x90 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("TXD-Datei zu klein ({file_len} Bytes) – benötige mindestens 0x90"),
        ));
    }

    let mut file = File::open(txd_path)?;
    println!("📂 Datei erfolgreich geöffnet");

    let mut header = [0u8; 0x90];
    file.read_exact(&mut header)?;
    println!("📥 Header erfolgreich gelesen");

    let offset = 0x80;
    let dxt_bytes = &header[offset..offset + 4];
    let dxt_str = String::from_utf8_lossy(dxt_bytes).to_string();

    let dxt = u32::from_le_bytes([dxt_bytes[0], dxt_bytes[1], dxt_bytes[2], dxt_bytes[3]]);

    let width = u16::from_le_bytes([header[offset + 4], header[offset + 5]]);
    let height = u16::from_le_bytes([header[offset + 6], header[offset + 7]]);
    let mips = header[offset + 9];

    println!(
        "↪️ Format: {}, Größe: {}x{}, Mips: {}",
        dxt_str, width, height, mips
    );

    match dxt_str.as_str() {
        "DXT1" | "DXT3" | "DXT5" => {
            if width == 0 || height == 0 || width > 8192 || height > 8192 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Ungültige Bildgröße: {}x{}", width, height),
                ));
            }
            let block_size = if dxt_str == "DXT1" { 8 } else { 16 };
            let blocks_w = ((width + 3) / 4) as usize;
            let blocks_h = ((height + 3) / 4) as usize;
            let linear_size = blocks_w * blocks_h * block_size;

            let required = 0x90usize + linear_size;
            if file_len < required {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    format!(
                        "TXD zu klein für DXT-Daten: benötigt mindestens {required} Bytes, hat {file_len}"
                    ),
                ));
            }

            file.seek(SeekFrom::Start(0x90))?;
            let mut image_data = vec![0u8; linear_size];
            file.read_exact(&mut image_data)?;
            println!("📦 Bilddaten gelesen: {} Bytes", image_data.len());

            // DDS speichern
            let txd_dir = txd_path.parent().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Kein gültiger TXD-Pfad")
            })?;
            let dds_dir = txd_dir.with_file_name("DDS");
            fs::create_dir_all(&dds_dir)?;

            let file_stem = txd_path
                .file_stem()
                .and_then(|f| f.to_str())
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "Ungültiger Dateiname")
                })?;

            let dds_path = dds_dir.join(format!("{file_stem}.dds"));

            let mut dds_header = vec![0u8; 128];
            dds_header[0..4].copy_from_slice(b"DDS ");
            dds_header[4..8].copy_from_slice(&0x7C_u32.to_le_bytes());
            dds_header[8..12].copy_from_slice(&0x0002100A_u32.to_le_bytes());
            dds_header[12..16].copy_from_slice(&(height as u32).to_le_bytes());
            dds_header[16..20].copy_from_slice(&(width as u32).to_le_bytes());
            dds_header[76..80].copy_from_slice(&0x20_u32.to_le_bytes());
            dds_header[80..84].copy_from_slice(&0x04_u32.to_le_bytes());
            dds_header[84..88].copy_from_slice(&dxt.to_le_bytes());

            let mut out_file = File::create(&dds_path)?;
            out_file.write_all(&dds_header)?;
            out_file.write_all(&image_data)?;

            println!("✅ DDS gespeichert: {}", dds_path.display());

            // PNG speichern über DDS-Konvertierung
            let png_dir = txd_dir.with_file_name("PNG");
            fs::create_dir_all(&png_dir)?;
        }
        _ => {
            // Wenn kein DXT-Format, versuche rohe RGBA-Bilddaten ab Offset 0x60 zu interpretieren
            file.seek(SeekFrom::Start(60))?;
            let mut raw_data = Vec::new();
            file.read_to_end(&mut raw_data)?;

            let file_name = txd_path
                .file_stem()
                .and_then(|f| f.to_str())
                .unwrap_or("unknown");
            let first_char = file_name.chars().next().unwrap_or('x').to_ascii_lowercase();

            let (width, height) = match first_char {
                'm' | 'q' => (32, 32),
                _ => (50, 50),
            };
            const PIXEL_SIZE: usize = 4;
            let expected = width * height * PIXEL_SIZE;

            if raw_data.len() < expected {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Bilddaten zu klein für RAW-PNG-Erzeugung",
                ));
            }

            let txd_dir = txd_path.parent().unwrap_or_else(|| Path::new("."));
            let png_dir = txd_dir.with_file_name("PNG");
            fs::create_dir_all(&png_dir)?;

            let file_stem = txd_path
                .file_stem()
                .and_then(|f| f.to_str())
                .unwrap_or("unknown");
            let out_path = png_dir.join(format!("{}.png", file_stem));

            let mut img = image::ImageBuffer::new(width as u32, height as u32);
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) * PIXEL_SIZE;
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
        }
    }

    Ok(())
}
