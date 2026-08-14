use std::{fs, io::Cursor, path::PathBuf, sync::Arc};

use gamemanager_core::{
    CoverResolver, GameConfig, IconAsset, IconSource, ProfileStore, Result, Runner,
};

#[test]
fn profile_layout_matches_v09_names() {
    let profiles = ProfileStore::new(PathBuf::from("/tmp/containers"));

    assert_eq!(
        profiles.config_path("abc"),
        PathBuf::from("/tmp/containers/profiles/abc/settings.toml")
    );
    assert_eq!(
        profiles.user_data_dir("abc"),
        PathBuf::from("/tmp/containers/profiles/abc/User Data")
    );
    assert_eq!(
        profiles.crash_dir("abc"),
        PathBuf::from("/tmp/containers/profiles/abc/Crash Reports")
    );
}

#[test]
fn existing_settings_toml_round_trips_without_field_loss() -> Result<()> {
    let root = tempfile::tempdir()?;
    let profiles = ProfileStore::new(root.path());
    let config = GameConfig {
        engine_type: "html".to_owned(),
        entry_path: "index.html".to_owned(),
        runtime_version: Some("0.84.0".to_owned()),
        runner: Runner::Nwjs,
        args: vec!["--enable-webgl".to_owned()],
        sandbox_home: true,
        bottle_name: None,
        cover_file: Some("cover.png".to_owned()),
    };
    profiles.save("v09-demo-game", &config)?;

    let mut loaded = profiles.load("v09-demo-game")?;
    loaded.sandbox_home = false;
    profiles.save("v09-demo-game", &loaded)?;

    let saved = profiles.load("v09-demo-game")?;
    assert!(!saved.sandbox_home);
    assert_eq!(saved.runtime_version, Some("0.84.0".to_owned()));
    assert_eq!(saved.runner, Runner::Nwjs);
    assert_eq!(saved.args, ["--enable-webgl"]);
    Ok(())
}

#[test]
fn extracted_icon_is_preferred_before_sidecar_and_directory_images() -> Result<()> {
    let root = tempfile::tempdir()?;
    let game_root = root.path().join("game");
    let profile_root = root.path().join("containers");
    fs::create_dir_all(game_root.join("icons"))?;
    let executable = game_root.join("Game.exe");
    fs::write(&executable, [])?;
    fs::write(game_root.join("Game.png"), b"sidecar")?;
    fs::write(game_root.join("icons/icon.png"), b"directory")?;

    let profiles = ProfileStore::new(profile_root);
    profiles.save(
        "cover-test",
        &GameConfig {
            engine_type: "other".to_owned(),
            entry_path: executable.to_string_lossy().into_owned(),
            ..GameConfig::default()
        },
    )?;
    let resolver = CoverResolver::with_icon_source(profiles, Arc::new(PngIconSource));
    let result = resolver.refresh(&game_root, Some(&executable), "cover-test")?;

    assert_eq!(
        result.unwrap().file_name().and_then(|name| name.to_str()),
        Some("cover.png")
    );
    assert_eq!(
        resolver
            .profiles()
            .load("cover-test")?
            .cover_file
            .as_deref(),
        Some("cover.png")
    );
    Ok(())
}

struct PngIconSource;

impl IconSource for PngIconSource {
    fn extract(&self, _: &std::path::Path) -> Result<Option<IconAsset>> {
        let image = image::RgbaImage::from_pixel(1, 1, image::Rgba([16, 32, 48, 255]));
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgba8(image)
            .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
            .expect("encode generated PNG");
        Ok(Some(IconAsset::Png(bytes)))
    }
}
