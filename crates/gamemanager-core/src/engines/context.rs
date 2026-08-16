use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use glob::Pattern;

/// Filesystem queries available while evaluating an engine profile.
///
/// Every relative path is resolved from the directory currently being detected.
pub trait DetectionContext {
    fn file_exists(&self, relative_path: &str) -> bool;
    fn dir_exists(&self, relative_path: &str) -> bool;
    fn glob_match(&self, pattern: &str) -> bool;
    fn has_extension(&self, extension: &str) -> bool;
    fn has_native_executable(&self) -> bool;
    fn game_dir(&self) -> &Path;

    fn glob_match_recursive(&self, pattern: &str, max_depth: u32) -> bool {
        let mut directories = vec![(self.game_dir().to_path_buf(), 0)];

        while let Some((directory, depth)) = directories.pop() {
            let Ok(entries) = std::fs::read_dir(directory) else {
                continue;
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if simple_glob_match(pattern, &entry.file_name().to_string_lossy()) {
                    return true;
                }
                if depth < max_depth && path.is_dir() {
                    directories.push((path, depth + 1));
                }
            }
        }

        false
    }
}

/// A cached real-filesystem detection context.
pub struct FsDetectionContext {
    game_dir: PathBuf,
    direct_entries: OnceLock<Vec<PathBuf>>,
    recursive_entries: OnceLock<Vec<(String, u32)>>,
}

impl FsDetectionContext {
    pub fn new(game_dir: impl Into<PathBuf>) -> Self {
        Self {
            game_dir: game_dir.into(),
            direct_entries: OnceLock::new(),
            recursive_entries: OnceLock::new(),
        }
    }

    fn direct_entries(&self) -> &[PathBuf] {
        self.direct_entries.get_or_init(|| {
            let Ok(entries) = std::fs::read_dir(&self.game_dir) else {
                return Vec::new();
            };
            entries.flatten().map(|entry| entry.path()).collect()
        })
    }

    fn recursive_entries(&self) -> &[(String, u32)] {
        self.recursive_entries.get_or_init(|| {
            let mut entries = Vec::new();
            let mut directories = vec![(self.game_dir.clone(), 0)];

            while let Some((directory, depth)) = directories.pop() {
                let Ok(children) = std::fs::read_dir(directory) else {
                    continue;
                };

                for entry in children.flatten() {
                    let path = entry.path();
                    entries.push((entry.file_name().to_string_lossy().into_owned(), depth));
                    if depth < 3 && path.is_dir() {
                        directories.push((path, depth + 1));
                    }
                }
            }

            entries
        })
    }
}

impl DetectionContext for FsDetectionContext {
    fn file_exists(&self, relative_path: &str) -> bool {
        self.game_dir.join(relative_path).is_file()
    }

    fn dir_exists(&self, relative_path: &str) -> bool {
        self.game_dir.join(relative_path).is_dir()
    }

    fn glob_match(&self, pattern: &str) -> bool {
        self.direct_entries().iter().any(|path| {
            path.file_name()
                .is_some_and(|name| simple_glob_match(pattern, &name.to_string_lossy()))
        })
    }

    fn glob_match_recursive(&self, pattern: &str, max_depth: u32) -> bool {
        self.recursive_entries()
            .iter()
            .any(|(name, depth)| *depth <= max_depth && simple_glob_match(pattern, name))
    }

    fn has_extension(&self, extension: &str) -> bool {
        let extension = extension.trim_start_matches('.');
        self.direct_entries().iter().any(|path| {
            path.extension()
                .is_some_and(|value| value.to_string_lossy().eq_ignore_ascii_case(extension))
        })
    }

    fn has_native_executable(&self) -> bool {
        self.direct_entries()
            .iter()
            .any(|path| is_native_executable(path))
    }

    fn game_dir(&self) -> &Path {
        &self.game_dir
    }
}

pub(crate) fn simple_glob_match(pattern: &str, name: &str) -> bool {
    Pattern::new(&pattern.to_lowercase()).is_ok_and(|pattern| pattern.matches(&name.to_lowercase()))
}

#[cfg(unix)]
pub(crate) fn is_native_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    if !path.is_file() {
        return false;
    }

    if path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
        || !path
            .metadata()
            .is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
    {
        return false;
    }

    infer::get_from_path(path)
        .ok()
        .flatten()
        .is_some_and(|kind| matches!(kind.extension(), "elf" | "mach"))
}

#[cfg(not(unix))]
pub(crate) fn is_native_executable(_: &Path) -> bool {
    false
}
