use crate::{decrypt_data_pure, DecryptKey, FileExtension};
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

/// Extrahiert Dateien aus `data.dat` und `reference.dat` ins Zielverzeichnis
pub fn extract_files(data_path: &Path, ref_path: &Path, destination_path: &Path) -> io::Result<()> {
    let data_encrypted = fs::read(data_path)?;
    let mut data_file = io::Cursor::new(&data_encrypted);

    let mut ref_encrypted = fs::read(ref_path)?;
    let decrypted_ref_len = decrypt_data_pure(&mut ref_encrypted, DecryptKey::Default);
    let mut ref_file = io::Cursor::new(&ref_encrypted[..decrypted_ref_len]);



    let files_count = read_u32(&mut ref_file)?;
    let folder_size = read_u32(&mut ref_file)?;
    let folder_name = read_string(&mut ref_file, folder_size as usize)?;

    let output_base = destination_path.join(folder_name);
    fs::create_dir_all(&output_base)?;

    let file_label = match data_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase().as_str() {
        "dat" => "DAT-Datei",
        "ma1" => "MA1 (wie reference.dat)",
        "ma2" => "MA2 (wie data.dat)",
        _ => "Unbekannte Datei",
    };

    println!("📂 Extrahiere {} ({}) → Ziel: {}", data_path.display(), file_label, output_base.display());
    println!("Entschlüsselte Referenzdatei Größe: {}", decrypted_ref_len);

    for i in 0..files_count {
        let name_size = read_u32(&mut ref_file)? as usize;
        let original_file_name = read_string(&mut ref_file, name_size)?;
        let offset = read_u32(&mut ref_file)? as u64;
        let size = read_u32(&mut ref_file)? as usize;

        let extension = Path::new(&original_file_name)
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or("")
            .to_lowercase();

        let ext_map = FileExtension::from_str(&extension);
        let new_extension = ext_map.mapped();

        let mut new_file_name = PathBuf::from(&original_file_name);
        new_file_name.set_extension(new_extension);
        let output_file_path = output_base.join(&new_file_name);

        data_file.seek(SeekFrom::Start(offset))?;
        let mut buffer = vec![0; size];
        data_file.read_exact(&mut buffer)?;

        let decrypted_data = match extension.as_str() {
            "ini" => {
                let mut data = buffer.clone();
                // decrypt_data(&mut data, DecryptKey::Default).map(|len| data[..len].to_vec()).unwrap_or(buffer)
                let len = decrypt_data_pure(&mut data, DecryptKey::Default);
                data[..len].to_vec()

            }
            "tx1" => {
                let mut data = buffer.clone();
                // decrypt_data(&mut data, DecryptKey::Texture).map(|len| data[..len].to_vec()).unwrap_or(buffer)
                let len = decrypt_data_pure(&mut data, DecryptKey::Texture);
                data[..len].to_vec()

            }
            _ => buffer,
        };

        let mut out_file = File::create(&output_file_path)?;
        out_file.write_all(&decrypted_data)?;
        print!("\r⏳ Extrahiere {} [{}/{}]", data_path.display(), i + 1, files_count);
        io::stdout().flush().ok();
    }

    println!("📦 Extraktion abgeschlossen → {}", output_base.display());
    Ok(())
}

fn read_u32<R: Read>(file: &mut R) -> io::Result<u32> {
    let mut buffer = [0; 4];
    file.read_exact(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

fn read_string<R: Read>(file: &mut R, size: usize) -> io::Result<String> {
    let mut buffer = vec![0; size];
    file.read_exact(&mut buffer)?;
    Ok(String::from_utf8_lossy(&buffer)
        .trim_matches(char::from(0))
        .to_string())
}
