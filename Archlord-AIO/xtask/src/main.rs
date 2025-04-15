use std::process::{Command, exit};
use std::fs;
use std::path::Path;

fn main() {
    let version = read_version("apps/meta/Cargo.toml");
    let tag = format!("Archlord-AIO-v{}", version);

    println!("🔨 Building release...");
    run("cargo", &["build", "--manifest-path", "Cargo.toml", "--workspace", "--release"]);

    let release_dir = format!("Archlord-AIO-v{}", version);
    fs::create_dir_all(&release_dir).unwrap();

    println!("📁 Collecting binaries...");
    let bin_path = Path::new("target/release");
    for entry in fs::read_dir(bin_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy().to_lowercase();

        if path.is_file()
            && path.extension().map_or(false, |e| e == "exe")
            && file_name != "xtask.exe"
        {
            fs::copy(&path, Path::new(&release_dir).join(path.file_name().unwrap())).unwrap();
        }
    }


    println!("🏷️ Checking if tag already exists...");
    let tag_check = Command::new("git")
        .args(["tag", "-l", &tag])
        .output()
        .expect("Failed to check tag");

    let tag_exists = String::from_utf8_lossy(&tag_check.stdout).trim() == tag;

    if tag_exists {
        println!("ℹ️ Tag '{}' already exists. Skipping tag creation.", tag);
    } else {
        println!("🏷️ Creating tag: {}", tag);
        run("git", &["tag", &tag]);
        run("git", &["push", "origin", &tag]);
    }


    println!("⬆️ Uploading GitHub release...");
    let paths: Vec<_> = fs::read_dir(&release_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();

    let title = format!("Archlord-AIO {}", version);
    let notes = format!("Automated Windows release for version {}", version);

    let mut args = vec![
        "release", "create", &tag,
        "--title", &title,
        "--notes", &notes,
    ];

    for path in &paths {
        args.push(path.to_str().unwrap());
    }

    run("gh", &args);

}

fn read_version(toml_path: &str) -> String {
    let content = fs::read_to_string(toml_path).expect("Couldn't read Cargo.toml");
    for line in content.lines() {
        if line.trim_start().starts_with("version") {
            return line.split('"').nth(1).unwrap().to_string();
        }
    }
    panic!("Version not found");
}

fn run(cmd: &str, args: &[&str]) {
    let status = Command::new(cmd)
        .args(args)
        .status()
        .expect("Failed to run command");
    if !status.success() {
        eprintln!("❌ Command failed: {} {:?}", cmd, args);
        exit(1);
    }
}
