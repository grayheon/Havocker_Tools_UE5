use image::{GenericImage, GenericImageView, ImageBuffer, Rgba};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn generate_world_map(destination_root: &Path) -> std::io::Result<()> {
    let minimap_dir = find_minimap_folder(destination_root)?;
    println!("📁 minimap-Ordner gefunden: {}", minimap_dir.display());

    let map_tiles = collect_map_tiles(&minimap_dir)?;
    let tile_size = find_tile_dimensions(&map_tiles)?;
    let big_tile_size = (tile_size.0 * 2, tile_size.1 * 2);

    let full_width = big_tile_size.0 * 16;
    let full_height = big_tile_size.1 * 14;
    let mut worldmap = ImageBuffer::from_pixel(full_width, full_height, Rgba([0, 0, 0, 255]));

    for y in 19..=32 {
        for x in 17..=32 {
            let key = format!("map{:02}{:02}", x, y);
            println!("🧩 Verarbeite Tile: {}", key);
            let tile_parts = map_tiles.get(&key);
            let merged_tile = match tile_parts {
                Some(parts) => merge_tile_parts(parts, &tile_size)?,
                None => {
                    ImageBuffer::from_pixel(big_tile_size.0, big_tile_size.1, Rgba([0, 0, 0, 255]))
                }
            };

            let px = (x - 17) * big_tile_size.0;
            let py = (y - 19) * big_tile_size.1;
            worldmap
                .copy_from(&merged_tile, px as u32, py as u32)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        }
    }

    let ext = map_tiles
        .values()
        .next()
        .and_then(|map| map.values().next())
        .and_then(|p| p.extension().and_then(|e| e.to_str()))
        .unwrap_or("png");

    let output_path = minimap_dir.join(format!("worldmap.{}", ext));
    worldmap
        .save(&output_path)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    println!("✅ Weltkarte gespeichert: {}", output_path.display());
    Ok(())
}

fn find_minimap_folder(root: &Path) -> std::io::Result<PathBuf> {
    for entry in WalkDir::new(root).min_depth(1).into_iter().flatten() {
        if entry.file_type().is_dir()
            && entry.file_name().to_string_lossy().to_lowercase() == "minimap"
        {
            return Ok(entry.into_path());
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "minimap-Ordner nicht gefunden",
    ))
}

fn collect_map_tiles(dir: &Path) -> std::io::Result<HashMap<String, HashMap<char, PathBuf>>> {
    let mut map_tiles: HashMap<String, HashMap<char, PathBuf>> = HashMap::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let filename = path.file_stem().and_then(|f| f.to_str()).unwrap_or("");
        let chars: Vec<char> = filename.chars().collect();
        if filename.starts_with("map") && chars.len() >= 8 {
            let tile_key = &filename[..7];
            let part = chars[7];
            if ['a', 'b', 'c', 'd'].contains(&part) {
                map_tiles
                    .entry(tile_key.to_string())
                    .or_default()
                    .insert(part, path);
            }
        }
    }
    Ok(map_tiles)
}

fn find_tile_dimensions(
    map_tiles: &HashMap<String, HashMap<char, PathBuf>>,
) -> std::io::Result<(u32, u32)> {
    map_tiles
        .values()
        .flat_map(|parts| parts.values())
        .find_map(|path| {
            image::open(path)
                .map(|img| img.dimensions())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
                .ok()
        })
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Keine gueltigen Bilddateien gefunden",
            )
        })
}

fn merge_tile_parts(
    parts: &HashMap<char, PathBuf>,
    tile_size: &(u32, u32),
) -> std::io::Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let (w, h) = *tile_size;
    let mut big_tile = ImageBuffer::from_pixel(w * 2, h * 2, Rgba([0, 0, 0, 255]));
    let positions = [('a', 0, 0), ('b', w, 0), ('c', 0, h), ('d', w, h)];

    for (part, x, y) in positions {
        if let Some(path) = parts.get(&part) {
            let img = image::open(path)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
                .into_rgba8();
            big_tile
                .copy_from(&img, x, y)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        }
    }
    Ok(big_tile)
}
