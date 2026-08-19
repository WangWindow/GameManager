use std::{
    io::Cursor,
    path::{Path, PathBuf},
    sync::Arc,
};

use icoextract_rs::IconExtractor;

use crate::{CoreError, Result};

use super::{
    ProfileStore,
    files::{conventional_cover, image_in_icon_directories, is_image_file, sidecar_image},
};

pub enum IconAsset {
    Ico(Vec<u8>),
    Png(Vec<u8>),
}

/// Extracts an application icon from an executable without altering the game.
pub trait IconSource: Send + Sync {
    fn extract(&self, executable: &Path) -> Result<Option<IconAsset>>;
}

#[derive(Default)]
pub struct PeIconSource;

impl IconSource for PeIconSource {
    fn extract(&self, executable: &Path) -> Result<Option<IconAsset>> {
        if !executable
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
        {
            return Ok(None);
        }

        match IconExtractor::from_path(executable)
            .and_then(|extractor| extractor.icon_by_index(0))
            .and_then(|icon| icon.to_ico_bytes())
        {
            Ok(bytes) => Ok(Some(IconAsset::Ico(bytes))),
            Err(_) => Ok(None),
        }
    }
}

#[derive(Clone)]
pub struct CoverResolver {
    profiles: ProfileStore,
    icon_source: Arc<dyn IconSource>,
}

impl CoverResolver {
    pub fn new(profiles: ProfileStore) -> Self {
        Self::with_icon_source(profiles, Arc::new(PeIconSource))
    }

    pub fn with_icon_source(profiles: ProfileStore, icon_source: Arc<dyn IconSource>) -> Self {
        Self {
            profiles,
            icon_source,
        }
    }

    pub fn refresh(
        &self,
        game_root: &Path,
        entry: Option<&Path>,
        profile_key: &str,
    ) -> Result<Option<PathBuf>> {
        self.profiles.ensure(profile_key)?;

        if let Some(icon) = package_icon(game_root) {
            return self.copy_image(profile_key, &icon).map(Some);
        }

        if let Some(entry) = entry.filter(|path| path.is_file()) {
            if let Some(asset) = self.icon_source.extract(entry)? {
                return self.save_icon(profile_key, asset).map(Some);
            }
            if let Some(sidecar) = sidecar_image(entry) {
                return self.copy_image(profile_key, &sidecar).map(Some);
            }
        }

        if let Some(image) = image_in_icon_directories(game_root) {
            return self.copy_image(profile_key, &image).map(Some);
        }
        if let Some(image) = conventional_cover(game_root) {
            return self.copy_image(profile_key, &image).map(Some);
        }

        Ok(None)
    }

    pub fn profiles(&self) -> &ProfileStore {
        &self.profiles
    }

    /// Copies a user-selected cover into the managed profile directory.
    pub fn set_custom_cover(&self, profile_key: &str, source: &Path) -> Result<PathBuf> {
        if source
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
        {
            let asset = self.icon_source.extract(source)?.ok_or_else(|| {
                CoreError::Cover(format!(
                    "could not extract an icon from executable: {}",
                    source.display()
                ))
            })?;
            return self.save_icon(profile_key, asset);
        }
        self.copy_image(profile_key, source)
    }

    fn save_icon(&self, profile_key: &str, asset: IconAsset) -> Result<PathBuf> {
        let (bytes, extension) = match asset {
            IconAsset::Png(bytes) => (bytes, "png"),
            IconAsset::Ico(bytes) => ico_to_png(&bytes).unwrap_or((bytes, "ico")),
        };
        let target = self
            .profiles
            .cover_path_with_extension(profile_key, extension);
        std::fs::write(&target, bytes)?;
        self.update_cover_file(profile_key, &target)?;
        Ok(target)
    }

    fn copy_image(&self, profile_key: &str, source: &Path) -> Result<PathBuf> {
        if !is_image_file(source) {
            return Err(CoreError::Cover(format!(
                "unsupported cover image: {}",
                source.display()
            )));
        }
        let extension = source
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("png");
        let target = self
            .profiles
            .cover_path_with_extension(profile_key, extension);
        if source != target {
            std::fs::copy(source, &target)?;
        }
        self.update_cover_file(profile_key, &target)?;
        Ok(target)
    }

    fn update_cover_file(&self, profile_key: &str, cover_path: &Path) -> Result<()> {
        let config_path = self.profiles.config_path(profile_key);
        if !config_path.is_file() {
            return Ok(());
        }
        let mut config = self.profiles.load(profile_key)?;
        config.cover_file = cover_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(ToOwned::to_owned);
        self.profiles.save(profile_key, &config)
    }
}

fn package_icon(game_root: &Path) -> Option<PathBuf> {
    let package = std::fs::read_to_string(game_root.join("package.json")).ok()?;
    let manifest: serde_json::Value = serde_json::from_str(&package).ok()?;
    let icon = manifest.get("icon")?.as_str()?.trim();
    if icon.is_empty() {
        return None;
    }
    let path = game_root.join(icon);
    path.is_file().then_some(path)
}

fn ico_to_png(bytes: &[u8]) -> Option<(Vec<u8>, &'static str)> {
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Ico).ok()?;
    let mut png = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
        .ok()?;
    Some((png, "png"))
}
