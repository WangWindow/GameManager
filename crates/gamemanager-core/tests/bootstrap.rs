mod test_storage;

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use gamemanager_core::{
    BottleCatalog, BottleCatalogReport, BottlesCli, BottlesCliLocator, GameManagerCore, Result,
    UiPreferences,
};

struct NoBottles;

impl BottlesCliLocator for NoBottles {
    fn locate(&self) -> Option<BottlesCli> {
        None
    }
}

struct CountingBottles {
    calls: Arc<AtomicUsize>,
}

impl BottlesCliLocator for CountingBottles {
    fn locate(&self) -> Option<BottlesCli> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        None
    }
}

struct FixedCatalog(BottleCatalogReport);

impl BottleCatalog for FixedCatalog {
    fn list(&self) -> BottleCatalogReport {
        self.0.clone()
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
async fn bootstrap_does_not_probe_bottles_before_the_app_is_ready() -> Result<()> {
    let installation = test_storage::create_existing_v09_layout().await?;
    let calls = Arc::new(AtomicUsize::new(0));
    let core = GameManagerCore::open_with_bottles_locator(
        installation.paths,
        Arc::new(CountingBottles {
            calls: Arc::clone(&calls),
        }),
    )
    .await?;

    let snapshot = core.bootstrap().await?;

    assert_eq!(calls.load(Ordering::Relaxed), 0);
    let bottles = snapshot
        .integrations
        .iter()
        .find(|integration| integration.id == "bottles")
        .expect("Bottles integration");
    assert!(!bottles.available);
    Ok(())
}

#[tokio::test]
async fn bottles_status_uses_the_filesystem_catalog_without_probing_the_cli() -> Result<()> {
    let installation = test_storage::create_existing_v09_layout().await?;
    let calls = Arc::new(AtomicUsize::new(0));
    let core = GameManagerCore::open_with_bottles_services(
        installation.paths,
        Arc::new(CountingBottles {
            calls: Arc::clone(&calls),
        }),
        Arc::new(FixedCatalog(BottleCatalogReport {
            bottles: vec!["Games".to_owned(), "Wines".to_owned()],
            definitive: true,
            warnings: Vec::new(),
        })),
    )
    .await?;

    let status = core.bottles_status().await?;

    assert!(status.available);
    assert_eq!(status.bottles, ["Games", "Wines"]);
    assert!(status.bottles_error.is_none());
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    Ok(())
}

#[tokio::test]
async fn bottles_status_falls_back_to_the_cli_when_no_catalog_is_found() -> Result<()> {
    let installation = test_storage::create_existing_v09_layout().await?;
    let calls = Arc::new(AtomicUsize::new(0));
    let core = GameManagerCore::open_with_bottles_services(
        installation.paths,
        Arc::new(CountingBottles {
            calls: Arc::clone(&calls),
        }),
        Arc::new(FixedCatalog(BottleCatalogReport {
            bottles: Vec::new(),
            definitive: false,
            warnings: Vec::new(),
        })),
    )
    .await?;

    let status = core.bottles_status().await?;

    assert!(!status.available);
    assert!(status.bottles.is_empty());
    assert_eq!(calls.load(Ordering::Relaxed), 1);
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
