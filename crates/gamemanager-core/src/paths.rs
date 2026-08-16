use std::path::{Path, PathBuf};

use directories::BaseDirs;

use crate::{APP_ID, CoreError, Result};

/// Stable locations shared by v0.9 and the native desktop application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppPaths {
    data_dir: PathBuf,
}

impl AppPaths {
    /// Resolves the platform data directory and appends the stable application id.
    pub fn discover() -> Result<Self> {
        let base_dirs = BaseDirs::new().ok_or(CoreError::DataDirectoryUnavailable)?;
        Ok(Self::from_data_dir(base_dirs.data_dir().join(APP_ID)))
    }

    /// Builds paths from a known application data directory.
    ///
    /// This constructor is deliberately side-effect free, which also makes it
    /// suitable for compatibility fixtures and integration tests.
    pub fn from_data_dir(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn database(&self) -> PathBuf {
        self.data_dir.join("db/app.sqlite")
    }

    pub fn container_root(&self) -> PathBuf {
        self.data_dir.join("containers")
    }

    pub fn engine_dir(&self) -> PathBuf {
        self.data_dir.join("engines")
    }

    pub fn runtime_root(&self) -> PathBuf {
        self.data_dir.join("runtimes")
    }

    pub fn nwjs_runtime_root(&self) -> PathBuf {
        self.runtime_root().join("nwjs")
    }

    pub fn mkxpz_runtime_root(&self) -> PathBuf {
        self.runtime_root().join("mkxpz")
    }
}
