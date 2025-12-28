use crate::{DecryptKey, FileExtension, decrypt_data_pure};
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
};

/// Extrahiert Dateien aus echten DAT-Dateien (data.dat + reference.dat) ins Zielverzeichnis
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
pub fn extract_from_dat(
    data_path: &Path,
    ref_path: &Path,
    destination_path: &Path,
) -> io::Result<()> {
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

    println!(
        "📂 Extrahiere {} (DAT-Datei) → Ziel: {}",
        data_path.display(),
        output_base.display()
    );
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

        let ext_enum = FileExtension::from_str(&extension);
        let mapped_ext = ext_enum.mapped();
        let final_name = Path::new(&original_file_name).with_extension(mapped_ext);
        let output_file_path = output_base.join(final_name);

        data_file.seek(SeekFrom::Start(offset))?;
        let mut buffer = vec![0u8; size];
        data_file.read_exact(&mut buffer)?;

        let decrypted_slice = match extension.as_str() {
            "ini" => {
                let len = decrypt_data_pure(&mut buffer, DecryptKey::Default);
                &buffer[..len]
            }
            "tx1" => {
                let len = decrypt_data_pure(&mut buffer, DecryptKey::Texture);
                &buffer[..len]
            }
            _ => &buffer[..],
        };

        let mut out_file = File::create(&output_file_path)?;
        out_file.write_all(decrypted_slice)?;

        if i % 50 == 0 || i + 1 == files_count {
            print!(
                "\r⏳ Extrahiere {} [{}/{}]",
                data_path.display(),
                i + 1,
                files_count
            );
            io::stdout().flush().ok();
        }
    }

    println!("\n📦 Extraktion abgeschlossen → {}", output_base.display());
    Ok(())
}
