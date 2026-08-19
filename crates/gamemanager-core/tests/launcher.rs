use std::{collections::BTreeMap, ffi::OsStr, fs, os::unix::fs::PermissionsExt, path::PathBuf};

use gamemanager_core::{
    AppPaths, BottlesCli, EngineRegistry, GameConfig, GameManagerCore, GameRecord, ImportRequest,
    Launcher, Runner,
};
use tempfile::TempDir;

#[tokio::test]
async fn core_launch_updates_play_history_after_spawning() -> gamemanager_core::Result<()> {
    let root = tempfile::tempdir()?;
    let paths = AppPaths::from_data_dir(root.path().join("data"));
    let core = GameManagerCore::open(paths).await?;
    let game_dir = root.path().join("game");
    let entry = game_dir.join("launch-game");
    fs::create_dir_all(&game_dir)?;
    fs::write(&entry, "#!/bin/sh\nexit 0\n")?;
    fs::set_permissions(&entry, fs::Permissions::from_mode(0o755))?;

    let imported = core.import_game(ImportRequest::from_entry(&entry)).await?;
    let mut config = core.game_config(&imported.profile_key).await?;
    config.runner = Runner::Native;
    core.profiles().save(&imported.profile_key, &config)?;

    let launched = core.launch_game(&imported.id).await?;

    assert_eq!(launched.play_count, 1);
    assert!(launched.last_played_at.is_some());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    Ok(())
}

#[tokio::test]
async fn importing_bottles_games_snapshots_the_current_default_bottle()
-> gamemanager_core::Result<()> {
    let root = tempfile::tempdir()?;
    let core = GameManagerCore::open(AppPaths::from_data_dir(root.path().join("data"))).await?;
    core.set_default_bottle(Some("Games")).await?;

    let first_entry = root.path().join("first/Game.exe");
    fs::create_dir_all(first_entry.parent().expect("entry parent"))?;
    fs::write(&first_entry, [])?;
    let first = core
        .import_game(ImportRequest::from_entry(&first_entry).with_engine("rpgmakervx"))
        .await?;

    let first_config = core.game_config(&first.profile_key).await?;
    assert_eq!(first_config.runner, Runner::Bottles);
    assert_eq!(first_config.bottle_name.as_deref(), Some("Games"));

    core.set_default_bottle(Some("Testing")).await?;
    let unchanged = core.game_config(&first.profile_key).await?;
    assert_eq!(unchanged.bottle_name.as_deref(), Some("Games"));
    Ok(())
}

#[test]
fn mkxpz_plan_keeps_profile_config_and_sandboxes_home() -> gamemanager_core::Result<()> {
    let fixture = Fixture::new()?;
    let plan = fixture
        .launcher
        .plan(&fixture.vx_game, &fixture.mkxpz_config())?;

    assert_eq!(plan.current_dir, fixture.profile_dir().join("mkxpz"));
    assert_eq!(
        PathBuf::from(plan.env.get(OsStr::new("SRCDIR")).expect("SRCDIR")),
        fixture.profile_dir().join("mkxpz")
    );
    assert_eq!(
        PathBuf::from(plan.env.get(OsStr::new("HOME")).expect("HOME")),
        fixture.profile_dir().join("User Data")
    );
    assert!(fixture.profile_dir().join("mkxpz/mkxp.json").is_file());
    Ok(())
}

#[test]
fn bottles_plan_does_not_override_home() -> gamemanager_core::Result<()> {
    let fixture = Fixture::new()?;
    let mut config = fixture.bottles_config();
    config.sandbox_home = true;

    let plan = fixture.launcher.plan(&fixture.bottles_game, &config)?;

    assert!(plan.env.is_empty(), "Bottles plan must not set HOME");
    Ok(())
}

#[test]
fn nwjs_plan_uses_user_and_crash_directories() -> gamemanager_core::Result<()> {
    let fixture = Fixture::new()?;
    let plan = fixture
        .launcher
        .plan(&fixture.html_game, &fixture.nwjs_config())?;
    assert!(
        plan.args
            .iter()
            .any(|arg| arg.to_string_lossy().starts_with("--user-data-dir="))
    );
    assert!(
        plan.args
            .iter()
            .any(|arg| arg.to_string_lossy().starts_with("--crash-dumps-dir="))
    );
    assert!(
        plan.args
            .iter()
            .any(|arg| arg.to_string_lossy().starts_with("--url=file://"))
    );
    Ok(())
}

#[test]
fn nwjs_plan_accepts_a_package_directory_and_profile_args() -> gamemanager_core::Result<()> {
    let fixture = Fixture::new()?;
    fs::write(
        fixture.html_game_root().join("package.json"),
        br#"{"main":"index.html"}"#,
    )?;
    let mut config = fixture.nwjs_config();
    config.entry_path = ".".to_owned();
    config.args = vec!["--enable-webgl".to_owned()];

    let plan = fixture.launcher.plan(&fixture.html_game, &config)?;

    assert!(plan.args.iter().any(|arg| arg == "--enable-webgl"));
    assert_eq!(
        plan.args.last().map(PathBuf::from),
        Some(fixture.html_game_root())
    );
    Ok(())
}

#[test]
fn runner_matrix_is_complete() -> gamemanager_core::Result<()> {
    let fixture = Fixture::new()?;
    assert_eq!(
        fixture
            .launcher
            .plan(&fixture.native_game, &fixture.native_config())?
            .runner(),
        Runner::Native
    );
    assert_eq!(
        fixture
            .launcher
            .plan(&fixture.bottles_game, &fixture.bottles_config())?
            .runner(),
        Runner::Bottles
    );
    assert_eq!(
        fixture
            .launcher
            .plan(&fixture.html_game, &fixture.nwjs_config())?
            .runner(),
        Runner::Nwjs
    );
    assert_eq!(
        fixture
            .launcher
            .plan(&fixture.vx_game, &fixture.mkxpz_config())?
            .runner(),
        Runner::Mkxpz
    );
    assert_eq!(
        fixture
            .launcher
            .plan(&fixture.external_game, &fixture.external_config())?
            .runner(),
        Runner::External
    );
    Ok(())
}

struct Fixture {
    root: TempDir,
    launcher: Launcher,
    native_game: GameRecord,
    bottles_game: GameRecord,
    html_game: GameRecord,
    vx_game: GameRecord,
    external_game: GameRecord,
}

impl Fixture {
    fn new() -> gamemanager_core::Result<Self> {
        let root = tempfile::tempdir()?;
        let game_root = root.path().join("games");
        fs::create_dir_all(&game_root)?;
        let native_dir = game_root.join("native");
        let bottles_dir = game_root.join("bottles");
        let html_dir = game_root.join("html");
        let vx_dir = game_root.join("vx");
        let external_dir = game_root.join("external");
        for dir in [&native_dir, &bottles_dir, &html_dir, &vx_dir, &external_dir] {
            fs::create_dir_all(dir)?;
        }
        let native_entry = native_dir.join("game");
        let bottles_entry = bottles_dir.join("game.exe");
        let html_entry = html_dir.join("index.html");
        let vx_entry = vx_dir.join("Game.exe");
        let external_entry = external_dir.join("game.exe");
        for path in [
            &native_entry,
            &bottles_entry,
            &html_entry,
            &vx_entry,
            &external_entry,
        ] {
            fs::write(path, [])?;
        }
        fs::set_permissions(&native_entry, fs::Permissions::from_mode(0o755))?;
        fs::create_dir_all(root.path().join("runtimes/nwjs"))?;
        let nw = root.path().join("runtimes/nwjs/nw");
        fs::write(&nw, [])?;
        fs::set_permissions(&nw, fs::Permissions::from_mode(0o755))?;
        let mkxp = root.path().join("runtimes/mkxpz/current/mkxp-z");
        fs::create_dir_all(mkxp.parent().expect("mkxp parent"))?;
        fs::write(&mkxp, [])?;
        fs::set_permissions(&mkxp, fs::Permissions::from_mode(0o755))?;

        let engines = root.path().join("engines");
        EngineRegistry::synchronize_builtin_profiles(&engines)?;
        fs::write(
            engines.join("external.toml"),
            r#"
[meta]
id = "external"
name = "External"
category = "other"

[detection]
min_score = 0
[[detection.required]]
type = "file_exists"
path = "game.exe"

[launch]
strategy = "external"
program = "wrapper"
args_template = "--game {exe}"
"#,
        )?;
        let registry = EngineRegistry::load(&engines, &BTreeMap::new()).registry;
        let bottles = BottlesCli::new("bottles-cli");
        let launcher = Launcher::new(root.path().join("containers"))
            .with_registry(registry)
            .with_bottles_cli(bottles)
            .with_nwjs_runtime(root.path().join("runtimes/nwjs"))
            .with_mkxpz_executable(mkxp);

        let make_game = |id: &str, path: &PathBuf, engine: &str| GameRecord {
            id: id.to_owned(),
            profile_key: id.to_owned(),
            title: id.to_owned(),
            engine_type: engine.to_owned(),
            game_path: path.to_string_lossy().into_owned(),
            normalized_path: path.to_string_lossy().into_owned(),
            game_type: "other".to_owned(),
            detection_confidence: 100,
            runtime_version: None,
            cover_path: None,
            play_count: 0,
            metadata_json: None,
            created_at: 0,
            last_played_at: None,
            updated_at: 0,
        };
        Ok(Self {
            root,
            launcher,
            native_game: make_game("native", &native_dir, "other"),
            bottles_game: make_game("bottles", &bottles_dir, "other"),
            html_game: make_game("html", &html_dir, "html"),
            vx_game: make_game("vx", &vx_dir, "rpgmakervx"),
            external_game: make_game("external", &external_dir, "external"),
        })
    }

    fn profile_dir(&self) -> PathBuf {
        self.root.path().join("containers/profiles").join("vx")
    }
    fn html_game_root(&self) -> PathBuf {
        self.root.path().join("games/html")
    }
    fn native_config(&self) -> GameConfig {
        config("other", "game", Runner::Native, false)
    }
    fn bottles_config(&self) -> GameConfig {
        config("other", "game.exe", Runner::Bottles, false).with_bottle()
    }
    fn nwjs_config(&self) -> GameConfig {
        config("html", "index.html", Runner::Nwjs, true)
    }
    fn mkxpz_config(&self) -> GameConfig {
        config("rpgmakervx", "Game.exe", Runner::Mkxpz, true)
    }
    fn external_config(&self) -> GameConfig {
        config("external", "game.exe", Runner::External, false)
    }
}

fn config(engine: &str, entry: &str, runner: Runner, sandbox: bool) -> GameConfig {
    GameConfig {
        engine_type: engine.to_owned(),
        entry_path: entry.to_owned(),
        runtime_version: None,
        runner,
        args: Vec::new(),
        sandbox_home: sandbox,
        bottle_name: None,
        cover_file: None,
    }
}

trait BottleConfig {
    fn with_bottle(self) -> GameConfig;
}
impl BottleConfig for GameConfig {
    fn with_bottle(mut self) -> GameConfig {
        self.bottle_name = Some("test".to_owned());
        self
    }
}
