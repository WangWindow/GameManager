use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager};
use tempfile::TempDir;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NwjsFlavor {
    Normal,
    Sdk,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NwjsStableInfo {
    pub version: String,
    pub target: String,
    pub normal_url: String,
    pub sdk_url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NwjsInstallResult {
    pub task_id: String,
    pub version: String,
    pub flavor: NwjsFlavor,
    pub target: String,
    pub install_dir: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NwjsDownloadProgress {
    pub task_id: String,
    pub version: String,
    pub flavor: NwjsFlavor,
    pub target: String,
    pub downloaded: u64,
    pub total: Option<u64>,
    pub percent: Option<u8>,
}

pub fn current_target() -> Result<String, String> {
    // Keep aligned with NW.js official downloads naming.
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let target = match (os, arch) {
        ("windows", "x86_64") => "win-x64",
        ("windows", "x86") => "win-ia32",
        ("windows", "aarch64") => "win-arm64",
        ("linux", "x86_64") => "linux-x64",
        ("linux", "x86") => "linux-ia32",
        ("macos", "x86_64") => "osx-x64",
        ("macos", "aarch64") => "osx-arm64",
        _ => {
            return Err(format!(
                "unsupported platform for official NW.js downloads: {os}-{arch}"
            ));
        }
    };

    Ok(target.to_string())
}

fn nwjs_archive_ext(target: &str) -> &'static str {
    if target.starts_with("linux-") {
        "tar.gz"
    } else {
        "zip"
    }
}

pub fn build_download_url(version: &str, flavor: NwjsFlavor, target: &str) -> String {
    let ext = nwjs_archive_ext(target);
    let prefix = match flavor {
        NwjsFlavor::Normal => "nwjs",
        NwjsFlavor::Sdk => "nwjs-sdk",
    };

    format!("https://dl.nwjs.io/v{version}/{prefix}-v{version}-{target}.{ext}")
}

#[derive(Debug, Deserialize)]
struct VersionsJson {
    stable: Option<String>,
    latest: Option<String>,
}

pub async fn fetch_stable_version() -> Result<String, String> {
    // The downloads page is rendered dynamically; the stable/latest versions are published in versions.json.
    let json_text = reqwest::Client::new()
        .get("https://nwjs.io/versions.json")
        .send()
        .await
        .map_err(|e| format!("failed to fetch versions.json: {e}"))?
        .text()
        .await
        .map_err(|e| format!("failed to read versions.json: {e}"))?;

    let versions: VersionsJson = serde_json::from_str(&json_text)
        .map_err(|e| format!("failed to parse versions.json: {e}"))?;

    let raw = versions
        .stable
        .or(versions.latest)
        .ok_or_else(|| "versions.json missing stable/latest".to_string())?;

    let ver = raw
        .trim()
        .trim_start_matches(|c: char| c == 'v' || c == 'V')
        .to_string();

    if ver.is_empty() {
        return Err("failed to parse stable version from versions.json".to_string());
    }

    Ok(ver)
}

pub async fn get_stable_info() -> Result<NwjsStableInfo, String> {
    let version = fetch_stable_version().await?;
    let target = current_target()?;

    Ok(NwjsStableInfo {
        normal_url: build_download_url(&version, NwjsFlavor::Normal, &target),
        sdk_url: build_download_url(&version, NwjsFlavor::Sdk, &target),
        version,
        target,
    })
}

fn app_runtime_root(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to resolve app data dir: {e}"))?;
    Ok(app_data_dir.join("runtimes").join("nwjs"))
}

fn ensure_dir(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path)
        .map_err(|e| format!("failed to create dir {}: {e}", path.display()))
}

fn safe_join(base: &Path, entry: &Path) -> Result<PathBuf, String> {
    let mut out = PathBuf::from(base);

    for comp in entry.components() {
        match comp {
            Component::Normal(p) => out.push(p),
            Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => {
                return Err(format!("unsafe path in archive entry: {}", entry.display()));
            }
        }
    }

    Ok(out)
}

fn extract_tar_gz(archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = File::open(archive_path)
        .map_err(|e| format!("failed to open {}: {e}", archive_path.display()))?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);

    for entry in archive
        .entries()
        .map_err(|e| format!("tar read error: {e}"))?
    {
        let mut entry = entry.map_err(|e| format!("tar entry error: {e}"))?;
        let entry_path = entry
            .path()
            .map_err(|e| format!("tar entry path error: {e}"))?
            .to_path_buf();

        let out_path = safe_join(dest_dir, &entry_path)?;
        if let Some(parent) = out_path.parent() {
            ensure_dir(parent)?;
        }
        entry
            .unpack(&out_path)
            .map_err(|e| format!("tar unpack error: {e}"))?;
    }

    Ok(())
}

fn extract_zip(archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = File::open(archive_path)
        .map_err(|e| format!("failed to open {}: {e}", archive_path.display()))?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| format!("zip open error: {e}"))?;

    for i in 0..zip.len() {
        let mut entry = zip
            .by_index(i)
            .map_err(|e| format!("zip entry error: {e}"))?;
        let name = entry
            .enclosed_name()
            .ok_or_else(|| "unsafe path in zip entry".to_string())?;
        let out_path = dest_dir.join(&name);

        if entry.is_dir() {
            ensure_dir(&out_path)?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            ensure_dir(parent)?;
        }

        let mut out = File::create(&out_path).map_err(|e| format!("zip write error: {e}"))?;
        std::io::copy(&mut entry, &mut out).map_err(|e| format!("zip extract error: {e}"))?;
    }

    Ok(())
}

fn find_single_root_dir(dir: &Path) -> Option<PathBuf> {
    let mut entries = std::fs::read_dir(dir).ok()?;
    let mut only: Option<PathBuf> = None;

    while let Some(Ok(e)) = entries.next() {
        let p = e.path();
        // ignore hidden/system artifacts
        if p.file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("")
            .starts_with('.')
        {
            continue;
        }
        if only.is_some() {
            return None;
        }
        only = Some(p);
    }

    match only {
        Some(p) if p.is_dir() => Some(p),
        _ => None,
    }
}

fn remove_dir_if_exists(path: &Path) -> Result<(), String> {
    if path.exists() {
        std::fs::remove_dir_all(path)
            .map_err(|e| format!("failed to remove {}: {e}", path.display()))?;
    }
    Ok(())
}

fn move_dir(src: &Path, dst: &Path) -> Result<(), String> {
    if let Some(parent) = dst.parent() {
        ensure_dir(parent)?;
    }

    // Prefer rename, fallback to copy+remove for cross-device.
    match std::fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(_) => {
            copy_dir_all(src, dst)?;
            std::fs::remove_dir_all(src).map_err(|e| format!("failed to cleanup temp dir: {e}"))?;
            Ok(())
        }
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
    ensure_dir(dst)?;
    for entry in std::fs::read_dir(src).map_err(|e| format!("read_dir error: {e}"))? {
        let entry = entry.map_err(|e| format!("read_dir entry error: {e}"))?;
        let ty = entry
            .file_type()
            .map_err(|e| format!("file_type error: {e}"))?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            std::fs::copy(&from, &to).map_err(|e| format!("copy error: {e}"))?;
        }
    }
    Ok(())
}

pub async fn download_and_install(
    app: &AppHandle,
    version: String,
    flavor: NwjsFlavor,
    target: String,
) -> Result<NwjsInstallResult, String> {
    let url = build_download_url(&version, flavor, &target);
    let task_id = Uuid::new_v4().to_string();

    let runtime_root = app_runtime_root(app)?;
    ensure_dir(&runtime_root)?;

    let download_dir = runtime_root.join("_downloads");
    ensure_dir(&download_dir)?;

    let ext = nwjs_archive_ext(&target);
    let archive_path = download_dir.join(format!("{task_id}-{version}-{target}.{ext}"));

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("download request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("download failed: {e}"))?;

    let total = resp.content_length();
    let mut downloaded: u64 = 0;

    let mut file = File::create(&archive_path)
        .map_err(|e| format!("failed to create {}: {e}", archive_path.display()))?;

    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("download stream error: {e}"))?;
        file.write_all(&chunk)
            .map_err(|e| format!("write error: {e}"))?;
        downloaded += chunk.len() as u64;

        let percent = total.and_then(|t| {
            if t == 0 {
                None
            } else {
                let p = ((downloaded as f64 / t as f64) * 100.0).floor() as u8;
                Some(p.min(100))
            }
        });

        let _ = app.emit(
            "nwjs_download_progress",
            NwjsDownloadProgress {
                task_id: task_id.clone(),
                version: version.clone(),
                flavor,
                target: target.clone(),
                downloaded,
                total,
                percent,
            },
        );
    }

    file.flush().ok();

    let tmp = TempDir::new().map_err(|e| format!("failed to create temp dir: {e}"))?;
    let tmp_extract = tmp.path().join("extract");
    ensure_dir(&tmp_extract)?;

    match ext {
        "tar.gz" => extract_tar_gz(&archive_path, &tmp_extract)?,
        "zip" => extract_zip(&archive_path, &tmp_extract)?,
        _ => return Err(format!("unsupported archive type: {ext}")),
    }

    let extracted_root = find_single_root_dir(&tmp_extract).unwrap_or(tmp_extract);

    let install_dir = runtime_root
        .join(&version)
        .join(match flavor {
            NwjsFlavor::Normal => "normal",
            NwjsFlavor::Sdk => "sdk",
        })
        .join(&target);

    remove_dir_if_exists(&install_dir)?;
    move_dir(&extracted_root, &install_dir)?;

    // Best-effort cleanup of downloaded archive.
    let _ = std::fs::remove_file(&archive_path);

    Ok(NwjsInstallResult {
        task_id,
        version,
        flavor,
        target,
        install_dir: install_dir.to_string_lossy().to_string(),
    })
}
