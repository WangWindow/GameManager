use std::fs;

use gamemanager_core::{
    AppPaths, Database, EngineRecord, GameRecord, Result, SETTING_CONTAINER_ROOT,
};
use tempfile::TempDir;

pub struct TestInstall {
    pub _root: TempDir,
    pub paths: AppPaths,
}

pub async fn create_existing_v09_layout() -> Result<TestInstall> {
    let root = tempfile::tempdir()?;
    let paths = AppPaths::from_data_dir(root.path().to_path_buf());
    let profile_dir = paths.container_root().join("profiles/v09-demo-game");
    fs::create_dir_all(&profile_dir)?;
    fs::write(
        profile_dir.join("settings.toml"),
        "engineType = \"html\"\nentryPath = \"index.html\"\nrunner = \"nwjs\"\nsandboxHome = true\n",
    )?;

    let database = Database::open(&paths).await?;
    database
        .insert_game(&GameRecord {
            id: "v09-demo-game".to_owned(),
            profile_key: "v09-demo-game".to_owned(),
            title: "v0.9 Compatibility Game".to_owned(),
            engine_type: "html".to_owned(),
            game_path: "/test/game".to_owned(),
            normalized_path: "/test/game".to_owned(),
            game_type: "unknown".to_owned(),
            detection_confidence: 0,
            runtime_version: Some("0.84.0".to_owned()),
            cover_path: None,
            play_count: 0,
            metadata_json: None,
            created_at: 1,
            last_played_at: None,
            updated_at: 1,
        })
        .await?;
    database
        .insert_engine(&EngineRecord {
            id: "v09-nwjs".to_owned(),
            name: "v0.9 NW.js".to_owned(),
            version: "0.84.0".to_owned(),
            engine_type: "html".to_owned(),
            engine_path: "runtimes/nwjs/0.84.0".to_owned(),
            installed_at: 1,
        })
        .await?;
    database
        .set_setting(SETTING_CONTAINER_ROOT, "/games/containers")
        .await?;
    drop(database);

    Ok(TestInstall { _root: root, paths })
}
