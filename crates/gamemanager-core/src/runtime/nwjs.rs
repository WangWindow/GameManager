use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use futures_util::future::BoxFuture;

use crate::{CoreError, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NwjsFlavor {
    Normal,
    Sdk,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NwjsStableInfo {
    pub version: String,
    pub target: String,
    pub normal_url: String,
    pub sdk_url: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NwjsInstallResult {
    pub version: String,
    pub flavor: NwjsFlavor,
    pub target: String,
    pub install_dir: PathBuf,
}

pub trait HttpClient: Send + Sync {
    fn get(&self, url: &str) -> BoxFuture<'static, Result<Vec<u8>>>;
}

pub(crate) struct UnavailableHttpClient;
impl HttpClient for UnavailableHttpClient {
    fn get(&self, _: &str) -> BoxFuture<'static, Result<Vec<u8>>> {
        Box::pin(async {
            Err(CoreError::Engine(
                "network client is not configured".to_owned(),
            ))
        })
    }
}

pub fn current_target() -> Result<String> {
    let target = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => "win-x64",
        ("windows", "x86") => "win-ia32",
        ("windows", "aarch64") => "win-arm64",
        ("linux", "x86_64") => "linux-x64",
        ("linux", "x86") => "linux-ia32",
        ("macos", "x86_64") => "osx-x64",
        ("macos", "aarch64") => "osx-arm64",
        (os, arch) => {
            return Err(CoreError::Engine(format!(
                "unsupported NW.js target: {os}-{arch}"
            )));
        }
    };
    Ok(target.to_owned())
}

pub fn build_download_url(version: &str, flavor: NwjsFlavor, target: &str) -> String {
    let extension = if target.starts_with("linux-") {
        "tar.gz"
    } else {
        "zip"
    };
    let prefix = match flavor {
        NwjsFlavor::Normal => "nwjs",
        NwjsFlavor::Sdk => "nwjs-sdk",
    };
    format!("https://dl.nwjs.io/v{version}/{prefix}-v{version}-{target}.{extension}")
}

pub(crate) async fn download_and_install(
    paths: &crate::AppPaths,
    client: Arc<dyn HttpClient>,
    version: &str,
    flavor: NwjsFlavor,
    target: &str,
) -> Result<NwjsInstallResult> {
    let url = build_download_url(version, flavor, target);
    let archive = client.get(&url).await?;
    let root = paths.nwjs_runtime_root();
    let staging = root.join(format!(".staging-{version}-{target}"));
    if staging.exists() {
        std::fs::remove_dir_all(&staging)?;
    }
    std::fs::create_dir_all(&staging)?;
    let archive_path = staging.join(if target.starts_with("linux-") {
        "runtime.tar.gz"
    } else {
        "runtime.zip"
    });
    File::create(&archive_path)?.write_all(&archive)?;
    let extract = staging.join("extract");
    std::fs::create_dir_all(&extract)?;
    if target.starts_with("linux-") {
        let file = File::open(&archive_path)?;
        let decoder = flate2::read::GzDecoder::new(file);
        tar::Archive::new(decoder)
            .unpack(&extract)
            .map_err(|error| CoreError::Engine(error.to_string()))?;
    } else {
        extract_zip(&archive_path, &extract)?;
    }
    let source = single_root(&extract).unwrap_or(extract.clone());
    let install = root
        .join(version)
        .join(match flavor {
            NwjsFlavor::Normal => "normal",
            NwjsFlavor::Sdk => "sdk",
        })
        .join(target);
    if install.exists() {
        std::fs::remove_dir_all(&install)?;
    }
    std::fs::create_dir_all(
        install
            .parent()
            .ok_or_else(|| CoreError::InvalidPath(install.display().to_string()))?,
    )?;
    copy_dir(&source, &install)?;
    std::fs::remove_dir_all(staging)?;
    Ok(NwjsInstallResult {
        version: version.to_owned(),
        flavor,
        target: target.to_owned(),
        install_dir: install,
    })
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
            std::fs::create_dir_all(output)?;
            continue;
        }
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = File::create(output)?;
        io::copy(&mut entry, &mut file)?;
    }
    Ok(())
}

fn single_root(path: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(path)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            !path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with('.'))
        })
        .collect::<Vec<_>>();
    (entries.len() == 1 && entries[0].is_dir()).then(|| entries[0].clone())
}

fn copy_dir(source: &Path, destination: &Path) -> Result<()> {
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir(&source_path, &destination_path)?;
        } else {
            std::fs::copy(source_path, destination_path)?;
        }
    }
    Ok(())
}
