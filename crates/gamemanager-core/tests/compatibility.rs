mod test_storage;

use gamemanager_core::{CoreError, Database, SETTING_CONTAINER_ROOT};

#[tokio::test]
async fn opens_v09_database_and_profile_without_reimporting() -> Result<(), CoreError> {
    let installation = test_storage::create_existing_v09_layout().await?;
    let database = Database::open(&installation.paths).await?;
    let games = database.games().await?;

    assert_eq!(games.len(), 1);
    assert_eq!(games[0].id, "v09-demo-game");
    assert_eq!(database.engines().await?[0].id, "v09-nwjs");
    let expected_container_root = installation
        .paths
        .container_root()
        .to_string_lossy()
        .to_string();
    assert_eq!(
        database.setting(SETTING_CONTAINER_ROOT).await?.as_deref(),
        Some(expected_container_root.as_str())
    );
    assert!(
        installation
            .paths
            .container_root()
            .join("profiles/v09-demo-game/settings.toml")
            .is_file()
    );

    Ok(())
}

#[tokio::test]
async fn v09_settings_write_back_to_the_same_database() -> Result<(), CoreError> {
    let installation = test_storage::create_existing_v09_layout().await?;
    let database = Database::open(&installation.paths).await?;

    database.set_setting("ui.theme_mode", "dark").await?;

    assert_eq!(
        database.setting("ui.theme_mode").await?.as_deref(),
        Some("dark")
    );

    let mut game = database.game("v09-demo-game").await?.expect("seeded game");
    game.title = "Updated title".to_owned();
    game.play_count = 7;
    database.update_game(&game).await?;
    assert_eq!(
        database
            .game("v09-demo-game")
            .await?
            .expect("updated game")
            .title,
        "Updated title"
    );
    database.delete_game("v09-demo-game").await?;
    assert!(database.game("v09-demo-game").await?.is_none());

    Ok(())
}
