use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use directories::BaseDirs;
use serde_yaml::Value;
use tracing::warn;

const FLATPAK_DATA_DIR: &str = ".var/app/com.usebottles.bottles/data";

/// Result of a filesystem-based Bottles catalog scan.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BottleCatalogReport {
    /// Bottle display names, sorted and deduplicated.
    pub bottles: Vec<String>,
    /// `true` when at least one known Bottles data directory was readable, so
    /// an empty list is authoritative. `false` means the caller should fall
    /// back to the CLI because no known installation was found.
    pub definitive: bool,
    /// Non-fatal problems encountered while scanning, useful for logs.
    pub warnings: Vec<String>,
}

pub trait BottleCatalog: Send + Sync {
    fn list(&self) -> BottleCatalogReport;
}

/// Reads Bottles' on-disk configuration instead of spawning `bottles-cli`.
///
/// Classic Bottles stores each bottle as `<data>/bottles/<dir>/bottle.yml`.
/// Flatpak uses `~/.var/app/com.usebottles.bottles/data/bottles`, while native
/// installs use `$XDG_DATA_HOME/bottles`. Both are merged so a system can have
/// more than one installation.
#[derive(Clone, Debug, Default)]
pub struct SystemBottleCatalog {
    roots: Vec<PathBuf>,
}

impl SystemBottleCatalog {
    pub fn discover() -> Self {
        let mut roots = Vec::new();
        if let Some(dirs) = BaseDirs::new() {
            roots.push(dirs.home_dir().join(FLATPAK_DATA_DIR).join("bottles"));
            roots.push(dirs.data_dir().join("bottles"));
        }
        Self::from_roots(roots)
    }

    /// Builds a catalog from explicit Bottles base directories. Each root is
    /// expected to contain `bottles/` and optionally `data.yml`.
    pub fn from_roots(roots: impl IntoIterator<Item = PathBuf>) -> Self {
        let mut roots = roots.into_iter().collect::<Vec<_>>();
        roots.sort();
        roots.dedup();
        Self { roots }
    }

    fn scan_locations(&self) -> Vec<PathBuf> {
        let mut locations = Vec::new();
        for root in &self.roots {
            locations.push(root.join("bottles"));
            if let Some(custom) = custom_bottles_path(root) {
                locations.push(custom);
            }
        }
        locations.sort();
        locations.dedup();
        locations
    }
}

impl BottleCatalog for SystemBottleCatalog {
    fn list(&self) -> BottleCatalogReport {
        let mut names = BTreeSet::new();
        let mut warnings = Vec::new();
        let mut definitive = self.roots.iter().any(|root| root.is_dir());

        for location in self.scan_locations() {
            if !location.is_dir() {
                continue;
            }
            definitive = true;
            scan_bottles_dir(&location, &mut names, &mut warnings);
        }

        if definitive && names.is_empty() && !warnings.is_empty() {
            warn!(
                warnings = warnings.len(),
                "Bottles catalog was found but no bottle could be read"
            );
        }

        BottleCatalogReport {
            bottles: names.into_iter().collect(),
            definitive,
            warnings,
        }
    }
}

fn scan_bottles_dir(directory: &Path, names: &mut BTreeSet<String>, warnings: &mut Vec<String>) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) => {
            warnings.push(format!("could not read {}: {error}", directory.display()));
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        match read_bottle_name(&path) {
            Ok(Some(name)) => {
                names.insert(name);
            }
            Ok(None) => {}
            Err(error) => warnings.push(format!("{}: {error}", path.display())),
        }
    }
}

fn read_bottle_name(directory: &Path) -> Result<Option<String>, String> {
    let config = directory.join("bottle.yml");
    if config.is_file() {
        return read_config_name(&config);
    }

    let placeholder = directory.join("placeholder.yml");
    if placeholder.is_file() {
        let value = read_yaml(&placeholder)?;
        let target = value
            .get("Path")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .ok_or_else(|| "placeholder.yml has no Path".to_owned())?;
        let config = PathBuf::from(target).join("bottle.yml");
        if config.is_file() {
            return read_config_name(&config);
        }
    }

    Ok(None)
}

fn read_config_name(path: &Path) -> Result<Option<String>, String> {
    let value = read_yaml(path)?;
    let name = value
        .get("Name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned);

    if name.is_some() {
        return Ok(name);
    }

    Ok(path
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned))
}

fn read_yaml(path: &Path) -> Result<Value, String> {
    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_yaml::from_str(&raw).map_err(|error| error.to_string())
}

fn custom_bottles_path(root: &Path) -> Option<PathBuf> {
    let raw = fs::read_to_string(root.join("data.yml")).ok()?;
    let value: Value = serde_yaml::from_str(&raw).ok()?;
    let path = value
        .get("custom_bottles_path")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|path| !path.is_empty())?;
    let path = PathBuf::from(path);
    Some(if path.is_absolute() {
        path
    } else {
        root.join(path)
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::{BottleCatalog, SystemBottleCatalog};

    fn base(root: &TempDir) -> std::path::PathBuf {
        root.path().join("bottles")
    }

    fn write_bottle(bottles_dir: &std::path::Path, folder: &str, name: &str) {
        let directory = bottles_dir.join(folder);
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("bottle.yml"), format!("Name: {name}\n")).unwrap();
    }

    #[test]
    fn reads_names_from_standard_bottle_directories() {
        let root = tempfile::tempdir().unwrap();
        let bottles = base(&root).join("bottles");
        fs::create_dir_all(&bottles).unwrap();
        write_bottle(&bottles, "games-folder", "Games");
        write_bottle(&bottles, "wines", "Wines");

        let report = SystemBottleCatalog::from_roots([base(&root)]).list();

        assert!(report.definitive);
        assert_eq!(report.bottles, ["Games", "Wines"]);
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn follows_placeholder_to_a_custom_bottle_path() {
        let root = tempfile::tempdir().unwrap();
        let bottles = base(&root).join("bottles");
        let external = root.path().join("external/Games");
        fs::create_dir_all(&external).unwrap();
        fs::write(external.join("bottle.yml"), "Name: Games\n").unwrap();
        fs::create_dir_all(bottles.join("games-link")).unwrap();
        fs::write(
            bottles.join("games-link/placeholder.yml"),
            format!("Path: {}\n", external.display()),
        )
        .unwrap();

        let report = SystemBottleCatalog::from_roots([base(&root)]).list();

        assert!(report.definitive);
        assert_eq!(report.bottles, ["Games"]);
    }

    #[test]
    fn includes_custom_bottles_path_from_data_yml() {
        let root = tempfile::tempdir().unwrap();
        let bottles = base(&root).join("bottles");
        fs::create_dir_all(&bottles).unwrap();
        let custom = root.path().join("custom-bottles");
        fs::create_dir_all(&custom).unwrap();
        write_bottle(&custom, "custom", "Custom");
        fs::write(
            base(&root).join("data.yml"),
            format!("custom_bottles_path: {}\n", custom.display()),
        )
        .unwrap();

        let report = SystemBottleCatalog::from_roots([base(&root)]).list();

        assert_eq!(report.bottles, ["Custom"]);
    }

    #[test]
    fn missing_installation_is_not_definitive() {
        let root = tempfile::tempdir().unwrap();
        let report = SystemBottleCatalog::from_roots([root.path().join("does-not-exist")]).list();

        assert!(!report.definitive);
        assert!(report.bottles.is_empty());
    }

    #[test]
    fn empty_installation_is_definitive() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(base(&root)).unwrap();

        let report = SystemBottleCatalog::from_roots([base(&root)]).list();

        assert!(report.definitive);
        assert!(report.bottles.is_empty());
    }
}
