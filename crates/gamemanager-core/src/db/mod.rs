mod schema;

use std::{path::Path, sync::Arc};

use tokio::sync::Mutex;

use crate::{AppPaths, CoreError, EngineRecord, GameRecord, Result};

/// SQLite repository preserving the v0.9 database location and table names.
#[derive(Clone)]
pub struct Database {
    connection: Arc<Mutex<toasty::Db>>,
}

impl Database {
    pub async fn open(paths: &AppPaths) -> Result<Self> {
        let database_path = paths.database();
        ensure_parent(&database_path)?;

        let database_exists = database_path.exists();
        let connection_string = format!("sqlite://{}", database_path.display());
        let database = toasty::Db::builder()
            .models(toasty::models!(
                schema::Game,
                schema::Engine,
                schema::Setting
            ))
            .connect(&connection_string)
            .await
            .map_err(CoreError::database)?;

        if !database_exists {
            database.push_schema().await.map_err(CoreError::database)?;
        }

        Ok(Self {
            connection: Arc::new(Mutex::new(database)),
        })
    }

    pub async fn games(&self) -> Result<Vec<GameRecord>> {
        let mut connection = self.connection.lock().await;
        let games = schema::Game::all()
            .exec(&mut *connection)
            .await
            .map_err(CoreError::database)?;

        Ok(games.into_iter().map(Into::into).collect())
    }

    pub async fn game(&self, id: &str) -> Result<Option<GameRecord>> {
        let mut connection = self.connection.lock().await;
        schema::Game::filter_by_id(id)
            .first()
            .exec(&mut *connection)
            .await
            .map_err(CoreError::database)
            .map(|game| game.map(Into::into))
    }

    pub async fn insert_game(&self, game: &GameRecord) -> Result<()> {
        let mut connection = self.connection.lock().await;
        toasty::create!(schema::Game {
            id: game.id.clone(),
            profile_key: game.profile_key.clone(),
            title: game.title.clone(),
            engine_type: game.engine_type.clone(),
            game_path: game.game_path.clone(),
            normalized_path: game.normalized_path.clone(),
            game_type: game.game_type.clone(),
            detection_confidence: game.detection_confidence,
            runtime_version: game.runtime_version.clone(),
            cover_path: game.cover_path.clone(),
            play_count: game.play_count,
            metadata_json: game.metadata_json.clone(),
            created_at: game.created_at,
            last_played_at: game.last_played_at,
            updated_at: game.updated_at,
        })
        .exec(&mut *connection)
        .await
        .map_err(CoreError::database)?;

        Ok(())
    }

    pub async fn update_game(&self, game: &GameRecord) -> Result<()> {
        let mut connection = self.connection.lock().await;
        let mut existing = schema::Game::filter_by_id(&game.id)
            .first()
            .exec(&mut *connection)
            .await
            .map_err(CoreError::database)?
            .ok_or_else(|| CoreError::Database(format!("game not found: {}", game.id)))?;

        existing
            .update()
            .profile_key(game.profile_key.clone())
            .title(game.title.clone())
            .engine_type(game.engine_type.clone())
            .game_path(game.game_path.clone())
            .normalized_path(game.normalized_path.clone())
            .game_type(game.game_type.clone())
            .detection_confidence(game.detection_confidence)
            .runtime_version(game.runtime_version.clone())
            .cover_path(game.cover_path.clone())
            .play_count(game.play_count)
            .metadata_json(game.metadata_json.clone())
            .created_at(game.created_at)
            .last_played_at(game.last_played_at)
            .updated_at(game.updated_at)
            .exec(&mut *connection)
            .await
            .map_err(CoreError::database)?;

        Ok(())
    }

    pub async fn delete_game(&self, id: &str) -> Result<()> {
        let mut connection = self.connection.lock().await;
        schema::Game::delete_by_id(&mut *connection, id)
            .await
            .map_err(CoreError::database)
    }

    pub async fn engines(&self) -> Result<Vec<EngineRecord>> {
        let mut connection = self.connection.lock().await;
        let engines = schema::Engine::all()
            .exec(&mut *connection)
            .await
            .map_err(CoreError::database)?;

        Ok(engines.into_iter().map(Into::into).collect())
    }

    pub async fn insert_engine(&self, engine: &EngineRecord) -> Result<()> {
        let mut connection = self.connection.lock().await;
        toasty::create!(schema::Engine {
            id: engine.id.clone(),
            name: engine.name.clone(),
            version: engine.version.clone(),
            engine_type: engine.engine_type.clone(),
            engine_path: engine.engine_path.clone(),
            installed_at: engine.installed_at,
        })
        .exec(&mut *connection)
        .await
        .map_err(CoreError::database)?;

        Ok(())
    }

    pub async fn setting(&self, key: &str) -> Result<Option<String>> {
        let mut connection = self.connection.lock().await;
        schema::Setting::filter_by_key(key)
            .first()
            .exec(&mut *connection)
            .await
            .map_err(CoreError::database)
            .map(|setting| setting.map(|setting| setting.value))
    }

    pub async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let mut connection = self.connection.lock().await;
        let setting = schema::Setting::filter_by_key(key)
            .first()
            .exec(&mut *connection)
            .await
            .map_err(CoreError::database)?;

        if let Some(mut setting) = setting {
            setting
                .update()
                .value(value.to_owned())
                .exec(&mut *connection)
                .await
                .map_err(CoreError::database)?;
        } else {
            toasty::create!(schema::Setting {
                key: key.to_owned(),
                value: value.to_owned(),
            })
            .exec(&mut *connection)
            .await
            .map_err(CoreError::database)?;
        }

        Ok(())
    }
}

fn ensure_parent(path: &Path) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        CoreError::InvalidPath(format!("database path has no parent: {}", path.display()))
    })?;
    std::fs::create_dir_all(parent)?;
    Ok(())
}

impl From<schema::Game> for GameRecord {
    fn from(game: schema::Game) -> Self {
        Self {
            id: game.id,
            profile_key: game.profile_key,
            title: game.title,
            engine_type: game.engine_type,
            game_path: game.game_path,
            normalized_path: game.normalized_path,
            game_type: game.game_type,
            detection_confidence: game.detection_confidence,
            runtime_version: game.runtime_version,
            cover_path: game.cover_path,
            play_count: game.play_count,
            metadata_json: game.metadata_json,
            created_at: game.created_at,
            last_played_at: game.last_played_at,
            updated_at: game.updated_at,
        }
    }
}

impl From<schema::Engine> for EngineRecord {
    fn from(engine: schema::Engine) -> Self {
        Self {
            id: engine.id,
            name: engine.name,
            version: engine.version,
            engine_type: engine.engine_type,
            engine_path: engine.engine_path,
            installed_at: engine.installed_at,
        }
    }
}
