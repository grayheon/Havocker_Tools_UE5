use std::process::{Command, exit};
use std::fs;
use std::path::Path;

fn main() {
    let version = read_version("apps/meta/Cargo.toml");
    let tag = format!("Archlord-AIO-v{}", version);

    println!("🔨 Building release...");
    run("cargo", &["build", "--manifest-path", "Cargo.toml", "--workspace", "--release"]);

    let release_dir = format!("release_out_{}", version);
    fs::create_dir_all(&release_dir).unwrap();

    println!("📁 Collecting binaries...");
    let bin_path = Path::new("target/release");
    for entry in fs::read_dir(bin_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |e| e == "exe") {
            fs::copy(&path, Path::new(&release_dir).join(path.file_name().unwrap())).unwrap();
        }
    }

    println!("🏷️ Creating tag: {}", tag);
    run("git", &["tag", &tag]);
    run("git", &["push", "origin", &tag]);

    println!("⬆️ Uploading GitHub release...");
    run("gh", &[
        "release", "create", &tag,
        "--title", &format!("Archlord-AIO {}", version),
        "--notes", &format!("Automated Windows release for version {}", version),
        &format!("{}/{}", release_dir, "*")
    ]);
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
