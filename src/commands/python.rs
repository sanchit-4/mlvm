// src/commands/python.rs
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::env;
use std::fs;
use std::io::Cursor;
use tar::Archive;
use flate2::read::GzDecoder; // Ensure you have flate2 in Cargo.toml

#[derive(Deserialize, Debug)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize, Debug)]
struct GitHubRelease {
    assets: Vec<GitHubAsset>,
}

fn get_target_platform() -> Result<String> {
    let arch = match env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        other => bail!("Unsupported architecture: {}", other),
    };
    let os = match env::consts::OS {
        "linux" => "unknown-linux-gnu",
        "macos" => "apple-darwin",
        // The repo uses 'pc-windows-msvc' usually
        "windows" => "pc-windows-msvc", 
        other => bail!("Unsupported OS: {}", other),
    };
    Ok(format!("{}-{}", arch, os))
}

pub async fn list_remote() -> Result<()> {
    println!("Fetching available Python versions...");
    
    let client = reqwest::Client::builder().user_agent("mlvm-rust").build()?;
    let url = "https://api.github.com/repos/indygreg/python-build-standalone/releases/latest";
    let response = client.get(url).send().await?.error_for_status()?;
    let release: GitHubRelease = response.json().await?;

    let platform = get_target_platform()?;
    println!("Searching for builds compatible with: {}", platform);

    let mut versions: Vec<String> = release.assets.iter()
        .filter_map(|asset| {
            // RELAXED FILTER: 
            // 1. Must contain "cpython-"
            // 2. Must contain platform string (e.g., x86_64-pc-windows-msvc)
            // 3. Must be an archive we can handle (.tar.zst or .tar.gz)
            // 4. Must be 'install_only' (smaller, no build tools)
            if asset.name.starts_with("cpython-") 
               && asset.name.contains(&platform)
               && asset.name.contains("install_only")
               && (asset.name.ends_with(".tar.zst") || asset.name.ends_with(".tar.gz"))
            {
                let parts: Vec<&str> = asset.name.split('-').collect();
                // Format usually: cpython-<VER>+<DATE>-<ARCH>-...
                if parts.len() > 1 {
                    let ver_parts: Vec<&str> = parts[1].split('+').collect();
                    return Some(ver_parts[0].to_string());
                }
            }
            None
        })
        .collect();
    
    versions.sort();
    versions.dedup();
    versions.reverse(); 

    if versions.is_empty() {
        println!("No versions found. Please check if your platform is supported by python-build-standalone.");
    } else {
        println!("Available versions (Top 20):");
        for v in versions.iter().take(20) {
            println!("- {}", v);
        }
    }
    Ok(())
}

pub async fn install(version: &str) -> Result<()> {
    println!("Fetching manifest to find Python {}...", version);
    
    let client = reqwest::Client::builder().user_agent("mlvm-rust").build()?;
    let url = "https://api.github.com/repos/indygreg/python-build-standalone/releases/latest";
    let response = client.get(url).send().await?.error_for_status()?;
    let release: GitHubRelease = response.json().await?;

    let target_platform = get_target_platform()?;
    let search_prefix = format!("cpython-{}", version);
    
    let asset = release.assets.iter().find(|a| {
        a.name.starts_with(&search_prefix) 
        && a.name.contains(&target_platform) 
        && a.name.contains("install_only")
        && (a.name.ends_with(".tar.zst") || a.name.ends_with(".tar.gz"))
    });

    let asset = match asset {
        Some(a) => a,
        None => bail!("Could not find Python {} for {}.", version, target_platform),
    };

    println!("Downloading {}...", asset.name);
    let response = reqwest::get(&asset.browser_download_url).await?.error_for_status()?;
    let bytes = response.bytes().await?;

    let mlvm_dir = dirs::home_dir().context("No home dir")?.join(".mlvm");
    let lang_dir = mlvm_dir.join("python");
    let install_path = lang_dir.join(version);
    
    if install_path.exists() {
        println!("Version {} is already installed.", version);
        return Ok(());
    }

    let temp_unpack_path = lang_dir.join("temp_unpack");
    if temp_unpack_path.exists() { fs::remove_dir_all(&temp_unpack_path)?; }
    
    // Handle Decompression based on extension
    if asset.name.ends_with(".tar.zst") {
        println!("Unpacking .tar.zst...");
        let decoder = zstd::stream::decode_all(Cursor::new(bytes))?;
        let mut archive = Archive::new(Cursor::new(decoder));
        archive.unpack(&temp_unpack_path)?;
    } else if asset.name.ends_with(".tar.gz") {
        println!("Unpacking .tar.gz...");
        let decoder = GzDecoder::new(&bytes[..]);
        let mut archive = Archive::new(decoder);
        archive.unpack(&temp_unpack_path)?;
    }

    let source_path = temp_unpack_path.join("python");
    if !source_path.exists() {
        bail!("Extracted archive did not contain 'python' folder.");
    }

    fs::create_dir_all(&lang_dir)?;
    
    // Retry rename logic (Windows file locking can sometimes fail instant renames)
    let mut success = false;
    for _ in 0..3 {
        if fs::rename(&source_path, &install_path).is_ok() {
            success = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    if !success {
        bail!("Failed to move extracted folder to installation path.");
    }

    fs::remove_dir_all(temp_unpack_path)?;

    println!("Successfully installed Python {}", version);
    Ok(())
}

// Make sure this function is PUBLIC
pub fn list_local() -> Result<()> {
    let lang_dir = dirs::home_dir().context("No home")?.join(".mlvm").join("python");
    if !lang_dir.exists() { 
        println!("No Python versions installed.");
        return Ok(()); 
    }

    println!("Installed Python versions:");
    for entry in fs::read_dir(lang_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if entry.file_type()?.is_dir() && name != "current" && name != "temp_unpack" {
            println!("- {}", name);
        }
    }
    Ok(())
}

// Ensure use_version is here as defined in previous steps
pub fn use_version(version: &str) -> Result<()> {
    let lang_dir = dirs::home_dir().context("No home")?.join(".mlvm").join("python");
    let version_path = lang_dir.join(version);
    
    if !version_path.exists() {
        bail!("Python {} not installed.", version);
    }

    let current_link = lang_dir.join("current");
    if current_link.exists() || current_link.symlink_metadata().is_ok() {
        if current_link.is_dir() { fs::remove_dir_all(&current_link).ok(); } 
        else { fs::remove_file(&current_link).ok(); }
    }

    #[cfg(unix)]
    std::os::unix::fs::symlink(&version_path, &current_link)?;
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(&version_path, &current_link)?;

    println!("Switched to Python {}", version);
    println!("Ensure {} is in your PATH.", current_link.display());
    Ok(())
}