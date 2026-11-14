use std::path::{Path, PathBuf};
use std::{fs, env};
use std::process::{self, Command, Stdio};
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

const JAVA_MIN: u32 = 17;
const OS: &str = if cfg!(windows) { "windows" } else if cfg!(target_os = "macos") { "mac" } else { "linux" };
const ARCH: &str = if cfg!(target_arch = "x86_64") || cfg!(windows) { "x64" } else { "aarch64" }; // No temurin 17 win arm64 :(
const JAVA_BIN: &str = if cfg!(windows) { "java.exe" } else { "java" };

fn main() {
    println!("Hello, world!");
}

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Result<Self, String> {
        let veryveryrandomgeneratedhash = RandomState::new().build_hasher().finish();
        let temp = env::temp_dir().join(format!("javlet-{}-{veryveryrandomgeneratedhash:016x}", process::id()));

        fs::create_dir_all(&temp).map_err(|e| format!("Cannot create {}: {e}", temp.display()))?;

        Ok(Self(temp))
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
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

fn java_ok(java: &Path) -> bool {
    let Ok(out) = Command::new(java).arg("-version").stdin(Stdio::null()).output() else {
        return false;
    };

    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));

    // got && major version number is actually the minimum or higher
    out.status.success() && text.split_once("version \"").and_then(|(_, rest)| rest.split(|c: char| !c.is_ascii_digit()).next()?.parse().ok()).unwrap_or(0) >= JAVA_MIN
}

fn find_java() -> Option<PathBuf> {
    if let Some(home) = env::var_os("JAVA_HOME") {
        let guess = Path::new(&home).join("bin").join(JAVA_BIN);

        if guess.is_file() && java_ok(&guess) {
            return Some(guess);
        }
    }

    let guess = PathBuf::from("java");

    java_ok(&guess).then_some(guess)
}