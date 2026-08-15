mod test_storage;

use gamemanager_core::{GameManagerCore, Result, UiPreferences};

#[tokio::test]
async fn bootstrap_opens_a_v09_installation_in_one_snapshot() -> Result<()> {
    let installation = test_storage::create_existing_v09_layout().await?;
    let core = GameManagerCore::open(installation.paths).await?;
    let snapshot = core.bootstrap().await?;

    assert_eq!(snapshot.games.len(), 1);
    assert_eq!(snapshot.games[0].id, "v09-demo-game");
    assert_eq!(snapshot.ui_preferences, UiPreferences::default());
    assert!(
        snapshot
            .engine_summaries
            .iter()
            .any(|engine| engine.id == "other")
    );
    Ok(())
}
