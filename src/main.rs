use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;

const JAVA_MIN: u32 = 17;
const OS: &str = if cfg!(windows) { "windows" } else if cfg!(target_os = "macos") { "mac" } else { "linux" };
const ARCH: &str = if cfg!(target_arch = "x86_64") || cfg!(windows) { "x64" } else { "aarch64" }; // No temurin 17 win arm64 :(
const JAVA_BIN: &str = if cfg!(windows) { "java.exe" } else { "java" };

fn main() {
    println!("Hello, world!");
}

fn download_jre(tmp: &Path) -> Result<PathBuf, String> {
    let archive = tmp.join("jre.archive");
    let dest = tmp.join("jre");

    fs::create_dir_all(&dest).map_err(|e| e.to_string())?;

    let ok = |c: &mut Command| c.status().map(|s| s.success()).unwrap_or(false);

    eprintln!("Java {JAVA_MIN}+ not found. Downloading Temurin...");

    if !ok(Command::new("curl").args(["-fsSL", "--retry", "3", "-o"]).arg(&archive).arg(format!("https://api.adoptium.net/v3/binary/latest/{JAVA_MIN}/ga/{OS}/{ARCH}/jre/hotspot/normal/eclipse"))) {
        return Err("Couldn't download a runtime".into());
    }

    if !ok(Command::new("tar").arg("-xf").arg(&archive).arg("-C").arg(&dest)) {
        return Err("Couldn't extract the downloaded runtime".into());
    }

    fs::read_dir(&dest).ok().into_iter().flatten().flatten().map(|e| e.path())
        .find_map(|path| {
            [
                path.join("bin").join(JAVA_BIN),
                path.join("Contents").join("Home").join("bin").join(JAVA_BIN),
            ]
            .into_iter()
            .find(|path| path.is_file())
        }).ok_or_else(|| "Weird layout.".to_string())
}