use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use crate::{EngineRegistry, FsDetectionContext, Result};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanRequest {
    pub root: PathBuf,
    pub max_depth: u32,
}

impl ScanRequest {
    pub fn new(root: impl Into<PathBuf>, max_depth: u32) -> Self {
        Self {
            root: root.into(),
            max_depth,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanPlan {
    pub candidates: Vec<PathBuf>,
    pub enabled_engine_ids: Vec<String>,
    pub scanned_directories: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanResult {
    pub candidates: Vec<PathBuf>,
    pub scanned_directories: u32,
}

#[derive(Clone, Debug)]
pub struct ScanPlanner {
    registry: EngineRegistry,
}

impl ScanPlanner {
    pub fn new(registry: EngineRegistry) -> Self {
        Self { registry }
    }

    pub fn plan(&self, request: ScanRequest) -> Result<ScanPlan> {
        if !request.root.is_dir() {
            return Err(crate::CoreError::InvalidPath(format!(
                "scan root is not a directory: {}",
                request.root.display()
            )));
        }

        let enabled_engine_ids = self
            .registry
            .summaries()
            .into_iter()
            .filter(|summary| summary.enabled && !self.registry.should_skip_scan(&summary.id))
            .map(|summary| summary.id)
            .collect::<Vec<_>>();
        let enabled = enabled_engine_ids.iter().cloned().collect::<BTreeSet<_>>();

        let mut queue = vec![(request.root.clone(), 0_u32)];
        let mut candidates = Vec::new();
        let mut scanned_directories = 0;

        while let Some((directory, depth)) = queue.pop() {
            if is_nwjs_runtime_dir(&directory) {
                continue;
            }
            scanned_directories += 1;

            let context = FsDetectionContext::new(directory.clone());
            if let Some(detection) = self.registry.detect(&context)
                && enabled.contains(&detection.engine_id)
            {
                candidates.push(directory.clone());
            }

            if depth >= request.max_depth {
                continue;
            }
            let Ok(entries) = std::fs::read_dir(&directory) else {
                continue;
            };
            let mut children = entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| {
                    path.is_dir()
                        && !path
                            .file_name()
                            .is_some_and(|name| name.to_string_lossy().starts_with('.'))
                })
                .collect::<Vec<_>>();
            children.sort();
            queue.extend(children.into_iter().map(|child| (child, depth + 1)));
        }

        candidates.sort();
        candidates.dedup();
        Ok(ScanPlan {
            candidates,
            enabled_engine_ids,
            scanned_directories,
        })
    }
}

pub(crate) fn is_nwjs_runtime_dir(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if name.starts_with("nwjs-") || name.starts_with("nwjs-sdk-") {
        return true;
    }

    let has_executable = ["nw", "nw.exe", "nwjs", "nwjs.exe"]
        .iter()
        .any(|name| path.join(name).is_file());
    let has_pak = path.join("nw.pak").is_file() || path.join("nw_100_percent.pak").is_file();
    has_executable && has_pak && path.join("icudtl.dat").is_file() && path.join("locales").is_dir()
}
