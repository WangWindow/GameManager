use crate::services::ArchiveService;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
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

pub async fn download_and_install(
    app: &AppHandle,
    version: String,
    flavor: NwjsFlavor,
    target: String,
) -> Result<NwjsInstallResult, String> {
    let url = build_download_url(&version, flavor, &target);
    let task_id = Uuid::new_v4().to_string();

    let runtime_root = app_runtime_root(app)?;
    crate::services::path::ensure_dir(&runtime_root)?;

    let download_dir = runtime_root.join("_downloads");
    crate::services::path::ensure_dir(&download_dir)?;

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

    let _ = app.emit(
        "nwjs_install_stage",
        serde_json::json!({
            "taskId": task_id,
            "version": version,
            "flavor": flavor,
            "target": target,
            "stage": "downloaded",
            "label": "下载完成，正在解压…"
        }),
    );

    // 使用 ArchiveService 进行解压
    let archive_service = ArchiveService::new();

    let tmp = TempDir::new().map_err(|e| format!("failed to create temp dir: {e}"))?;
    let tmp_extract = tmp.path().join("extract");
    crate::services::path::ensure_dir(&tmp_extract)?;

    // 根据扩展名自动选择解压方法
    archive_service.extract_auto(&archive_path, &tmp_extract)?;

    // 查找单一根目录
    let extracted_root = archive_service
        .find_single_root_dir(&tmp_extract)
        .unwrap_or(tmp_extract);

    let install_dir = runtime_root
        .join(&version)
        .join(match flavor {
            NwjsFlavor::Normal => "normal",
            NwjsFlavor::Sdk => "sdk",
        })
        .join(&target);

    // 删除已存在的安装目录
    archive_service.remove_dir_if_exists(&install_dir)?;
    // 移动解压后的文件到安装目录
    archive_service.move_dir(&extracted_root, &install_dir)?;

    // Best-effort cleanup of downloaded archive.
    let _ = std::fs::remove_file(&archive_path);

    let _ = app.emit(
        "nwjs_install_stage",
        serde_json::json!({
            "taskId": task_id,
            "version": version,
            "flavor": flavor,
            "target": target,
            "stage": "installed",
            "label": "解压完成"
        }),
    );

    Ok(NwjsInstallResult {
        task_id,
        version,
        flavor,
        target,
        install_dir: install_dir.to_string_lossy().to_string(),
    })
}
