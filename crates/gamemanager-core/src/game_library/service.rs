use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use uuid::Uuid;

use crate::{
    CoreError, CoverResolver, Database, EngineRegistry, FsDetectionContext, GameConfig, GameRecord,
    GameSummary, ProfileStore, Result, Runner,
};

use super::scan::is_nwjs_runtime_dir;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntryPoint {
    pub path: PathBuf,
}

impl EntryPoint {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportRequest {
    pub entry: EntryPoint,
    pub engine_id: Option<String>,
    pub title: Option<String>,
    pub runtime_version: Option<String>,
}

impl ImportRequest {
    pub fn from_entry(path: impl Into<PathBuf>) -> Self {
        Self {
            entry: EntryPoint::new(path),
            engine_id: None,
            title: None,
            runtime_version: None,
        }
    }

    pub fn with_engine(mut self, engine_id: impl Into<String>) -> Self {
        self.engine_id = Some(engine_id.into());
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_runtime_version(mut self, version: impl Into<String>) -> Self {
        self.runtime_version = Some(version.into());
        self
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UpdateGameRequest {
    pub title: Option<String>,
    pub entry: Option<PathBuf>,
    pub engine_id: Option<String>,
    pub runtime_version: Option<String>,
    pub metadata_json: Option<String>,
}

impl UpdateGameRequest {
    pub fn title(title: impl Into<String>) -> Self {
        Self::with_title(title)
    }

    pub fn with_title(title: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            ..Self::default()
        }
    }
}

#[derive(Clone)]
pub struct GameLibraryService {
    database: Database,
    profiles: ProfileStore,
    registry: EngineRegistry,
    covers: CoverResolver,
}

impl GameLibraryService {
    pub fn new(database: Database, profiles: ProfileStore, registry: EngineRegistry) -> Self {
        let covers = CoverResolver::new(profiles.clone());
        Self {
            database,
            profiles,
            registry,
            covers,
        }
    }

    pub fn database(&self) -> &Database {
        &self.database
    }

    pub fn profiles(&self) -> &ProfileStore {
        &self.profiles
    }

    pub fn registry(&self) -> &EngineRegistry {
        &self.registry
    }

    pub async fn list(&self) -> Result<Vec<GameSummary>> {
        Ok(self
            .database
            .games()
            .await?
            .iter()
            .map(GameSummary::from)
            .collect())
    }

    pub async fn import_game(&self, request: ImportRequest) -> Result<GameSummary> {
        let entry = canonical_entry(&request.entry.path)?;
        let game_root = if entry.is_file() {
            entry
                .parent()
                .ok_or_else(|| CoreError::InvalidPath(entry.display().to_string()))?
                .to_path_buf()
        } else {
            entry.clone()
        };
        if is_nwjs_runtime_dir(&game_root) {
            return Err(CoreError::InvalidPath(format!(
                "NW.js runtime directory cannot be imported: {}",
                game_root.display()
            )));
        }

        let existing = self.database.games().await?;
        let normalized_path = normalize_path(&game_root);
        if existing
            .iter()
            .any(|game| game.normalized_path == normalized_path)
        {
            return Err(CoreError::Configuration(format!(
                "game already imported: {}",
                game_root.display()
            )));
        }

        let detection = self
            .registry
            .detect(&FsDetectionContext::new(game_root.clone()));
        let engine_id = request
            .engine_id
            .clone()
            .or_else(|| detection.as_ref().map(|match_| match_.engine_id.clone()))
            .unwrap_or_else(|| "other".to_owned());
        let confidence = request.engine_id.as_ref().map_or_else(
            || detection.as_ref().map_or(0, |match_| match_.confidence),
            |_| 100,
        );
        let title = request
            .title
            .as_deref()
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| derive_title(&entry, &game_root));
        let profile_key = profile_key_for_title(
            &title,
            &existing
                .iter()
                .map(|game| game.profile_key.as_str())
                .collect::<HashSet<_>>(),
        );
        let now = unix_timestamp();
        let game = GameRecord {
            id: Uuid::new_v4().to_string(),
            profile_key,
            title,
            engine_type: engine_id.clone(),
            game_path: game_root.to_string_lossy().into_owned(),
            normalized_path,
            game_type: game_type_for_engine(&engine_id),
            detection_confidence: confidence,
            runtime_version: request.runtime_version,
            cover_path: None,
            play_count: 0,
            metadata_json: None,
            created_at: now,
            last_played_at: None,
            updated_at: now,
        };

        let config = default_config(&self.registry, &engine_id, &entry);
        self.database.insert_game(&game).await?;
        self.profiles.save(&game.profile_key, &config)?;
        if let Some(cover) = self
            .covers
            .refresh(&game_root, Some(&entry), &game.profile_key)?
        {
            let mut updated = game.clone();
            updated.cover_path = Some(cover.to_string_lossy().into_owned());
            self.database.update_game(&updated).await?;
            return Ok(GameSummary::from(&updated));
        }
        Ok(GameSummary::from(&game))
    }

    pub async fn update_game(&self, id: &str, request: UpdateGameRequest) -> Result<GameSummary> {
        let mut game = self
            .database
            .game(id)
            .await?
            .ok_or_else(|| CoreError::Database(format!("game not found: {id}")))?;
        if let Some(title) = request
            .title
            .as_deref()
            .map(str::trim)
            .filter(|title| !title.is_empty())
        {
            if title != game.title {
                let games = self.database.games().await?;
                let used = games
                    .iter()
                    .filter(|other| other.id != id)
                    .map(|other| other.profile_key.as_str())
                    .collect::<HashSet<_>>();
                let new_key = profile_key_for_title(title, &used);
                self.profiles.rename(&game.profile_key, &new_key)?;
                game.profile_key = new_key;
                game.title = title.to_owned();
            }
        }
        if let Some(entry) = request.entry {
            let canonical = canonical_entry(&entry)?;
            let root = if canonical.is_file() {
                canonical.parent().unwrap_or(&canonical)
            } else {
                &canonical
            };
            let normalized = normalize_path(root);
            if self
                .database
                .games()
                .await?
                .iter()
                .any(|other| other.id != id && other.normalized_path == normalized)
            {
                return Err(CoreError::Configuration(format!(
                    "game already imported: {normalized}"
                )));
            }
            game.game_path = root.to_string_lossy().into_owned();
            game.normalized_path = normalized;
            let config_path = self.profiles.config_path(&game.profile_key);
            if config_path.is_file() {
                let mut config = self.profiles.load(&game.profile_key)?;
                config.entry_path = canonical.to_string_lossy().into_owned();
                self.profiles.save(&game.profile_key, &config)?;
            }
        }
        if let Some(engine_id) = request.engine_id {
            game.engine_type = engine_id.clone();
            game.game_type = game_type_for_engine(&engine_id);
            if let Ok(mut config) = self.profiles.load(&game.profile_key) {
                config.engine_type = engine_id.clone();
                config.runner = runner_for_strategy(
                    self.registry
                        .profile(&engine_id)
                        .map(|profile| profile.launch.strategy.as_str()),
                    Path::new(&config.entry_path),
                );
                self.profiles.save(&game.profile_key, &config)?;
            }
        }
        if let Some(runtime) = request.runtime_version {
            game.runtime_version = (!runtime.trim().is_empty()).then_some(runtime);
        }
        if request.metadata_json.is_some() {
            game.metadata_json = request.metadata_json;
        }
        game.updated_at = unix_timestamp();
        self.database.update_game(&game).await?;
        Ok(GameSummary::from(&game))
    }

    pub async fn remove_game(&self, id: &str) -> Result<()> {
        self.database
            .game(id)
            .await?
            .ok_or_else(|| CoreError::Database(format!("game not found: {id}")))?;
        self.database.delete_game(id).await
    }

    pub async fn remove_all_games(&self) -> Result<()> {
        for game in self.database.games().await? {
            self.database.delete_game(&game.id).await?;
        }
        Ok(())
    }

    pub async fn refresh_cover(&self, id: &str) -> Result<GameSummary> {
        let game = self
            .database
            .game(id)
            .await?
            .ok_or_else(|| CoreError::Database(format!("game not found: {id}")))?;
        let config = self.profiles.load(&game.profile_key).ok();
        let entry = config
            .as_ref()
            .map(|config| PathBuf::from(&config.entry_path));
        let cover = self.covers.refresh(
            Path::new(&game.game_path),
            entry.as_deref(),
            &game.profile_key,
        )?;
        let mut updated = game;
        updated.cover_path = cover.map(|path| path.to_string_lossy().into_owned());
        updated.updated_at = unix_timestamp();
        self.database.update_game(&updated).await?;
        Ok(GameSummary::from(&updated))
    }
}

fn canonical_entry(path: &Path) -> Result<PathBuf> {
    if !path.is_file() && !path.is_dir() {
        return Err(CoreError::InvalidPath(format!(
            "entry does not exist: {}",
            path.display()
        )));
    }
    Ok(std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()))
}

fn normalize_path(path: &Path) -> String {
    std::fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

fn derive_title(entry: &Path, root: &Path) -> String {
    let stem = entry
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .trim();
    if !stem.is_empty()
        && !matches!(
            stem.to_ascii_lowercase().as_str(),
            "game" | "nw" | "nwjs" | "rpg_rt"
        )
    {
        return stem.to_owned();
    }
    root.file_name()
        .and_then(|value| value.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("Unnamed Game")
        .to_owned()
}

fn profile_key_for_title(title: &str, used: &HashSet<&str>) -> String {
    let mut base = title
        .chars()
        .filter_map(|character| {
            if character.is_control() || matches!(character, '/' | '\\' | ':') {
                None
            } else if character.is_whitespace() {
                Some('_')
            } else {
                Some(character)
            }
        })
        .collect::<String>();
    base = base.trim_matches('_').chars().take(40).collect();
    if base.is_empty() {
        base = "game".to_owned();
    }
    if !used.contains(base.as_str()) {
        return base;
    }
    for suffix in 1..10_000 {
        let candidate = format!("{base}-{suffix:03}");
        if !used.contains(candidate.as_str()) {
            return candidate;
        }
    }
    format!("{base}-{}", Uuid::new_v4().simple())
}

fn game_type_for_engine(engine_id: &str) -> String {
    match engine_id {
        id if id.starts_with("rpgmaker") => "rpg".to_owned(),
        "renpy" => "visual_novel".to_owned(),
        "unity" | "godot" | "unreal" | "electron" => "game".to_owned(),
        _ => "other".to_owned(),
    }
}

fn default_config(registry: &EngineRegistry, engine_id: &str, entry: &Path) -> GameConfig {
    let profile = registry.profile(engine_id);
    let strategy = profile.map(|profile| profile.launch.strategy.as_str());
    let mut config = GameConfig {
        engine_type: engine_id.to_owned(),
        entry_path: entry.to_string_lossy().into_owned(),
        runner: runner_for_strategy(strategy, entry),
        sandbox_home: profile.is_none_or(|profile| profile.launch.sandbox_home),
        ..GameConfig::default()
    };
    if config.runner == Runner::Native {
        config.sandbox_home = true;
    }
    config
}

fn runner_for_strategy(strategy: Option<&str>, entry: &Path) -> Runner {
    if is_linux_native_entry(entry) {
        return Runner::Native;
    }
    match strategy {
        Some("native") => Runner::Native,
        Some("nwjs") => Runner::Nwjs,
        Some("mkxpz") => Runner::Mkxpz,
        Some("external") => Runner::External,
        Some("bottles") | None => Runner::Bottles,
        _ => Runner::Bottles,
    }
}

fn is_linux_native_entry(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.is_file()
            && !path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
            && path
                .metadata()
                .is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        false
    }
}

fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs() as i64)
}
