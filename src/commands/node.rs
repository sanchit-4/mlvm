use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::fs;
use flate2::read::GzDecoder; // For Gzip decompression
use tar::Archive;           // For Tar archive handling
use std::env; // <-- Import the `env` module to get OS info
use zip::ZipArchive;
use std::os::windows::fs as windows_fs;
use std::io;

// A struct that represents the fields we care about in the JSON response.
// `serde` will automatically map the JSON keys to these struct fields.
#[derive(Deserialize, Debug)]
struct NodeVersion {
    version: String,
    lts: serde_json::Value, // Can be a string or `false`, so we use a generic Value
}

// This is our main async function for this command.
pub async fn list_remote() -> Result<()> {
    println!("Fetching available Node.js versions...");

    // 1. Define the URL of the official Node.js versions JSON index.
    const NODE_DIST_URL: &str = "https://nodejs.org/dist/index.json";

    // 2. Make the HTTP GET request.
    // The `?` operator will propagate any network errors.
    let response = reqwest::get(NODE_DIST_URL).await?;

    // 3. Parse the JSON response into a vector of our `NodeVersion` structs.
    // The `json()` method is a convenience from `reqwest` (enabled by our "json" feature flag)
    // that handles deserialization for us.
    let versions: Vec<NodeVersion> = response.json().await?;

    println!("Available versions:");
    // 4. Iterate and print the versions.
    for version in versions {
        // Check if the `lts` field is a string to label it nicely.
        if version.lts.is_string() {
            println!("- {} (LTS: {})", version.version, version.lts.as_str().unwrap());
        } else {
            println!("- {}", version.version);
        }
    }

    Ok(())
}

pub async fn install(version: &str) -> Result<()> {
    // 1. Sanitize the version string.
    let version = if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{}", version)
    };
    println!("Installing Node.js version {}...", version);

    // 2. Determine the correct download URL based on OS and architecture.
    // std::env::consts::OS will be "linux", "windows", "macos", etc.
    let target_os = match env::consts::OS {
        "windows" => "win",
        "macos" => "darwin",
        "linux" => "linux",
        other => bail!("Unsupported operating system: {}", other),
    };

    // std::env::consts::ARCH will be "x86_64", "aarch64", etc.
    let target_arch = match env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64", // For Apple Silicon Macs
        other => bail!("Unsupported architecture: {}", other),
    };
    
    // The file extension is different for Windows.
    let extension = if target_os == "win" { "zip" } else { "tar.gz" };

    let filename = format!("node-{}-{}-{}.{}", version, target_os, target_arch, extension);
    let download_url = format!("https://nodejs.org/dist/{}/{}", version, filename);

    println!("Downloading from {}...", download_url);

    // 3. Download the file.
    let response = reqwest::get(&download_url).await?
        .error_for_status()
        .with_context(|| format!("Failed to download Node.js version {}. It might not exist for your platform.", version))?;

    // We need the raw bytes for both zip and tar.
    let file_bytes = response.bytes().await?;

    // 4. Find the installation directory.
    let mlvm_dir = dirs::home_dir()
        .context("Could not find home directory")?
        .join(".mlvm");
    let lang_dir = mlvm_dir.join("node");
    fs::create_dir_all(&lang_dir)?;

    let install_path = lang_dir.join(&version);
    if install_path.exists() {
        println!("Version {} is already installed.", version);
        return Ok(());
    }
    println!("Unpacking to {:?}...", install_path);

    // 5. Decompress and unpack based on the file extension.
    let temp_unpack_path = lang_dir.join("temp_unpack");
    if temp_unpack_path.exists() {
        fs::remove_dir_all(&temp_unpack_path)?;
    }

    if extension == "zip" {
        // Use the `zip` crate to handle .zip files
        let mut archive = ZipArchive::new(std::io::Cursor::new(file_bytes))?;
        archive.extract(&temp_unpack_path)?;
    } else {
        // Use the `tar` and `flate2` crates for .tar.gz
        let tar = GzDecoder::new(&file_bytes[..]);
        let mut archive = Archive::new(tar);
        archive.unpack(&temp_unpack_path)?;
    }
    
    // The unpacked folder name is also different for Windows.
    let unpacked_folder_name = format!("node-{}-{}-{}", version, target_os, target_arch);
    let source_path = temp_unpack_path.join(unpacked_folder_name);

    fs::rename(source_path, &install_path)?;
    fs::remove_dir_all(temp_unpack_path)?;

    println!("Successfully installed Node.js {}", version);

    Ok(())
}

pub fn use_version(version: &str) -> Result<()> {
    // ... steps 1-4 (sanitizing version, finding paths) are the same ...
    let version = if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{}", version)
    };
    println!("Switching to Node.js version {}...", version);

    let lang_dir = dirs::home_dir()
        .context("Could not find home directory")?
        .join(".mlvm")
        .join("node");

    let version_path = lang_dir.join(&version);
    if !version_path.exists() {
        bail!("Version {} is not installed. Please run `mlvm node install {}` first.", version, version.trim_start_matches('v'));
    }

    let current_symlink_path = lang_dir.join("current");

    if current_symlink_path.exists() || current_symlink_path.symlink_metadata().is_ok() {
         // On Windows, remove_file might fail for a directory symlink.
         // A more robust approach would be to use remove_dir.
        if current_symlink_path.is_dir() {
            fs::remove_dir(&current_symlink_path).ok();
        } else {
            fs::remove_file(&current_symlink_path).ok();
        }
    }

    // Create the new symlink.
    let result = {
        #[cfg(windows)]
        {
            windows_fs::symlink_dir(&version_path, &current_symlink_path)
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&version_path, &current_symlink_path)
        }
    };

    // --- THIS IS THE NEW, INTELLIGENT ERROR HANDLING ---
    if let Err(e) = result {
        #[cfg(windows)]
        {
            // `ErrorKind::PermissionDenied` is often what os error 1314 maps to.
            if e.kind() == io::ErrorKind::PermissionDenied {
                bail!(
                    "Failed to create symlink. This is a permissions issue.\n\n\
                    On Windows, you must either:\n\
                    1. (Recommended) Enable 'Developer Mode' in your Windows settings.\n\
                       Go to Settings > Update & Security > For Developers > and turn on 'Developer Mode'.\n\
                    2. Run this command in a terminal that is 'Run as Administrator'.\n\n\
                    Original error: {}",
                    e
                );
            }
        }
        // For any other error, or on other platforms, just return it directly.
        return Err(e).context("Failed to create symlink");
    }
    #[cfg(unix)]
    let bin_path = current_symlink_path.join("bin");
    #[cfg(windows)]
    let bin_path = current_symlink_path;

    println!("  Add this to your PATH: {}", bin_path.display());
    
    println!("Successfully switched to Node.js {}", version);
    println!("\nTo finish setup, you must add the 'current' directory to your PATH.");
    println!("You can do this in PowerShell with:");
    let current_bin_path = lang_dir.join("current");
    println!("  $env:PATH = \"{};\" + $env:PATH", current_bin_path.to_string_lossy());
    println!("\nTo make this permanent, add that line to your PowerShell profile.");
    println!("Then, restart your terminal.");


    Ok(())
}

pub fn list_local() -> Result<()> {
    let lang_dir = dirs::home_dir()
        .context("Could not find home directory")?
        .join(".mlvm")
        .join("node");

    if !lang_dir.exists() {
        println!("No Node.js versions installed yet.");
        return Ok(());
    }

    println!("Installed Node.js versions:");
    for entry in fs::read_dir(lang_dir)? {
        let entry = entry?;
        let file_name = entry.file_name().into_string().unwrap_or_default();
        // We only list directories that start with 'v' to avoid listing 'current'.
        if entry.file_type()?.is_dir() && file_name.starts_with('v') {
            println!("- {}", file_name);
        }
    }

    Ok(())
}