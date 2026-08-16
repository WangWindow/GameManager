use std::{collections::BTreeMap, fs};

use gamemanager_core::{
    AppPaths, Database, DetectionContext, EngineRegistry, FsDetectionContext, GameLibraryService,
    GameRecord, ImportRequest, ProfileStore, Runner, ScanPlanner, ScanRequest, UpdateGameRequest,
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
async fn list_sorts_by_last_played_then_created_at() -> gamemanager_core::Result<()> {
    let test = TestLibrary::new().await?;
    for game in [
        game_record("created-later", 30, None),
        game_record("played-later", 10, Some(40)),
        game_record("created-earlier", 20, None),
    ] {
        test.service.database().insert_game(&game).await?;
    }

    let games = test.service.list().await?;
    let ids = games
        .iter()
        .map(|game| game.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(ids, ["played-later", "created-later", "created-earlier"]);
    assert_eq!(games[0].created_at, 10);
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

#[test]
fn scan_plan_resolves_the_profile_entry_before_importing() -> gamemanager_core::Result<()> {
    let root = tempfile::tempdir()?;
    let game = root.path().join("vx-ace");
    fs::create_dir_all(game.join("Data"))?;
    fs::write(game.join("Game.ini"), "[Game]\nTitle=Test\n")?;
    fs::write(game.join("Game.exe"), [])?;
    fs::write(game.join("Data/Scripts.rvdata2"), [])?;

    let engines = tempfile::tempdir()?;
    EngineRegistry::synchronize_builtin_profiles(engines.path())?;
    let registry = EngineRegistry::load(engines.path(), &BTreeMap::new()).registry;
    let context = FsDetectionContext::new(&game);
    assert!(context.file_exists("Game.ini"));
    assert!(context.file_exists("Game.exe"));
    assert!(context.file_exists("Data/Scripts.rvdata2"));
    assert_eq!(
        registry
            .detect(&context)
            .map(|detection| detection.engine_id),
        Some("rpgmakervx".to_owned())
    );
    let plan = ScanPlanner::new(registry).plan(ScanRequest::new(root.path(), 2))?;

    assert_eq!(plan.candidates, vec![game.clone()]);
    assert_eq!(plan.entry_candidates.len(), 1);
    assert_eq!(plan.entry_candidates[0].engine_id, "rpgmakervx");
    assert_eq!(plan.entry_candidates[0].game_root, game);
    assert_eq!(
        plan.entry_candidates[0].entry_path,
        root.path().join("vx-ace/Game.exe")
    );
    Ok(())
}

#[test]
fn scan_plan_stops_at_a_detected_game_directory() -> gamemanager_core::Result<()> {
    let root = tempfile::tempdir()?;
    let game = root.path().join("game");
    fs::create_dir_all(game.join("assets/nested-game"))?;
    fs::write(game.join("Game.exe"), [])?;
    fs::write(game.join("assets/nested-game/Game.exe"), [])?;

    let engines = tempfile::tempdir()?;
    EngineRegistry::synchronize_builtin_profiles(engines.path())?;
    let registry = EngineRegistry::load(engines.path(), &BTreeMap::new()).registry;
    let plan = ScanPlanner::new(registry).plan(ScanRequest::new(root.path(), 3))?;

    assert_eq!(plan.candidates, vec![game]);
    assert_eq!(plan.scanned_directories, 2);
    Ok(())
}

#[test]
fn scan_plan_treats_a_root_with_multiple_games_as_a_collection() -> gamemanager_core::Result<()> {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("collection-launcher.exe"), [])?;
    for name in ["game-a", "game-b"] {
        let game = root.path().join(name);
        fs::create_dir_all(&game)?;
        fs::write(game.join("Game.exe"), [])?;
    }

    let engines = tempfile::tempdir()?;
    EngineRegistry::synchronize_builtin_profiles(engines.path())?;
    let registry = EngineRegistry::load(engines.path(), &BTreeMap::new()).registry;
    let plan = ScanPlanner::new(registry).plan(ScanRequest::new(root.path(), 2))?;

    assert_eq!(
        plan.candidates,
        vec![root.path().join("game-a"), root.path().join("game-b")]
    );
    assert!(!plan.candidates.contains(&root.path().to_path_buf()));
    Ok(())
}

#[test]
fn scan_plan_ignores_a_skip_scan_match_when_another_engine_can_scan() -> gamemanager_core::Result<()>
{
    let root = tempfile::tempdir()?;
    let game = root.path().join("game");
    fs::create_dir_all(&game)?;
    fs::write(game.join("Game.exe"), [])?;
    fs::write(game.join("readme.html"), "<p>notes</p>")?;

    let engines = tempfile::tempdir()?;
    EngineRegistry::synchronize_builtin_profiles(engines.path())?;
    let registry = EngineRegistry::load(engines.path(), &BTreeMap::new()).registry;
    let plan = ScanPlanner::new(registry).plan(ScanRequest::new(root.path(), 1))?;

    assert_eq!(plan.candidates, vec![game.clone()]);
    assert_eq!(plan.entry_candidates[0].engine_id, "other");
    assert_eq!(plan.entry_candidates[0].entry_path, game.join("Game.exe"));
    Ok(())
}

#[cfg(unix)]
#[test]
fn entry_resolution_does_not_treat_executable_mount_files_as_native() -> gamemanager_core::Result<()>
{
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::tempdir()?;
    for name in [
        ".nomedia",
        "GameAssembly.dll",
        "MiSideFull.exe",
        "UnityCrashHandler64.exe",
        "UnityPlayer.dll",
    ] {
        let path = root.path().join(name);
        fs::write(&path, [])?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
    }
    fs::create_dir(root.path().join("MiSideFull_Data"))?;

    let engines = tempfile::tempdir()?;
    EngineRegistry::synchronize_builtin_profiles(engines.path())?;
    let registry = EngineRegistry::load(engines.path(), &BTreeMap::new()).registry;

    assert_eq!(
        registry.resolve_entry("unity", root.path()),
        Some(root.path().join("MiSideFull.exe"))
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn entry_resolution_uses_a_real_native_binary_for_native_patterns() -> gamemanager_core::Result<()>
{
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::tempdir()?;
    fs::write(root.path().join("Phoenixes.pck"), [])?;
    let native = root.path().join("Phoenixes.x86_64");
    fs::copy(std::env::current_exe()?, &native)?;
    fs::set_permissions(&native, fs::Permissions::from_mode(0o755))?;
    let windows = root.path().join("Phoenixes.exe");
    fs::write(&windows, [])?;
    fs::set_permissions(&windows, fs::Permissions::from_mode(0o755))?;

    let engines = tempfile::tempdir()?;
    EngineRegistry::synchronize_builtin_profiles(engines.path())?;
    let registry = EngineRegistry::load(engines.path(), &BTreeMap::new()).registry;

    assert_eq!(registry.resolve_entry("godot", root.path()), Some(native));
    Ok(())
}

#[test]
fn other_entry_resolution_skips_common_helper_executables() -> gamemanager_core::Result<()> {
    let engines = tempfile::tempdir()?;
    EngineRegistry::synchronize_builtin_profiles(engines.path())?;
    let registry = EngineRegistry::load(engines.path(), &BTreeMap::new()).registry;

    let alice = tempfile::tempdir()?;
    for name in ["OpenSaveFolder.exe", "ResetConfig.exe", "dohnadohna.exe"] {
        fs::write(alice.path().join(name), [])?;
    }
    assert_eq!(
        registry.resolve_entry("other", alice.path()),
        Some(alice.path().join("dohnadohna.exe"))
    );

    let kirikiri = tempfile::tempdir()?;
    for name in [
        "savedata_location_changer.exe",
        "youmuin.exe",
        "youmuin@config.exe",
        "youmuin_oldsystem.exe",
        "youmuin_setup.exe",
    ] {
        fs::write(kirikiri.path().join(name), [])?;
    }
    assert_eq!(
        registry.resolve_entry("other", kirikiri.path()),
        Some(kirikiri.path().join("youmuin.exe"))
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

fn game_record(id: &str, created_at: i64, last_played_at: Option<i64>) -> GameRecord {
    GameRecord {
        id: id.to_owned(),
        profile_key: id.to_owned(),
        title: id.to_owned(),
        engine_type: "other".to_owned(),
        game_path: format!("/games/{id}"),
        normalized_path: format!("/games/{id}"),
        game_type: "other".to_owned(),
        detection_confidence: 0,
        runtime_version: None,
        cover_path: None,
        play_count: 0,
        metadata_json: None,
        created_at,
        last_played_at,
        updated_at: created_at,
    }
}
