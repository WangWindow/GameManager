mod mkxpz;
mod nwjs;

pub use mkxpz::{MkxpzInstallResult, ensure_compatibility_patch, import_mkxpz_archive};
pub use nwjs::{
    HttpClient, NwjsFlavor, NwjsInstallResult, NwjsStableInfo,
    build_download_url as build_nwjs_download_url, current_target as current_nwjs_target,
};

use std::{path::Path, sync::Arc};

use crate::{AppPaths, Operation, Result};

#[derive(Clone)]
pub struct RuntimeManager {
    paths: AppPaths,
    client: Arc<dyn HttpClient>,
}

impl RuntimeManager {
    pub fn new(paths: AppPaths) -> Self {
        Self {
            paths,
            client: Arc::new(nwjs::UnavailableHttpClient),
        }
    }

    pub fn with_http_client(mut self, client: Arc<dyn HttpClient>) -> Self {
        self.client = client;
        self
    }

    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    pub fn nwjs_stable_info(&self, version: impl Into<String>) -> Result<NwjsStableInfo> {
        let version = version.into();
        let target = nwjs::current_target()?;
        Ok(NwjsStableInfo {
            version: version.clone(),
            target: target.clone(),
            normal_url: nwjs::build_download_url(&version, NwjsFlavor::Normal, &target),
            sdk_url: nwjs::build_download_url(&version, NwjsFlavor::Sdk, &target),
        })
    }

    pub fn download_nwjs(
        &self,
        version: String,
        flavor: NwjsFlavor,
        target: String,
    ) -> Operation<NwjsInstallResult> {
        let client = self.client.clone();
        let paths = self.paths.clone();
        Operation::from_future("download", async move {
            nwjs::download_and_install(&paths, client, &version, flavor, &target).await
        })
    }

    pub fn update_engine(
        &self,
        version: String,
        flavor: NwjsFlavor,
        target: String,
    ) -> Operation<NwjsInstallResult> {
        self.download_nwjs(version, flavor, target)
    }

    pub fn import_mkxpz_archive(&self, archive_path: &Path) -> Result<MkxpzInstallResult> {
        import_mkxpz_archive(&self.paths, archive_path)
    }

    pub fn remove_engine(&self, path: &Path) -> Result<()> {
        if path.exists() {
            std::fs::remove_dir_all(path)?;
        }
        Ok(())
    }
}
