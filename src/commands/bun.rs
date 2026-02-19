// src/commands/bun.rs
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::fs;
use std::env;
use std::io::Cursor;
use zip::ZipArchive;
use std::os::windows::fs as windows_fs;

pub async fn list_remote() -> Result<()> {
    println!("Fetching Bun versions...");
    let client = reqwest::Client::builder().user_agent("mlvm-rust").build()?;
    // Bun tags are simple
    let url = "https://api.github.com/repos/oven-sh/bun/tags";
    let response = client.get(url).send().await?.error_for_status()?;
    let tags: Vec<serde_json::Value> = response.json().await?;

    for tag in tags.iter().take(15) {
        if let Some(name) = tag.get("name").and_then(|v| v.as_str()) {
            println!("- {}", name);
        }
    }
    Ok(())
}

pub async fn install(version: &str) -> Result<()> {
    let version = if version.starts_with('v') { version.to_string() } else { format!("v{}", version) };
    
    println!("Installing Bun {}...", version);

    // Determine URL based on OS
    let target = match (env::consts::OS, env::consts::ARCH) {
        ("windows", "x86_64") => "bun-windows-x64",
        ("linux", "x86_64") => "bun-linux-x64",
        ("linux", "aarch64") => "bun-linux-aarch64",
        ("macos", "x86_64") => "bun-darwin-x64",
        ("macos", "aarch64") => "bun-darwin-aarch64",
        _ => bail!("Unsupported platform for Bun"),
    };

    let download_url = format!("https://github.com/oven-sh/bun/releases/download/{}/{}.zip", version, target);
    println!("Downloading {}...", download_url);

    let response = reqwest::get(&download_url).await?.error_for_status()?;
    let bytes = response.bytes().await?;

    let mlvm_dir = dirs::home_dir().context("No home")?.join(".mlvm");
    let lang_dir = mlvm_dir.join("bun");
    let install_path = lang_dir.join(&version);

    if install_path.exists() {
        println!("Bun {} already installed", version);
        return Ok(());
    }

    println!("Unpacking...");
    let temp = lang_dir.join("temp");
    if temp.exists() { fs::remove_dir_all(&temp)?; }

    let mut archive = ZipArchive::new(Cursor::new(bytes))?;
    archive.extract(&temp)?;

    // Bun zips usually extract to a folder named like "bun-windows-x64"
    let source = temp.join(target);
    fs::create_dir_all(&lang_dir)?;
    fs::rename(source, &install_path)?;
    fs::remove_dir_all(temp)?;

    println!("Installed Bun {}", version);
    Ok(())
}

pub fn use_version(version: &str) -> Result<()> {
    let version = if version.starts_with('v') { version.to_string() } else { format!("v{}", version) };
    let lang_dir = dirs::home_dir().context("No home")?.join(".mlvm").join("bun");
    let version_path = lang_dir.join(&version);

    if !version_path.exists() {
        bail!("Bun {} not installed", version);
    }
    
    let current = lang_dir.join("current");
    if current.exists() || current.symlink_metadata().is_ok() {
        if current.is_dir() { fs::remove_dir_all(&current).ok(); } else { fs::remove_file(&current).ok(); }
    }

    #[cfg(unix)]
    std::os::unix::fs::symlink(&version_path, &current)?;
    #[cfg(windows)]
    windows_fs::symlink_dir(&version_path, &current)?;

    println!("Switched to Bun {}", version);
    println!("Add this to PATH: {}", current.display());

    Ok(())
}