use std::{
    collections::{BTreeSet, VecDeque},
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
    pub entry_candidates: Vec<ScanCandidate>,
    pub enabled_engine_ids: Vec<String>,
    pub scanned_directories: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanCandidate {
    pub game_root: PathBuf,
    pub entry_path: PathBuf,
    pub engine_id: String,
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

        let mut queue = VecDeque::from([(request.root.clone(), 0_u32)]);
        let mut candidates = Vec::new();
        let mut entry_candidates = Vec::new();
        let mut scanned_directories = 0;

        while let Some((directory, depth)) = queue.pop_front() {
            scanned_directories += 1;

            if depth > request.max_depth || is_nwjs_runtime_dir(&directory) {
                continue;
            }

            let context = FsDetectionContext::new(directory.clone());
            let detection = self
                .registry
                .detect_for_scan(&context)
                .filter(|detection| enabled.contains(&detection.engine_id));

            // A detected game is a scan boundary. The only exception is the
            // selected root itself when it is a collection containing multiple
            // directly-detectable game directories.
            let is_collection_root =
                depth == 0 && is_game_collection_root(&self.registry, &directory, &enabled);

            if let Some(detection) = detection
                && !is_collection_root
            {
                let game_root = resolve_nwjs_package_root(&detection.engine_id, &directory);
                if let Some(entry_path) = self
                    .registry
                    .resolve_entry(&detection.engine_id, &game_root)
                {
                    candidates.push(game_root.clone());
                    entry_candidates.push(ScanCandidate {
                        game_root,
                        entry_path,
                        engine_id: detection.engine_id,
                    });
                }

                // Do not scan inside a recognized game. Its subdirectories
                // contain assets, saves, runtimes, and plugins rather than
                // independent games; those can still be imported manually.
                continue;
            }

            if depth == request.max_depth {
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
        entry_candidates.sort_by(|left, right| left.game_root.cmp(&right.game_root));
        entry_candidates.dedup_by(|left, right| left.game_root == right.game_root);
        Ok(ScanPlan {
            candidates,
            entry_candidates,
            enabled_engine_ids,
            scanned_directories,
        })
    }
}

pub(crate) fn resolve_nwjs_package_root(engine_id: &str, path: &Path) -> PathBuf {
    if !engine_id.starts_with("rpgmaker") {
        return path.to_path_buf();
    }
    let mut current = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };
    loop {
        if current.join("package.json").is_file() {
            return current.to_path_buf();
        }
        let Some(parent) = current.parent() else {
            break;
        };
        if parent == current {
            break;
        }
        current = parent;
    }
    path.to_path_buf()
}

/// Returns whether the scan root is a collection directory rather than a
/// single game. This mirrors the v0.9.3 behavior: two directly detectable
/// children are enough to keep traversing the root.
fn is_game_collection_root(
    registry: &EngineRegistry,
    root: &Path,
    enabled: &BTreeSet<String>,
) -> bool {
    let Ok(entries) = std::fs::read_dir(root) else {
        return false;
    };

    let mut detected_children = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir()
            || path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with('.'))
            || is_nwjs_runtime_dir(&path)
        {
            continue;
        }

        let context = crate::FsDetectionContext::new(path);
        if registry
            .detect_for_scan(&context)
            .is_some_and(|detection| enabled.contains(&detection.engine_id))
        {
            detected_children += 1;
            if detected_children >= 2 {
                return true;
            }
        }
    }

    false
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
