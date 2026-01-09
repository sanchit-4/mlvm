// // src/commands/python.rs

// use anyhow::bail;
// use anyhow::{Context, Result};
// use serde::Deserialize;
// use std::env;
// use std::fs;
// use std::io::Cursor;
// use std::time::Duration; // Needed to read from an in-memory buffer
// use tar::Archive;

// #[derive(Deserialize, Debug)]
// struct GitHubTag {
//     name: String, // This will be like "v3.11.4"
// }

// #[derive(Deserialize, Debug)]
// struct GitHubAsset {
//     name: String,
//     browser_download_url: String,
// }

// #[derive(Deserialize, Debug)]
// struct GitHubRelease {
//     tag_name: String, // This will be like "v3.11.4"
//     name: String, 
//     assets: Vec<GitHubAsset>,     // This will be like "Python 3.11.4"
// }

// pub async fn list_remote() -> Result<()> {
//     println!("Fetching available Python versions from python-build-standalone...");

//     // 1. Define the GitHub API endpoint for the tags, which is more lightweight.
//     const GITHUB_API_URL: &str = "https://api.github.com/repos/indygreg/python-build-standalone/tags";

//     // 2. The GitHub API requires a `User-Agent` header.
//     let client = reqwest::Client::builder()
//         .user_agent("mlvm-rust-app") // Or any other identifier
//         .build()?;

//     // 3. Make the HTTP GET request.
//     let response = client.get(GITHUB_API_URL).send().await?
//         .error_for_status() // Check for non-2xx status codes
//         .context("Failed to fetch tags from GitHub API")?;

//     // 4. Parse the JSON response into a vector of our new `GitHubTag` structs.
//     let tags: Vec<GitHubTag> = response.json().await?;

//     println!("Available versions:");
//     // 5. Iterate and print the tag names.
//     for tag in tags {
//         // We only care about the stable releases, which have a simple "vX.Y.Z" tag format.
//         // This filters out tags for release candidates, etc.
//         if tag.name.starts_with("v20") || tag.name.contains('-') {
//             // python-build-standalone has some date-based tags we can ignore
//             continue;
//         }
//         println!("- {}", tag.name);
//     }

//     Ok(())
// }

// pub async fn install(version: &str) -> Result<()> {
//     // 1. Sanitize the version string (e.g., "3.11.5" -> "v3.11.5").
//     let version_tag = if version.starts_with('v') {
//         version.to_string()
//     } else {
//         format!("v{}", version)
//     };
//     println!("Installing Python version {}...", version_tag);

//     // 2. Construct the platform-specific identifier string that python-build-standalone uses.
//     // e.g., "x86_64-unknown-linux-gnu" or "aarch64-apple-darwin"
//     let target_platform = {
//         let arch = match env::consts::ARCH {
//             "x86_64" => "x86_64",
//             "aarch64" => "aarch64",
//             other => bail!("Unsupported architecture: {}", other),
//         };
//         let os = match env::consts::OS {
//             "linux" => "unknown-linux-gnu",
//             "macos" => "apple-darwin",
//             "windows" => "pc-windows-msvc",
//             other => bail!("Unsupported OS: {}", other),
//         };
//         format!("{}-{}", arch, os)
//     };

//     // 3. Find the correct download URL from the GitHub API.
//     println!("Finding download URL for platform: {}", target_platform);
//     let download_url = find_download_url(&version_tag, &target_platform).await?;
//     println!("Downloading from {}...", download_url);

//     // 4. Download the .tar.zst file.
//     let response = reqwest::get(&download_url).await?.error_for_status()?;
//     let compressed_bytes = response.bytes().await?;

//     // 5. Decompress and unpack the archive.
//     // The downloaded bytes are a Zstandard compressed stream.
//     let decoder = zstd::stream::decode_all(Cursor::new(compressed_bytes))?;
//     // The output of the decoder is a .tar archive.
//     let mut archive = Archive::new(&decoder[..]);

//     let mlvm_dir = dirs::home_dir().context("Could not find home directory")?.join(".mlvm");
//     let lang_dir = mlvm_dir.join("python");
//     let install_path = lang_dir.join(version_tag.trim_start_matches('v'));
//     if install_path.exists() {
//         println!("Version {} is already installed.", version);
//         return Ok(());
//     }

//     println!("Unpacking to {:?}...", install_path);
//     // The python-build-standalone archives have a top-level "python" folder.
//     // We'll unpack to a temp location and then rename the "python" folder.
//     let temp_unpack_path = lang_dir.join("temp_unpack");
//     if temp_unpack_path.exists() {
//         fs::remove_dir_all(&temp_unpack_path)?;
//     }
//     archive.unpack(&temp_unpack_path)?;
    
//     fs::rename(temp_unpack_path.join("python"), &install_path)?;
//     fs::remove_dir_all(temp_unpack_path)?;


//     println!("Successfully installed Python {}", version_tag);
//     Ok(())
// }


// // --- Helper function to query the GitHub API ---
// async fn find_download_url(version_tag: &str, target_platform: &str) -> Result<String> {
//     // We fetch info for a single release tag.
//     let url = format!(
//         "https://api.github.com/repos/indygreg/python-build-standalone/releases/tags/{}",
//         version_tag
//     );

//     let client = reqwest::Client::builder().user_agent("mlvm-rust-app").timeout(Duration::from_secs(30)).build()?;
//     let response = client.get(&url).send().await?.error_for_status()
//         .with_context(|| format!("Could not find Python version release: {}. Please check the version number.", version_tag))?;
    
//     let release: GitHubRelease = response.json().await?;

//     // The asset filename looks like:
//     // cpython-3.11.5+20230826-x86_64-unknown-linux-gnu-install_only.tar.zst
//     for asset in release.assets {
//         if asset.name.contains(target_platform) && asset.name.ends_with("install_only.tar.zst") {
//             return Ok(asset.browser_download_url);
//         }
//     }

//     bail!(
//         "Could not find a compatible Python build for your platform ({}) and version ({}).",
//         target_platform,
//         version_tag
//     )
// }


// src/commands/python.rs

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::env;
use std::fs;
use std::io::Cursor;
use tar::Archive;

// --- Structs for deserializing GitHub API responses ---

#[derive(Deserialize, Debug)]
struct GitHubTag {
    name: String,
}

#[derive(Deserialize, Debug)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize, Debug)]
struct GitHubRelease {
    assets: Vec<GitHubAsset>,
}

// --- `list-remote`: Now prints a reliable, static list ---

pub async fn list_remote() -> Result<()> {
    println!("Showing a curated list of available Python versions.");
    println!("Note: Minor patch versions are available but not listed. The `install` command will automatically find the latest build.");

    let versions = vec![
        "3.12.4", "3.11.9", "3.10.14", "3.9.19", "3.8.19",
    ];

    println!("\nAvailable versions:");
    for version in versions {
        println!("- {}", version);
    }

    Ok(())
}

// --- `install` command and its helpers ---
// This logic is robust and remains largely the same.

async fn fetch_all_tags() -> Result<Vec<String>> {
    const GITHUB_API_URL: &str = "https://api.github.com/repos/indygreg/python-build-standalone/tags?per_page=100";
    let client = reqwest::Client::builder().user_agent("mlvm-rust-app").build()?;
    let response = client.get(GITHUB_API_URL).send().await?.error_for_status()
        .context("Failed to fetch tags from GitHub API. The API might be temporarily down.")?;
    let tags: Vec<GitHubTag> = response.json().await?;
    Ok(tags.into_iter().map(|t| t.name).collect())
}

async fn find_latest_tag_for_version(version: &str) -> Result<String> {
    let tags = fetch_all_tags().await?;
    let version_prefix = format!("v{}", version);
    
    let latest_tag = tags
        .into_iter()
        .filter(|tag| tag.starts_with(&version_prefix))
        .max(); // .max() correctly finds the latest date string, e.g., ...+20240524 > ...+20240413

    match latest_tag {
        Some(tag) => Ok(tag),
        None => bail!("Could not find any build for Python version {}. Check the version number or run `list-remote`.", version),
    }
}

pub async fn install(version: &str) -> Result<()> {
    println!("Finding latest build for Python {}...", version);
    let exact_tag = find_latest_tag_for_version(version).await?;
    println!("Found latest build tag: {}", exact_tag);

    let release_url = format!(
        "https://api.github.com/repos/indygreg/python-build-standalone/releases/tags/{}",
        exact_tag
    );
    let client = reqwest::Client::builder().user_agent("mlvm-rust-app").build()?;
    let response = client.get(&release_url).send().await?.error_for_status()
        .with_context(|| format!("Could not get release details for tag: {}.", exact_tag))?;
    let release: GitHubRelease = response.json().await?;

    let target_platform = {
        let arch = match env::consts::ARCH {
            "x86_64" => "x86_64",
            "aarch64" => "aarch64",
            other => bail!("Unsupported architecture: {}", other),
        };
        let os = match env::consts::OS {
            "linux" => "unknown-linux-gnu",
            "macos" => "apple-darwin",
            "windows" => "pc-windows-msvc",
            other => bail!("Unsupported OS: {}", other),
        };
        format!("{}-{}", arch, os)
    };

    println!("Finding download URL for platform: {}", target_platform);
    let download_url = release.assets.iter()
        .find(|asset| asset.name.contains(&target_platform) && asset.name.ends_with("install_only.tar.zst"))
        .map(|asset| &asset.browser_download_url)
        .with_context(|| format!("Could not find a compatible Python build for your platform ({}) in release {}.", target_platform, exact_tag))?;
        
    println!("Downloading from {}...", download_url);

    let response = reqwest::get(download_url).await?.error_for_status()?;
    let compressed_bytes = response.bytes().await?;

    let decoder = zstd::stream::decode_all(Cursor::new(compressed_bytes))?;
    let mut archive = Archive::new(&decoder[..]);

    let mlvm_dir = dirs::home_dir().context("Could not find home directory")?.join(".mlvm");
    let lang_dir = mlvm_dir.join("python");
    let install_path = lang_dir.join(version);
    if install_path.exists() {
        println!("Version {} is already installed.", version);
        return Ok(());
    }

    println!("Unpacking to {:?}...", install_path);
    let temp_unpack_path = lang_dir.join("temp_unpack");
    if temp_unpack_path.exists() { fs::remove_dir_all(&temp_unpack_path)?; }
    archive.unpack(&temp_unpack_path)?;
    
    fs::rename(temp_unpack_path.join("python"), &install_path)?;
    fs::remove_dir_all(temp_unpack_path)?;

    println!("Successfully installed Python {}", version);
    Ok(())
}

// src/commands/python.rs

// use anyhow::{bail, Context, Result};
// use serde::Deserialize;
// use std::collections::HashSet;
// use std::env;
// use std::fs;
// use std::io::Cursor;
// use tar::Archive;

// // --- Structs for deserializing the versions.json manifest ---

// #[derive(Deserialize, Debug, Clone)]
// struct PythonBuild {
//     name: String,
//     version: String,
//     url: String,
// }

// // --- Helper function to fetch and parse the build manifest ---
// // This is the new, reliable core of our logic.
// async fn fetch_build_manifest() -> Result<Vec<PythonBuild>> {
//     const MANIFEST_URL: &str = "https://raw.githubusercontent.com/astral-sh/python-build-standalone/main/versions.json";

//     println!("Fetching build manifest...");
//     let response = reqwest::get(MANIFEST_URL)
//         .await?
//         .error_for_status()
//         .context("Failed to download the Python build manifest.")?;

//     let builds: Vec<PythonBuild> = response
//         .json()
//         .await?
//         .context("Failed to parse the Python build manifest.")?;

//     Ok(builds)
// }

// // --- `list-remote`: Reads the manifest to create a clean, accurate list ---

// pub async fn list_remote() -> Result<()> {
//     let builds = fetch_build_manifest().await?;
//     let mut versions_set: HashSet<String> = HashSet::new();

//     for build in builds {
//         versions_set.insert(build.version);
//     }

//     if versions_set.is_empty() {
//         bail!("Could not find any Python versions in the manifest.");
//     }

//     let mut versions: Vec<_> = versions_set.into_iter().collect();
//     // Sort versions correctly (e.g., 3.12 before 3.11)
//     versions.sort_by(|a, b| {
//         let a_parts: Vec<u32> = a.split('.').map(|s| s.parse().unwrap_or(0)).collect();
//         let b_parts: Vec<u32> = b.split('.').map(|s| s.parse().unwrap_or(0)).collect();
//         b_parts.cmp(&a_parts)
//     });

//     println!("\nAvailable versions:");
//     for version in versions {
//         println!("- {}", version);
//     }

//     Ok(())
// }

// // --- `install`: Uses the manifest to find the exact download URL ---

// pub async fn install(version: &str) -> Result<()> {
//     let builds = fetch_build_manifest().await?;

//     // Determine the user's platform identifier string.
//     let target_platform = {
//         let arch = match env::consts::ARCH {
//             "x86_64" => "x86_64",
//             "aarch64" => "aarch64",
//             other => bail!("Unsupported architecture: {}", other),
//         };
//         let os = match env::consts::OS {
//             "linux" => "unknown-linux-gnu",
//             "macos" => "apple-darwin",
//             "windows" => "pc-windows-msvc",
//             other => bail!("Unsupported OS: {}", other),
//         };
//         format!("{}-{}", arch, os)
//     };

//     println!("Searching for Python {} for your platform ({})...", version, target_platform);

//     // Find the correct build in the manifest.
//     let target_build = builds
//         .iter()
//         .find(|build| build.version == version && build.name.contains(&target_platform) && build.name.contains("install_only"))
//         .ok_or_else(|| anyhow::anyhow!("Could not find a compatible build for Python version {} and your platform.", version))?;

//     println!("Found build. Downloading from {}...", target_build.url);
    
//     // Download, decompress, and install.
//     let response = reqwest::get(&target_build.url).await?.error_for_status()?;
//     let compressed_bytes = response.bytes().await?;

//     let decoder = zstd::stream::decode_all(Cursor::new(compressed_bytes))?;
//     let mut archive = Archive::new(&decoder[..]);

//     let mlvm_dir = dirs::home_dir().context("Could not find home directory")?.join(".mlvm");
//     let lang_dir = mlvm_dir.join("python");
//     let install_path = lang_dir.join(version);
//     if install_path.exists() {
//         println!("Version {} is already installed.", version);
//         return Ok(());
//     }

//     println!("Unpacking to {:?}...", install_path);
//     let temp_unpack_path = lang_dir.join("temp_unpack");
//     if temp_unpack_path.exists() { fs::remove_dir_all(&temp_unpack_path)?; }
//     archive.unpack(&temp_unpack_path)?;
    
//     fs::rename(temp_unpack_path.join("python"), &install_path)?;
//     fs::remove_dir_all(temp_unpack_path)?;

//     println!("Successfully installed Python {}", version);
//     Ok(())
// }