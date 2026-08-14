use crate::services::fs::ArchiveService;
use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

const PATCH_FILE: &str = "compatibility.rb";
const DEFAULT_PATCH: &str = r#"# Global mkxp-z compatibility patch.
# It is loaded before every game launched by GameManager.
if defined?($RGSS_SCRIPTS) && $RGSS_SCRIPTS
  $RGSS_SCRIPTS.delete_if do |script|
    name = script[1].to_s.downcase
    if name == "steam_acheivement" || name == "steam_achievement"
      puts "[mkxp patch] Disabled incompatible script: #{script[1]}"
      true
    else
      false
    end
  end
end
"#;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MkxpzImportResult {
    pub version: String,
    pub install_dir: String,
    pub executable_path: String,
}

pub fn import_archive(app: &AppHandle, archive_path: &Path) -> Result<MkxpzImportResult, String> {
    if !archive_path.is_file()
        || !archive_path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
    {
        return Err("请选择 mkxp-z GitHub Actions 下载的 ZIP 文件".to_string());
    }

    let runtime_root = runtime_root(app)?;
    fs::create_dir_all(&runtime_root)
        .map_err(|error| format!("无法创建 mkxp-z 运行时目录: {error}"))?;

    let transaction_id = Uuid::new_v4().to_string();
    let staging_dir = runtime_root.join(format!(".staging-{transaction_id}"));
    let previous_dir = runtime_root.join(format!(".previous-{transaction_id}"));
    let current_dir = runtime_root.join("current");
    let archive_service = ArchiveService::new();
    let _ = fs::remove_dir_all(&previous_dir);

    let install_result = (|| {
        archive_service.extract_zip(archive_path, &staging_dir)?;
        let executable = find_elf_executable(&staging_dir)
            .ok_or_else(|| "压缩包中未找到适用于当前架构的 mkxp-z Linux 可执行文件".to_string())?;
        let executable_relative = executable
            .strip_prefix(&staging_dir)
            .map_err(|error| format!("无法解析 mkxp-z 可执行文件路径: {error}"))?
            .to_path_buf();
        mark_executable(&executable)?;

        preserve_patches(&current_dir, &staging_dir)?;
        ensure_compatibility_patch(&staging_dir)?;

        if current_dir.exists() {
            fs::rename(&current_dir, &previous_dir)
                .map_err(|error| format!("无法准备替换旧 mkxp-z 运行时: {error}"))?;
        }

        if let Err(error) = fs::rename(&staging_dir, &current_dir) {
            if previous_dir.exists() {
                let _ = fs::rename(&previous_dir, &current_dir);
            }
            return Err(format!("无法安装 mkxp-z 运行时: {error}"));
        }

        let executable_path = current_dir.join(executable_relative);
        if !executable_path.is_file() {
            return Err("mkxp-z 安装完成后无法找到可执行文件".to_string());
        }

        let _ = fs::remove_dir_all(&previous_dir);
        Ok(MkxpzImportResult {
            version: archive_label(archive_path),
            install_dir: current_dir.to_string_lossy().to_string(),
            executable_path: executable_path.to_string_lossy().to_string(),
        })
    })();

    if install_result.is_err() {
        let _ = fs::remove_dir_all(&staging_dir);
    }

    install_result
}

fn runtime_root(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("无法获取应用数据目录: {error}"))?;
    Ok(app_data_dir.join("runtimes").join("mkxpz"))
}

fn archive_label(archive_path: &Path) -> String {
    archive_path
        .file_stem()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("GitHub Actions build")
        .to_string()
}

fn find_elf_executable(root: &Path) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    collect_files(root, &mut candidates).ok()?;
    candidates.sort_by_key(|path| {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        (!name.contains("mkxp"), path.clone())
    });
    candidates.into_iter().find(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.to_ascii_lowercase().contains("mkxp"))
            && is_elf_binary(path)
            && is_current_architecture(path)
    })
}

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|error| format!("无法读取解压目录: {error}"))? {
        let entry = entry.map_err(|error| format!("无法读取解压目录条目: {error}"))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("无法读取文件类型: {error}"))?;
        if file_type.is_dir() {
            collect_files(&path, files)?;
        } else if file_type.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn is_elf_binary(path: &Path) -> bool {
    let Ok(mut file) = fs::File::open(path) else {
        return false;
    };
    let mut magic = [0_u8; 4];
    file.read_exact(&mut magic).is_ok() && magic == [0x7f, b'E', b'L', b'F']
}

fn is_current_architecture(path: &Path) -> bool {
    let Ok(mut file) = fs::File::open(path) else {
        return false;
    };
    let mut header = [0_u8; 20];
    if file.read_exact(&mut header).is_err() || header[..4] != [0x7f, b'E', b'L', b'F'] {
        return false;
    }
    if header[5] != 1 {
        return false;
    }
    let machine = u16::from_le_bytes([header[18], header[19]]);
    matches!(
        (std::env::consts::ARCH, machine),
        ("x86_64", 62) | ("aarch64", 183) | ("x86", 3)
    )
}

#[cfg(unix)]
fn mark_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .map_err(|error| format!("无法读取 mkxp-z 权限: {error}"))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
        .map_err(|error| format!("无法设置 mkxp-z 执行权限: {error}"))
}

#[cfg(not(unix))]
fn mark_executable(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn preserve_patches(current_dir: &Path, staging_dir: &Path) -> Result<(), String> {
    let current_patches = current_dir.join("patches");
    if !current_patches.is_dir() {
        return Ok(());
    }

    copy_dir(&current_patches, &staging_dir.join("patches"))
}

pub fn ensure_compatibility_patch(runtime_dir: &Path) -> Result<(), String> {
    let patch_path = runtime_dir.join("patches").join(PATCH_FILE);
    let parent = patch_path
        .parent()
        .ok_or_else(|| "无法创建 mkxp-z 补丁目录".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("无法创建 mkxp-z 补丁目录: {error}"))?;

    if patch_path.exists() {
        return Ok(());
    }

    fs::write(&patch_path, DEFAULT_PATCH).map_err(|error| format!("无法创建兼容补丁: {error}"))
}

fn copy_dir(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|error| format!("无法创建补丁目录: {error}"))?;
    for entry in fs::read_dir(source).map_err(|error| format!("无法读取补丁目录: {error}"))?
    {
        let entry = entry.map_err(|error| format!("无法读取补丁目录条目: {error}"))?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry
            .file_type()
            .map_err(|error| format!("无法读取补丁文件类型: {error}"))?
            .is_dir()
        {
            copy_dir(&source_path, &destination_path)?;
        } else {
            fs::copy(&source_path, &destination_path)
                .map_err(|error| format!("无法保留 mkxp-z 补丁: {error}"))?;
        }
    }
    Ok(())
}
