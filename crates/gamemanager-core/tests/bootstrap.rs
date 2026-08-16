mod test_storage;

use std::sync::Arc;

use gamemanager_core::{BottlesCli, BottlesCliLocator, GameManagerCore, Result, UiPreferences};

struct NoBottles;

impl BottlesCliLocator for NoBottles {
    fn locate(&self) -> Option<BottlesCli> {
        None
    }
}

#[tokio::test]
async fn bootstrap_opens_a_v09_installation_in_one_snapshot() -> Result<()> {
    let installation = test_storage::create_existing_v09_layout().await?;
    let core =
        GameManagerCore::open_with_bottles_locator(installation.paths, Arc::new(NoBottles)).await?;
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

#[tokio::test]
async fn replacing_container_root_reopens_the_core_without_touching_user_data() -> Result<()> {
    let installation = test_storage::create_existing_v09_layout().await?;
    let core =
        GameManagerCore::open_with_bottles_locator(installation.paths.clone(), Arc::new(NoBottles))
            .await?;
    let replacement = installation.paths.data_dir().join("alternate-containers");

    let replacement_core = core.replace_container_root(&replacement).await?;

    assert_eq!(
        replacement_core.app_settings().await?.container_root,
        replacement.to_string_lossy()
    );
    assert_eq!(replacement_core.profiles().container_root(), replacement);
    assert_eq!(replacement_core.bootstrap().await?.games.len(), 1);
    Ok(())
}

#[tokio::test]
async fn removing_all_games_returns_the_number_of_deleted_records() -> Result<()> {
    let installation = test_storage::create_existing_v09_layout().await?;
    let core =
        GameManagerCore::open_with_bottles_locator(installation.paths, Arc::new(NoBottles)).await?;

    assert_eq!(core.remove_all_games().await?, 1);
    assert!(core.bootstrap().await?.games.is_empty());
    Ok(())
}
