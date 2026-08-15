use std::{collections::BTreeMap, fs};

use gamemanager_core::{
    AppPaths, Database, EngineRegistry, GameLibraryService, ImportRequest, ProfileStore, Runner,
    ScanPlanner, ScanRequest, UpdateGameRequest,
};
use tempfile::TempDir;

#[tokio::test]
async fn importing_an_entry_creates_profile_and_uses_detected_runner()
-> gamemanager_core::Result<()> {
    let test = TestLibrary::new().await?;
    let entry = test.root.path().join("html-game/index.html");
    fs::create_dir_all(entry.parent().expect("entry parent"))?;
    fs::write(&entry, "<!doctype html>")?;

    let summary = test
        .service
        .import_game(ImportRequest::from_entry(&entry))
        .await?;

    assert_eq!(summary.engine_type, "html");
    assert_eq!(summary.game_type, "other");
    assert_eq!(summary.profile_key, "index");
    assert_eq!(
        test.service.profiles().load(&summary.profile_key)?.runner,
        Runner::Nwjs
    );
    assert_eq!(
        test.service
            .profiles()
            .load(&summary.profile_key)?
            .entry_path,
        fs::canonicalize(&entry)?.to_string_lossy()
    );
    Ok(())
}

#[tokio::test]
async fn updating_title_migrates_profile_and_remove_clears_library() -> gamemanager_core::Result<()>
{
    let test = TestLibrary::new().await?;
    let entry = test.root.path().join("game/Game.html");
    fs::create_dir_all(entry.parent().expect("entry parent"))?;
    fs::write(&entry, "<html></html>")?;
    let imported = test
        .service
        .import_game(ImportRequest::from_entry(&entry))
        .await?;

    let updated = test
        .service
        .update_game(&imported.id, UpdateGameRequest::with_title("New Name"))
        .await?;
    assert_eq!(updated.title, "New Name");
    assert_eq!(updated.profile_key, "New_Name");
    assert!(
        !test
            .service
            .profiles()
            .config_path(&imported.profile_key)
            .exists()
    );
    assert!(test.service.profiles().config_path("New_Name").exists());

    test.service.remove_game(&updated.id).await?;
    assert!(test.service.list().await?.is_empty());
    Ok(())
}

#[test]
fn scan_plan_skips_runtime_directories_and_disabled_engines() -> gamemanager_core::Result<()> {
    let root = tempfile::tempdir()?;
    let runtime = root.path().join("nwjs-v0.84.0");
    fs::create_dir_all(runtime.join("locales"))?;
    for name in ["nw", "nw.pak", "icudtl.dat"] {
        fs::write(runtime.join(name), [])?;
    }
    let game = root.path().join("game");
    fs::create_dir_all(&game)?;
    fs::write(game.join("index.html"), "<html></html>")?;

    let engines = tempfile::tempdir()?;
    EngineRegistry::synchronize_builtin_profiles(engines.path())?;
    let mut registry = EngineRegistry::load(engines.path(), &BTreeMap::new()).registry;
    registry.set_enabled("html", false)?;
    let plan = ScanPlanner::new(registry).plan(ScanRequest::new(root.path(), 3))?;

    assert!(!plan.enabled_engine_ids.iter().any(|id| id == "html"));
    assert!(
        plan.candidates
            .iter()
            .all(|path| path.file_name().and_then(|name| name.to_str()) != Some("nwjs-v0.84.0"))
    );
    Ok(())
}

struct TestLibrary {
    root: TempDir,
    service: GameLibraryService,
}

impl TestLibrary {
    async fn new() -> gamemanager_core::Result<Self> {
        let root = tempfile::tempdir()?;
        let paths = AppPaths::from_data_dir(root.path().join("data"));
        let database = Database::open(&paths).await?;
        let engines = root.path().join("engines");
        EngineRegistry::synchronize_builtin_profiles(&engines)?;
        let registry = EngineRegistry::load(&engines, &BTreeMap::new()).registry;
        let profiles = ProfileStore::new(paths.container_root());
        Ok(Self {
            root,
            service: GameLibraryService::new(database, profiles, registry),
        })
    }
}
