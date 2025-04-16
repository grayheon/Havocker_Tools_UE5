use crate::{DecryptKey, FileExtension, SkipMode, decrypt_data_pure};
use regex::Regex;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn find_files(source_path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let ignored_folders = ["low", "medium"];

    if let Ok(entries) = fs::read_dir(source_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let folder_name = path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                if ignored_folders
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(&folder_name))
                {
                    continue;
                }

                files.extend(find_files(&path));
            } else if let Some(ext_str) = path.extension().and_then(|e| e.to_str()) {
                let ext = FileExtension::from_str(ext_str);
                if ext.is_relevant() {
                    let file_name = path
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    if matches!(
                        ext,
                        FileExtension::Dat | FileExtension::Ma1 | FileExtension::Ma2
                    ) {
                        if file_name == "data.dat"
                            || file_name == "reference.dat"
                            || file_name.ends_with(".ma1")
                            || file_name.ends_with(".ma2")
                        {
                            files.push(path);
                        }
                    } else {
                        files.push(path);
                    }
                }
            }
        }
    }

    files
}

pub fn should_skip_decryption(file_name: &str) -> SkipMode {
    let name = file_name.to_lowercase();

    if name == "archlordgb.ini" {
        SkipMode::Ignore
    } else if name == "ggpoint.ini"
        || name == "coption.ini"
        || name == "autopickup.xml"
        || name == "loginsettings.txt"
    {
        SkipMode::CopyOnly
    } else {
        let pattern = Regex::new(r"^obj\d{5}\.ini$|^obs\d{4}\.ini$").unwrap();
        if pattern.is_match(&name) {
            SkipMode::CopyOnly
        } else {
            SkipMode::Decrypt
        }
    }
}

pub fn process_regular_files(files: &[PathBuf], source_path: &str, destination_path: &str) {
    let non_dat_files: Vec<PathBuf> = files
        .iter()
        .filter(|f| {
            let ext = f.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            !matches!(ext.as_str(), "dat" | "ma1" | "ma2")
        })
        .cloned()
        .collect();

    for file in &non_dat_files {
        let relative_path = file.strip_prefix(source_path).unwrap_or(file);
        let file_name = file
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("")
            .to_lowercase();
        let extension = file
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let ext_enum = FileExtension::from_str(&extension);
        let new_ext = ext_enum.mapped();

        let mut new_relative_path = relative_path.to_path_buf();
        if new_ext != extension {
            new_relative_path.set_extension(new_ext);
        }

        let dest_file = Path::new(destination_path).join(&new_relative_path);

        match should_skip_decryption(&file_name) {
            SkipMode::Ignore => continue,
            SkipMode::CopyOnly => copy_file(file, &dest_file),
            SkipMode::Decrypt => {
                if ext_enum.should_decrypt() {
                    decrypt_and_write(file, &dest_file);
                } else {
                    copy_file(file, &dest_file);
                }
            }
        }
    }
}

fn copy_file(src: &Path, dest: &Path) {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).ok();
    }
    if let Ok(data) = fs::read(src) {
        fs::write(dest, &data).ok();
    }
}

fn decrypt_and_write(src: &Path, dest: &Path) {
    if let Ok(mut data) = fs::read(src) {
        let _ = decrypt_data_pure(&mut data, DecryptKey::Default);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(dest, &data).ok();
    }
}
