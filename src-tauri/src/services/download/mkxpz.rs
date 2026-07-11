//! mkxp-z 导入安装服务。
//!
//! mkxp-z 没有规范的 Release 下载，只能从 GitHub Actions artifact 获取。
//! 本模块提供从本地 ZIP 压缩包导入安装的功能。

use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MkxpzImportResult {
    pub version: String,
    pub install_dir: String,
}

/// 从文件名中提取可读的版本标识。
///
/// GitHub Actions artifact 命名格式多样，例如：
/// - `mkxpz-v2.4.2.zip`
/// - `mkxp-z.linux.ubuntu.22.04.x86_64.PR342-83cc9fd.zip`
/// - `mkxp-z.linux.ubuntu.22.04.x86_64.autobuild-abc1234.zip`
///
/// 优先提取语义化版本号，否则用 `{branch}-{short_sha}` 或 UUID 兜底。
fn parse_version_from_filename(file_stem: &str) -> String {
    // 1) mkxpz-v2.4.2 / mkxp-z-v2.4.2
    if let Some(rest) = file_stem
        .strip_prefix("mkxpz-v")
        .or_else(|| file_stem.strip_prefix("mkxp-z-v"))
    {
        let ver = rest.split('.').take(3).collect::<Vec<_>>().join(".");
        if ver.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return ver;
        }
    }

    // 2) mkxp-z.{target}.{branch}-{sha} → 取 branch-sha 作为版本
    if let Some(rest) = file_stem.strip_prefix("mkxp-z.") {
        // 去掉 target 部分（例如 linux.ubuntu.22.04.x86_64）
        if let Some(tail) = rest.splitn(2, '.').nth(1) {
            // tail 格式：{branch}-{sha}
            let variant = tail.replace('.', "-");
            // 截断过长的 sha
            if variant.len() > 30 {
                let (short, _) = variant.split_at(30);
                return short.to_string();
            }
            return variant;
        }
    }

    // 3) 兜底：用 UUID
    Uuid::new_v4().to_string()
}

/// 从本地 ZIP 压缩包导入安装 mkxp-z。
///
/// 解压到 `{app_data}/runtimes/mkxpz/{version}/`，
/// 版本号从压缩包文件名中智能提取。
pub fn import_from_archive(
    app: &AppHandle,
    archive_path: &Path,
) -> Result<MkxpzImportResult, String> {
    if !archive_path.is_file() {
        return Err(format!("文件不存在: {}", archive_path.display()));
    }

    let file_stem = archive_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "无法解析文件名".to_string())?;

    let version = parse_version_from_filename(file_stem);

    let runtime_root = app_runtime_root(app)?;
    crate::utils::path::ensure_dir(&runtime_root)?;

    let archive_service = crate::services::ArchiveService::new();
    let staging_dir = runtime_root.join(format!(".import-{}", Uuid::new_v4()));
    if staging_dir.exists() {
        std::fs::remove_dir_all(&staging_dir)
            .map_err(|e| format!("清理临时目录失败: {e}"))?;
    }
    crate::utils::path::ensure_dir(&staging_dir)?;

    let version_dir = runtime_root.join(&version);
    let result = (|| -> Result<MkxpzImportResult, String> {
        // 先解压到临时目录，再规范化到最终 runtimes 目录，避免半成品污染。
        archive_service.extract_zip(archive_path, &staging_dir)?;

        let content_root = archive_service
            .find_single_root_dir(&staging_dir)
            .unwrap_or_else(|| staging_dir.clone());

        if !has_mkxpz_binary(&content_root) {
            return Err(format!(
                "导入的压缩包中未找到 mkxp-z 可执行文件（mkxp-z.x86_64/mkxp-z）。\
                请确认下载的是正确的 mkxp-z Linux 构建包。"
            ));
        }

        if version_dir.exists() {
            std::fs::remove_dir_all(&version_dir)
                .map_err(|e| format!("清理旧版本目录失败: {e}"))?;
        }
        crate::utils::path::ensure_dir(&version_dir)?;

        if content_root == staging_dir {
            move_dir_contents(&staging_dir, &version_dir)?;
        } else {
            archive_service.move_dir(&content_root, &version_dir)?;
        }

        Ok(MkxpzImportResult {
            version,
            install_dir: version_dir.to_string_lossy().to_string(),
        })
    })();

    let _ = std::fs::remove_dir_all(&staging_dir);

    result
}

/// 检查目录中是否包含 mkxp-z 可执行文件
fn has_mkxpz_binary(dir: &Path) -> bool {
    if dir.join("mkxp-z.x86_64").is_file() || dir.join("mkxp-z").is_file() {
        return true;
    }
    // 也检查子目录
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir()
                && (path.join("mkxp-z.x86_64").is_file() || path.join("mkxp-z").is_file())
            {
                return true;
            }
        }
    }
    false
}

/// 将目录中的所有顶层内容移动到目标目录。
fn move_dir_contents(src: &Path, dst: &Path) -> Result<(), String> {
    let archive_service = crate::services::ArchiveService::new();
    let entries = std::fs::read_dir(src).map_err(|e| format!("读取临时目录失败: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取临时目录条目失败: {e}"))?;
        let from = entry.path();
        let to = dst.join(entry.file_name());

        if from.is_dir() {
            archive_service.move_dir(&from, &to)?;
        } else {
            std::fs::rename(&from, &to).map_err(|e| format!("移动文件失败: {e}"))?;
        }
    }

    let _ = std::fs::remove_dir_all(src);
    Ok(())
}

/// mkxp-z 运行时根目录
fn app_runtime_root(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {e}"))?;
    Ok(app_data_dir.join("runtimes").join("mkxpz"))
}
