use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::{AppPaths, CoreError, Result};

const PATCH_FILE: &str = "compatibility.rb";
const DEFAULT_PATCH: &str = include_str!("../../assets/mkxpz/patches/compatibility.rb");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MkxpzInstallResult {
    pub version: String,
    pub install_dir: PathBuf,
    pub executable_path: PathBuf,
}

pub fn import_mkxpz_archive(paths: &AppPaths, archive_path: &Path) -> Result<MkxpzInstallResult> {
    if !archive_path.is_file()
        || !archive_path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
    {
        return Err(CoreError::Configuration(
            "mkxp-z import requires a ZIP archive".to_owned(),
        ));
    }
    let root = paths.mkxpz_runtime_root();
    let staging = root.join(".staging-import");
    if staging.exists() {
        fs::remove_dir_all(&staging)?;
    }
    fs::create_dir_all(&staging)?;
    let extract = staging.join("extract");
    fs::create_dir_all(&extract)?;
    extract_zip(archive_path, &extract)?;
    let source = single_root(&extract).unwrap_or(extract.clone());
    let executable = find_mkxp_executable(&source).ok_or_else(|| {
        CoreError::Engine("mkxp-z archive has no compatible executable".to_owned())
    })?;
    let current = root.join("current");
    let previous = root.join(".previous-import");
    if previous.exists() {
        fs::remove_dir_all(&previous)?;
    }
    let previous_patch = current.join("patches").join(PATCH_FILE);
    let previous_patch_bytes = previous_patch
        .is_file()
        .then(|| fs::read(&previous_patch))
        .transpose()?;
    if current.exists() {
        fs::rename(&current, &previous)?;
    }
    if let Err(error) = fs::rename(&source, &current) {
        if previous.exists() {
            let _ = fs::rename(&previous, &current);
        }
        let _ = fs::remove_dir_all(&staging);
        return Err(error.into());
    }
    let relative = executable
        .strip_prefix(&source)
        .map_err(|error| CoreError::Engine(error.to_string()))?;
    let executable_path = current.join(relative);
    mark_executable(&executable_path)?;
    ensure_compatibility_patch(&current)?;
    if let Some(bytes) = previous_patch_bytes {
        let patch = current.join("patches").join(PATCH_FILE);
        if patch.is_file() {
            fs::write(patch, bytes)?;
        }
    }
    let _ = fs::remove_dir_all(&previous);
    let _ = fs::remove_dir_all(&staging);
    Ok(MkxpzInstallResult {
        version: archive_path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("local")
            .to_owned(),
        install_dir: current,
        executable_path,
    })
}

pub fn ensure_compatibility_patch(runtime_dir: &Path) -> Result<()> {
    let path = runtime_dir.join("patches").join(PATCH_FILE);
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    File::create(path)?.write_all(DEFAULT_PATCH.as_bytes())?;
    Ok(())
}

fn extract_zip(archive: &Path, destination: &Path) -> Result<()> {
    let file = File::open(archive)?;
    let mut zip =
        zip::ZipArchive::new(file).map_err(|error| CoreError::Engine(error.to_string()))?;
    for index in 0..zip.len() {
        let mut entry = zip
            .by_index(index)
            .map_err(|error| CoreError::Engine(error.to_string()))?;
        let Some(relative) = entry.enclosed_name() else {
            return Err(CoreError::Engine("archive contains unsafe path".to_owned()));
        };
        let output = destination.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(output)?;
            continue;
        }
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut output_file = File::create(output)?;
        std::io::copy(&mut entry, &mut output_file)?;
    }
    Ok(())
}

fn single_root(path: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(path)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    (entries.len() == 1 && entries[0].is_dir()).then(|| entries[0].clone())
}

fn find_mkxp_executable(root: &Path) -> Option<PathBuf> {
    let mut files = Vec::new();
    collect_files(root, &mut files).ok()?;
    files.sort();
    files.into_iter().find(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.to_ascii_lowercase().contains("mkxp") && is_elf(path))
    })
}

fn collect_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn is_elf(path: &Path) -> bool {
    let Ok(mut file) = File::open(path) else {
        return false;
    };
    let mut header = [0; 20];
    if file.read_exact(&mut header).is_err() || header[..4] != [0x7f, b'E', b'L', b'F'] {
        return false;
    }
    let machine = if header[5] == 1 {
        u16::from_le_bytes([header[18], header[19]])
    } else {
        u16::from_be_bytes([header[18], header[19]])
    };
    matches!(
        (std::env::consts::ARCH, header[4], machine),
        ("x86_64", 2, 62) | ("aarch64", 2, 183) | ("x86", 1, 3)
    )
}

#[cfg(unix)]
fn mark_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn mark_executable(_: &Path) -> Result<()> {
    Ok(())
}
