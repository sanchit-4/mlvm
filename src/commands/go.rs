use anyhow::Ok;
// src/commands/go.rs
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::fs;
use std::env;
use flate2::read::GzDecoder;
use tar::Archive;
use zip::ZipArchive;
use std::io::Cursor;
use std::os::windows::fs as windows_fs;

#[derive(Deserialize, Debug)]
struct GoVersion {
    version: String,
    // The Go API doesn't guarantee a 'stable' boolean field, so we rely on the list itself.
}

pub async fn list_remote() -> Result<()> {
    println!("Fetching available Go versions...");
    
    // API LIMITATION: By default go.dev only returns the top 2 versions.
    // We add `&include=all` to get the history.
    let url = "https://go.dev/dl/?mode=json&include=all";
    
    let response = reqwest::get(url).await?;
    let versions: Vec<GoVersion> = response.json().await?;

    println!("Available Go versions (Top 20 shown):");
    // Limit to top 20 to avoid flooding the terminal
    for v in versions.iter().take(20) {
        println!("- {}", v.version.trim_start_matches("go"));
    }

    Ok(())
}

pub async fn install(version: &str) -> Result<()> {
    let raw_version = version.trim_start_matches("go");
    let go_version_str = format!("go{}", raw_version); 

    let (os, arch, ext) = match (env::consts::OS, env::consts::ARCH) {
        ("windows", "x86_64") => ("windows", "amd64", "zip"),
        ("windows", "x86") => ("windows", "386", "zip"),
        ("linux", "x86_64") => ("linux", "amd64", "tar.gz"),
        ("linux", "aarch64") => ("linux", "arm64", "tar.gz"),
        ("macos", "x86_64") => ("darwin", "amd64", "tar.gz"),
        ("macos", "aarch64") => ("darwin", "arm64", "tar.gz"),
        (o, a) => bail!("Unsupported platform: {} {}", o, a),
    };

    let download_url = format!("https://go.dev/dl/{}.{}-{}.{}", go_version_str, os, arch, ext);
    println!("Downloading {}...", download_url);

    let response = reqwest::get(&download_url).await?
        .error_for_status()
        .context("Failed to download. Check version number.")?;
    let bytes = response.bytes().await?;

    let mlvm_dir = dirs::home_dir().context("No home")?.join(".mlvm");
    let lang_dir = mlvm_dir.join("go");
    let install_path = lang_dir.join(raw_version);

    if install_path.exists() {
        println!("Go {} is already installed.", raw_version);
        return Ok(());
    }

    println!("Unpacking...");
    let temp_unpack = lang_dir.join("temp_unpack");
    if temp_unpack.exists() { fs::remove_dir_all(&temp_unpack)?; }

    if ext == "zip" {
        let mut archive = ZipArchive::new(Cursor::new(bytes))?;
        archive.extract(&temp_unpack)?;
    } else {
        let tar = GzDecoder::new(&bytes[..]);
        let mut archive = Archive::new(tar);
        archive.unpack(&temp_unpack)?;
    }

    // Go archives extract into a "go" folder
    let source = temp_unpack.join("go");
    fs::create_dir_all(&lang_dir)?;
    fs::rename(source, &install_path)?;
    fs::remove_dir_all(temp_unpack)?;

    println!("Installed Go {}", raw_version);
    Ok(())
}

pub fn use_version(version: &str) -> Result<()> {
    let version = version.trim_start_matches("go");
    let lang_dir = dirs::home_dir().context("No home")?.join(".mlvm").join("go");
    let version_path = lang_dir.join(version);

    if !version_path.exists() {
        bail!("Go {} not installed.", version);
    }

    let current = lang_dir.join("current");
    if current.exists() || current.symlink_metadata().is_ok() {
        if current.is_dir() { fs::remove_dir_all(&current).ok(); } 
        else { fs::remove_file(&current).ok(); }
    }

    #[cfg(unix)]
    std::os::unix::fs::symlink(&version_path, &current)?;
    #[cfg(windows)]
    windows_fs::symlink_dir(&version_path, &current)?;

    println!("Successfully switched 'current' symlink to Go {}.", version);
    
    let bin_path = current.join("bin");
    
    println!("\nACTION REQUIRED:");
    println!("1. Ensure this path is in your System PATH environment variable:");
    println!("   {}", bin_path.display());
    
    println!("2. RESTART YOUR TERMINAL.");
    println!("   Windows cannot update the PATH of an already running terminal.");
    println!("   After restarting, run `go version` to confirm.");

    Ok(())
}