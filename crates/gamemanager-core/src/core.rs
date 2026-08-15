use std::{collections::BTreeMap, fs, path::PathBuf};

use serde::de::DeserializeOwned;

use crate::{
    AppPaths, AppSettings, CoreError, Database, EngineDetail, EngineRecord, EngineRegistry,
    EngineSummary, GameConfig, GameLibraryService, GameSummary, ImportRequest, Operation,
    ProfileStore, RegistryWarning, Result, RuntimeManager, SETTING_BOTTLES_ENABLED,
    SETTING_CONTAINER_ROOT, SETTING_ENGINE_ENABLED, SETTING_UI_PREFERENCES, ScanPlanner,
    ScanRequest, ScanResult, UiPreferences,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrationStatus {
    pub id: String,
    pub enabled: bool,
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
}

impl GameManagerCore {
    pub async fn open(paths: AppPaths) -> Result<Self> {
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
        let profiles = ProfileStore::new(&container_root);
        let library =
            GameLibraryService::new(database.clone(), profiles.clone(), report.registry.clone());
        let runtime = RuntimeManager::new(paths.clone());
        Ok(Self {
            paths,
            database,
            profiles,
            registry: report.registry,
            registry_warnings: report.warnings,
            library,
            runtime,
            container_root,
        })
    }

    pub async fn bootstrap(&self) -> Result<BootstrapSnapshot> {
        let games = self.library.list().await?;
        let app_settings = self.app_settings().await?;
        let ui_preferences = self.ui_preferences().await?;
        let runtimes = self
            .database
            .engines()
            .await?
            .into_iter()
            .map(runtime_status)
            .collect();
        Ok(BootstrapSnapshot {
            games,
            app_settings,
            ui_preferences,
            engine_summaries: self.registry.summaries(),
            engine_details: self.registry.details(),
            engine_warnings: self.registry_warnings.clone(),
            integrations: vec![IntegrationStatus {
                id: "bottles".to_owned(),
                enabled: self
                    .database
                    .setting(SETTING_BOTTLES_ENABLED)
                    .await?
                    .is_some_and(|value| value == "true"),
            }],
            runtimes,
        })
    }

    pub async fn import_game(&self, request: ImportRequest) -> Result<GameSummary> {
        self.library.import_game(request).await
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
        let game = self.library.update_game(game_id, request).await?;
        self.profiles.save(&game.profile_key, config)?;
        Ok(game)
    }

    pub fn scan(&self, request: ScanRequest) -> Operation<ScanResult> {
        let planner = ScanPlanner::new(self.registry.clone());
        let library = self.library.clone();
        Operation::from_future("scan", async move {
            let plan = planner.plan(request)?;
            let existing = library.list().await?;
            let mut imported = Vec::new();
            for candidate in &plan.candidates {
                let canonical =
                    std::fs::canonicalize(candidate).unwrap_or_else(|_| candidate.to_path_buf());
                if existing
                    .iter()
                    .any(|game| std::path::Path::new(&game.game_path) == canonical)
                {
                    continue;
                }
                library
                    .import_game(ImportRequest::from_entry(candidate))
                    .await?;
                imported.push(candidate.clone());
            }
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

    pub async fn set_container_root(&mut self, path: impl Into<PathBuf>) -> Result<()> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(CoreError::InvalidPath("container root is empty".to_owned()));
        }
        fs::create_dir_all(&path)?;
        self.database
            .set_setting(SETTING_CONTAINER_ROOT, &path.to_string_lossy())
            .await?;
        self.container_root = path.clone();
        self.profiles = ProfileStore::new(&path);
        self.library = GameLibraryService::new(
            self.database.clone(),
            self.profiles.clone(),
            self.registry.clone(),
        );
        Ok(())
    }

    pub async fn ui_preferences(&self) -> Result<UiPreferences> {
        Ok(load_setting(&self.database, SETTING_UI_PREFERENCES)
            .await?
            .unwrap_or_default())
    }

    pub async fn save_ui_preferences(&self, preferences: &UiPreferences) -> Result<()> {
        let value = toml::to_string(preferences)
            .map_err(|error| CoreError::Configuration(error.to_string()))?;
        self.database
            .set_setting(SETTING_UI_PREFERENCES, &value)
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
}

async fn load_setting<T: DeserializeOwned>(database: &Database, key: &str) -> Result<Option<T>> {
    let Some(value) = database.setting(key).await? else {
        return Ok(None);
    };
    toml::from_str(&value)
        .map(Some)
        .map_err(|error| CoreError::Configuration(format!("invalid setting {key}: {error}")))
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
