use std::path::{Component, Path, PathBuf};

use crate::{CoreError, GameConfig, Result};

/// Per-game profile directories stored below the configured container root.
#[derive(Clone, Debug)]
pub struct ProfileStore {
    container_root: PathBuf,
}

impl ProfileStore {
    pub fn new(container_root: impl Into<PathBuf>) -> Self {
        Self {
            container_root: container_root.into(),
        }
    }

    pub fn container_root(&self) -> &Path {
        &self.container_root
    }

    pub fn profile_dir(&self, profile_key: &str) -> PathBuf {
        self.container_root.join("profiles").join(profile_key)
    }

    pub fn user_data_dir(&self, profile_key: &str) -> PathBuf {
        self.profile_dir(profile_key).join("User Data")
    }

    pub fn crash_dir(&self, profile_key: &str) -> PathBuf {
        self.profile_dir(profile_key).join("Crash Reports")
    }

    pub fn config_path(&self, profile_key: &str) -> PathBuf {
        self.profile_dir(profile_key).join("settings.toml")
    }

    pub fn cover_path(&self, profile_key: &str) -> PathBuf {
        self.cover_path_with_extension(profile_key, "png")
    }

    pub fn cover_path_with_extension(&self, profile_key: &str, extension: &str) -> PathBuf {
        self.profile_dir(profile_key)
            .join(format!("cover.{}", normalized_extension(extension)))
    }

    pub fn ensure(&self, profile_key: &str) -> Result<()> {
        validate_profile_key(profile_key)?;
        std::fs::create_dir_all(self.user_data_dir(profile_key))?;
        std::fs::create_dir_all(self.crash_dir(profile_key))?;
        Ok(())
    }

    pub fn load(&self, profile_key: &str) -> Result<GameConfig> {
        validate_profile_key(profile_key)?;
        let content = std::fs::read_to_string(self.config_path(profile_key))?;
        toml::from_str(&content).map_err(|error| CoreError::Configuration(error.to_string()))
    }

    pub fn save(&self, profile_key: &str, config: &GameConfig) -> Result<()> {
        self.ensure(profile_key)?;
        let content = toml::to_string_pretty(config)
            .map_err(|error| CoreError::Configuration(error.to_string()))?;
        std::fs::write(self.config_path(profile_key), content)?;
        Ok(())
    }
}

fn validate_profile_key(profile_key: &str) -> Result<()> {
    if profile_key.trim().is_empty()
        || Path::new(profile_key)
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(CoreError::InvalidPath(format!(
            "invalid profile key: {profile_key}"
        )));
    }
    Ok(())
}

fn normalized_extension(extension: &str) -> &str {
    extension.trim_start_matches('.').trim()
}
