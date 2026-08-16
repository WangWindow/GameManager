use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::de::DeserializeOwned;
use tracing::{debug, error, info, warn};

use crate::{
    AppPaths, AppSettings, BottlesCli, BottlesCliLocator, CoreError, Database, EngineDetail,
    EngineRecord, EngineRegistry, EngineSummary, GameConfig, GameLibraryService, GameSummary,
    ImportRequest, Launcher, MkxpzInstallResult, NwjsInstallResult, Operation, ProfileStore,
    RegistryWarning, Result, RuntimeManager, SETTING_BOTTLES_DEFAULT, SETTING_BOTTLES_ENABLED,
    SETTING_CONTAINER_ROOT, SETTING_ENGINE_ENABLED, SETTING_UI_PREFERENCES, ScanPlanner,
    ScanRequest, ScanResult, SystemBottlesCliLocator, UiPreferences,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrationStatus {
    pub id: String,
    pub enabled: bool,
    pub available: bool,
    pub bottles: Vec<String>,
    pub default_bottle: Option<String>,
    pub bottles_error: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeStatus {
    pub id: String,
    pub name: String,
    pub version: String,
    pub engine_type: String,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct BootstrapSnapshot {
    pub games: Vec<GameSummary>,
    pub app_settings: AppSettings,
    pub ui_preferences: UiPreferences,
    pub engine_summaries: Vec<EngineSummary>,
    pub engine_details: Vec<EngineDetail>,
    pub engine_warnings: Vec<RegistryWarning>,
    pub integrations: Vec<IntegrationStatus>,
    pub runtimes: Vec<RuntimeStatus>,
}

pub struct GameManagerCore {
    paths: AppPaths,
    database: Database,
    profiles: ProfileStore,
    registry: EngineRegistry,
    registry_warnings: Vec<RegistryWarning>,
    library: GameLibraryService,
    runtime: RuntimeManager,
    container_root: PathBuf,
    bottles_locator: Arc<dyn BottlesCliLocator>,
}

impl GameManagerCore {
    pub async fn open(paths: AppPaths) -> Result<Self> {
        Self::open_with_bottles_locator(paths, Arc::new(SystemBottlesCliLocator)).await
    }

    pub async fn open_with_bottles_locator(
        paths: AppPaths,
        bottles_locator: Arc<dyn BottlesCliLocator>,
    ) -> Result<Self> {
        info!(
            data_dir = %paths.data_dir().display(),
            runtime_root = %paths.runtime_root().display(),
            "opening core"
        );
        fs::create_dir_all(paths.data_dir())?;
        fs::create_dir_all(paths.container_root())?;
        fs::create_dir_all(paths.engine_dir())?;
        fs::create_dir_all(paths.runtime_root())?;
        let database = Database::open(&paths).await?;
        let container_root = database
            .setting(SETTING_CONTAINER_ROOT)
            .await?
            .filter(|value| !value.trim().is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| paths.container_root());
        fs::create_dir_all(&container_root)?;

        EngineRegistry::synchronize_builtin_profiles(&paths.engine_dir())?;
        let enabled = load_setting::<BTreeMap<String, bool>>(&database, SETTING_ENGINE_ENABLED)
            .await?
            .unwrap_or_default();
        let report = EngineRegistry::load(&paths.engine_dir(), &enabled);
        if !report.warnings.is_empty() {
            warn!(
                count = report.warnings.len(),
                "engine registry loaded with warnings"
            );
        }
        let profiles = ProfileStore::new(&container_root);
        let library =
            GameLibraryService::new(database.clone(), profiles.clone(), report.registry.clone());
        let runtime = RuntimeManager::new(paths.clone());
        let core = Self {
            paths,
            database,
            profiles,
            registry: report.registry,
            registry_warnings: report.warnings,
            library,
            runtime,
            container_root,
            bottles_locator,
        };
        info!("core opened");
        Ok(core)
    }

    pub async fn bootstrap(&self) -> Result<BootstrapSnapshot> {
        debug!("bootstrapping application snapshot");
        let games = self.library.list().await?;
        let app_settings = self.app_settings().await?;
        let ui_preferences = self.ui_preferences().await?;
        let bottles_enabled = self
            .database
            .setting(SETTING_BOTTLES_ENABLED)
            .await?
            .is_some_and(|value| value == "true");
        let default_bottle = self.default_bottle().await?;
        let (available, bottles, bottles_error) = match self.locate_bottles_cli().await {
            Ok(Some(cli)) => match cli.list_bottles().await {
                Ok(bottles) => (true, bottles, None),
                Err(error) => (true, Vec::new(), Some(error.to_string())),
            },
            Ok(None) => (false, Vec::new(), None),
            Err(error) => (false, Vec::new(), Some(error.to_string())),
        };
        let runtimes = self
            .database
            .engines()
            .await?
            .into_iter()
            .map(runtime_status)
            .collect();
        let snapshot = BootstrapSnapshot {
            games,
            app_settings,
            ui_preferences,
            engine_summaries: self.registry.summaries(),
            engine_details: self.registry.details(),
            engine_warnings: self.registry_warnings.clone(),
            integrations: vec![IntegrationStatus {
                id: "bottles".to_owned(),
                enabled: bottles_enabled,
                available,
                bottles,
                default_bottle,
                bottles_error,
            }],
            runtimes,
        };
        info!(
            games = snapshot.games.len(),
            runtimes = snapshot.runtimes.len(),
            bottles_available = snapshot
                .integrations
                .iter()
                .any(|integration| integration.available),
            "application snapshot ready"
        );
        Ok(snapshot)
    }

    pub async fn import_game(&self, request: ImportRequest) -> Result<GameSummary> {
        info!(entry = %request.entry.path.display(), "importing game");
        match self.library.import_game(request).await {
            Ok(game) => {
                self.initialize_imported_bottle(&game).await?;
                info!(game_id = %game.id, title = %game.title, "game imported");
                Ok(game)
            }
            Err(error) => {
                error!(error = %error, "game import failed");
                Err(error)
            }
        }
    }

    async fn initialize_imported_bottle(&self, game: &GameSummary) -> Result<()> {
        initialize_imported_bottle(&self.database, &self.profiles, game).await
    }

    pub async fn game_config(&self, profile_key: &str) -> Result<GameConfig> {
        self.profiles.load(profile_key)
    }

    pub async fn save_game_settings(
        &self,
        game_id: &str,
        request: crate::UpdateGameRequest,
        config: &GameConfig,
    ) -> Result<GameSummary> {
        debug!(game_id, "saving game settings");
        let game = self.library.update_game(game_id, request).await?;
        self.profiles.save(&game.profile_key, config)?;
        Ok(game)
    }

    pub async fn launch_game(&self, game_id: &str) -> Result<GameSummary> {
        info!(game_id, "launch requested");
        let mut game = self
            .database
            .game(game_id)
            .await?
            .ok_or_else(|| CoreError::Database(format!("game not found: {game_id}")))?;
        let mut config = self.profiles.load(&game.profile_key)?;
        let uses_bottles = config.runner == crate::Runner::Bottles
            || (config.runner.is_legacy_auto()
                && !matches!(
                    self.registry
                        .profile(&game.engine_type)
                        .map(|profile| profile.launch.strategy.as_str()),
                    Some("native" | "nwjs" | "mkxpz" | "external")
                ));
        if uses_bottles
            && config
                .bottle_name
                .as_deref()
                .is_none_or(|name| name.trim().is_empty())
        {
            let Some(default_bottle) = self.default_bottle().await? else {
                return Err(CoreError::Configuration(
                    "Bottles runner requires a bottle".to_owned(),
                ));
            };
            config.bottle_name = Some(default_bottle);
            self.profiles.save(&game.profile_key, &config)?;
            debug!(game_id = %game.id, "resolved game bottle from global default");
        }
        let runtimes = self.database.engines().await?;
        let mut launcher = Launcher::new(&self.container_root).with_registry(self.registry.clone());
        if uses_bottles
            && self
                .database
                .setting(SETTING_BOTTLES_ENABLED)
                .await?
                .is_some_and(|value| value == "true")
            && let Some(cli) = self.bottles_locator.locate()
        {
            launcher = launcher.with_bottles_cli(cli);
        }
        if let Some(runtime) = select_runtime(&runtimes, "nwjs", config.runtime_version.as_deref())
        {
            launcher = launcher.with_nwjs_runtime(&runtime.engine_path);
        }
        if let Some(runtime) = select_runtime(&runtimes, "mkxpz", None) {
            launcher = launcher.with_mkxpz_executable(&runtime.engine_path);
        }

        let plan = launcher.plan(&game, &config)?;
        let runner = plan.runner();
        let launch = launcher.spawn(plan)?;
        info!(
            game_id = %game.id,
            title = %game.title,
            runner = ?runner,
            pid = launch.pid,
            "game process started"
        );
        game.last_played_at = Some(unix_timestamp());
        game.play_count = game.play_count.saturating_add(1);
        game.updated_at = unix_timestamp();
        self.database.update_game(&game).await?;
        Ok(GameSummary::from(&game))
    }

    pub async fn remove_game(&self, game_id: &str) -> Result<()> {
        info!(game_id, "removing game");
        let result = self.library.remove_game(game_id).await;
        if let Err(error) = &result {
            error!(game_id, error = %error, "game removal failed");
        }
        result
    }

    pub async fn remove_all_games(&self) -> Result<usize> {
        let games = self.library.list().await?;
        info!(count = games.len(), "removing all games");
        self.library.remove_all_games().await?;
        Ok(games.len())
    }

    pub async fn cleanup_unused_profiles(&self) -> Result<usize> {
        let live_profile_keys = self
            .library
            .list()
            .await?
            .into_iter()
            .map(|game| game.profile_key)
            .collect::<BTreeSet<_>>();
        let profiles = self.profiles.clone();
        let result =
            tokio::task::spawn_blocking(move || profiles.cleanup_unused(&live_profile_keys))
                .await
                .map_err(|error| {
                    CoreError::Configuration(format!(
                        "profile cleanup task could not complete: {error}"
                    ))
                })??;
        info!(removed = result, "unused profiles cleaned");
        Ok(result)
    }

    pub async fn set_custom_cover(
        &self,
        game_id: &str,
        source: impl AsRef<std::path::Path>,
    ) -> Result<GameSummary> {
        debug!(game_id, source = %source.as_ref().display(), "setting custom cover");
        self.library
            .set_custom_cover(game_id, source.as_ref())
            .await
    }

    pub async fn refresh_cover(&self, game_id: &str) -> Result<GameSummary> {
        debug!(game_id, "refreshing cover");
        self.library.refresh_cover(game_id).await
    }

    pub async fn set_bottles_enabled(&self, enabled: bool) -> Result<()> {
        info!(enabled, "updating Bottles integration");
        self.database
            .set_setting(
                SETTING_BOTTLES_ENABLED,
                if enabled { "true" } else { "false" },
            )
            .await
    }

    pub async fn default_bottle(&self) -> Result<Option<String>> {
        Ok(self
            .database
            .setting(SETTING_BOTTLES_DEFAULT)
            .await?
            .filter(|value| !value.trim().is_empty()))
    }

    pub async fn set_default_bottle(&self, bottle: Option<&str>) -> Result<()> {
        let value = bottle
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("");
        info!(bottle = %value, "updating default Bottles bottle");
        self.database
            .set_setting(SETTING_BOTTLES_DEFAULT, value)
            .await
    }

    pub async fn list_bottles(&self) -> Result<Vec<String>> {
        debug!("listing Bottles");
        let cli = self.locate_bottles_cli().await?.ok_or_else(|| {
            CoreError::Configuration("Bottles is not available on this system".to_owned())
        })?;
        let bottles = cli.list_bottles().await?;
        info!(count = bottles.len(), "Bottles listed");
        Ok(bottles)
    }

    pub fn scan(&self, request: ScanRequest) -> Operation<ScanResult> {
        info!(
            root = %request.root.display(),
            max_depth = request.max_depth,
            "scan requested"
        );
        let planner = ScanPlanner::new(self.registry.clone());
        let library = self.library.clone();
        let database = self.database.clone();
        let profiles = self.profiles.clone();
        Operation::from_future("扫描游戏", async move {
            let plan = planner.plan(request)?;
            info!(
                scanned_directories = plan.scanned_directories,
                candidates = plan.entry_candidates.len(),
                "scan plan ready"
            );
            let existing = library.list().await?;
            let mut imported = Vec::new();
            for candidate in &plan.entry_candidates {
                let canonical = std::fs::canonicalize(&candidate.game_root)
                    .unwrap_or_else(|_| candidate.game_root.clone());
                if existing
                    .iter()
                    .any(|game| std::path::Path::new(&game.game_path) == canonical)
                {
                    debug!(path = %candidate.game_root.display(), "scan candidate already imported");
                    continue;
                }
                let imported_game = match library
                    .import_game(
                        ImportRequest::from_entry(&candidate.entry_path)
                            .with_engine(&candidate.engine_id),
                    )
                    .await
                {
                    Ok(game) => game,
                    Err(error) => {
                        error!(
                            path = %candidate.entry_path.display(),
                            error = %error,
                            "scan candidate import failed"
                        );
                        return Err(error);
                    }
                };
                initialize_imported_bottle(&database, &profiles, &imported_game).await?;
                info!(
                    path = %candidate.entry_path.display(),
                    engine = %candidate.engine_id,
                    "scan candidate imported"
                );
                imported.push(candidate.game_root.clone());
            }
            info!(imported = imported.len(), "scan completed");
            Ok(ScanResult {
                candidates: imported,
                scanned_directories: plan.scanned_directories,
            })
        })
    }

    pub async fn app_settings(&self) -> Result<AppSettings> {
        let container_root = self
            .database
            .setting(SETTING_CONTAINER_ROOT)
            .await?
            .unwrap_or_else(|| self.container_root.to_string_lossy().into_owned());
        Ok(AppSettings { container_root })
    }

    pub async fn replace_container_root(&self, path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        info!(path = %path.display(), "replacing container root");
        if path.as_os_str().is_empty() {
            return Err(CoreError::InvalidPath("container root is empty".to_owned()));
        }
        fs::create_dir_all(&path)?;
        self.database
            .set_setting(SETTING_CONTAINER_ROOT, &path.to_string_lossy())
            .await?;
        Self::open_with_bottles_locator(self.paths.clone(), Arc::clone(&self.bottles_locator)).await
    }

    pub async fn ui_preferences(&self) -> Result<UiPreferences> {
        Ok(load_setting(&self.database, SETTING_UI_PREFERENCES)
            .await?
            .unwrap_or_default())
    }

    /// Reads the persisted UI preferences before the desktop window is
    /// created. This is intentionally kept separate from `open` so the
    /// launcher can honor a saved window backend without creating an Iced
    /// window first.
    pub async fn read_ui_preferences(paths: &AppPaths) -> Result<UiPreferences> {
        let database = Database::open(paths).await?;
        Ok(load_setting(&database, SETTING_UI_PREFERENCES)
            .await?
            .unwrap_or_default())
    }

    pub async fn save_ui_preferences(&self, preferences: &UiPreferences) -> Result<()> {
        debug!(
            show_status_bar = preferences.show_status_bar,
            "saving UI preferences"
        );
        let value = toml::to_string(preferences)
            .map_err(|error| CoreError::Configuration(error.to_string()))?;
        self.database
            .set_setting(SETTING_UI_PREFERENCES, &value)
            .await
    }

    pub async fn set_engine_enabled(&self, id: &str, enabled: bool) -> Result<()> {
        info!(engine = id, enabled, "updating engine enabled state");
        let mut values =
            load_setting::<BTreeMap<String, bool>>(&self.database, SETTING_ENGINE_ENABLED)
                .await?
                .unwrap_or_default();
        values.insert(id.to_owned(), enabled);
        let value = toml::to_string(&values)
            .map_err(|error| CoreError::Configuration(error.to_string()))?;
        self.database
            .set_setting(SETTING_ENGINE_ENABLED, &value)
            .await
    }

    pub async fn register_mkxpz_runtime(&self, install: &MkxpzInstallResult) -> Result<()> {
        info!(version = %install.version, path = %install.executable_path.display(), "registering mkxp-z runtime");
        self.database
            .insert_engine(&EngineRecord {
                id: uuid::Uuid::new_v4().to_string(),
                name: "mkxp-z".to_owned(),
                version: install.version.clone(),
                engine_type: "mkxpz".to_owned(),
                engine_path: install.executable_path.to_string_lossy().into_owned(),
                installed_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
            })
            .await
    }

    pub async fn register_nwjs_runtime(&self, install: &NwjsInstallResult) -> Result<()> {
        info!(version = %install.version, path = %install.install_dir.display(), "registering NW.js runtime");
        self.database
            .insert_engine(&EngineRecord {
                id: uuid::Uuid::new_v4().to_string(),
                name: "NW.js".to_owned(),
                version: install.version.clone(),
                engine_type: "nwjs".to_owned(),
                engine_path: install.install_dir.to_string_lossy().into_owned(),
                installed_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
            })
            .await
    }

    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }
    pub fn database(&self) -> &Database {
        &self.database
    }
    pub fn library(&self) -> &GameLibraryService {
        &self.library
    }
    pub fn profiles(&self) -> &ProfileStore {
        &self.profiles
    }
    pub fn registry(&self) -> &EngineRegistry {
        &self.registry
    }
    pub fn runtime_manager(&self) -> &RuntimeManager {
        &self.runtime
    }

    async fn locate_bottles_cli(&self) -> Result<Option<BottlesCli>> {
        let locator = Arc::clone(&self.bottles_locator);
        tokio::task::spawn_blocking(move || locator.locate())
            .await
            .map_err(|error| {
                CoreError::Configuration(format!(
                    "Bottles detection task could not complete: {error}"
                ))
            })
    }
}

async fn load_setting<T: DeserializeOwned>(database: &Database, key: &str) -> Result<Option<T>> {
    let Some(value) = database.setting(key).await? else {
        return Ok(None);
    };
    toml::from_str(&value)
        .map(Some)
        .map_err(|error| CoreError::Configuration(format!("invalid setting {key}: {error}")))
}

async fn initialize_imported_bottle(
    database: &Database,
    profiles: &ProfileStore,
    game: &GameSummary,
) -> Result<()> {
    let Some(default_bottle) = database
        .setting(SETTING_BOTTLES_DEFAULT)
        .await?
        .filter(|value| !value.trim().is_empty())
    else {
        return Ok(());
    };
    let mut config = profiles.load(&game.profile_key)?;
    if config.runner != crate::Runner::Bottles || config.bottle_name.is_some() {
        return Ok(());
    }
    config.bottle_name = Some(default_bottle.clone());
    profiles.save(&game.profile_key, &config)?;
    info!(
        game_id = %game.id,
        bottle = %default_bottle,
        "initialized imported game with default Bottles bottle"
    );
    Ok(())
}

fn select_runtime<'a>(
    runtimes: &'a [EngineRecord],
    engine_type: &str,
    requested_version: Option<&str>,
) -> Option<&'a EngineRecord> {
    runtimes
        .iter()
        .filter(|runtime| runtime.engine_type.eq_ignore_ascii_case(engine_type))
        .filter(|runtime| {
            requested_version.is_none_or(|version| runtime.version.eq_ignore_ascii_case(version))
        })
        .max_by_key(|runtime| runtime.installed_at)
}

fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn runtime_status(engine: EngineRecord) -> RuntimeStatus {
    RuntimeStatus {
        id: engine.id,
        name: engine.name,
        version: engine.version,
        engine_type: engine.engine_type,
        path: engine.engine_path,
    }
}
