use crate::db::schema::Game;
use crate::models::{AddGameInput, EngineType, GameDto, UpdateGameInput};
use crate::services::fs::FileService;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// 游戏管理服务
#[derive(Clone)]
pub struct GameService {
    db: Arc<Mutex<toasty::Db>>,
}

impl GameService {
    /// 创建游戏服务实例
    pub fn new(db: Arc<Mutex<toasty::Db>>) -> Self {
        Self { db }
    }

    /// 获取所有游戏列表
    pub async fn get_all_games(&self) -> Result<Vec<Game>, String> {
        let mut db = self.db.lock().await;
        let games = Game::all()
            .exec(&mut *db)
            .await
            .map_err(|e| format!("查询游戏列表失败: {}", e))?;

        Ok(games)
    }

    /// 根据ID获取游戏
    pub async fn get_game_by_id(&self, id: &str) -> Result<Option<Game>, String> {
        let mut db = self.db.lock().await;
        let game = Game::filter_by_id(id)
            .first()
            .exec(&mut *db)
            .await
            .map_err(|e| format!("查询游戏失败: {}", e))?;

        Ok(game)
    }
    /// 根据路径获取游戏（path 必须为规范化目录路径）
    pub async fn get_game_by_path(&self, path: &str) -> Result<Option<Game>, String> {
        let normalized_input = crate::utils::path::canonicalize(std::path::Path::new(path))
            .to_string_lossy()
            .to_string();

        let mut db = self.db.lock().await;

        // 先尝试精确匹配
        let games = Game::filter_by_normalized_path(&normalized_input)
            .exec(&mut *db)
            .await
            .map_err(|e| format!("查询游戏失败: {}", e))?;
        if !games.is_empty() {
            return Ok(Some(games.into_iter().next().unwrap()));
        }

        // 如果未找到，做一次归一化比较以兼容历史数据
        let all_games = Game::all()
            .exec(&mut *db)
            .await
            .map_err(|e| format!("查询游戏失败: {}", e))?;

        for g in all_games {
            let g_norm = crate::utils::path::canonicalize(std::path::Path::new(&g.game_path))
                .to_string_lossy()
                .to_string();
            if g_norm == normalized_input {
                return Ok(Some(g));
            }
        }

        Ok(None)
    }
    /// 添加新游戏
    pub async fn add_game(&self, input: AddGameInput) -> Result<Game, String> {
        // 规范化路径并检查是否已存在（按路径）
        let normalized_path = crate::utils::path::canonicalize(Path::new(&input.path))
            .to_string_lossy()
            .to_string();
        if let Ok(Some(_)) = self.get_game_by_path(&normalized_path).await {
            return Err("游戏已存在".to_string());
        }

        // 生成游戏ID
        let id = Uuid::new_v4().to_string();

        // 如果没有提供标题，从路径提取
        let title = input.title.unwrap_or_else(|| {
            Path::new(&normalized_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("未命名游戏")
                .to_string()
        });

        // 生成profile目录名
        let profile_key = self.generate_profile_key(&title).await?;

        let engine_type = EngineType::from_str(&input.engine_type);
        let game_type = input
            .game_type
            .unwrap_or_else(|| infer_game_type_from_engine(&input.engine_type));
        let detection_confidence = input.detection_confidence.unwrap_or(0).clamp(0, 100);
        let now = crate::utils::now_unix_ms();

        let mut db = self.db.lock().await;
        toasty::create!(Game {
            id: id.clone(),
            profile_key: profile_key.clone(),
            title: title.clone(),
            engine_type: engine_type.as_str().to_string(),
            game_path: normalized_path.clone(),
            normalized_path: normalized_path.clone(),
            game_type,
            detection_confidence,
            runtime_version: input.runtime_version.clone(),
            metadata_json: input.metadata_json.clone(),
            created_at: now,
            updated_at: now,
        })
        .exec(&mut *db)
        .await
        .map_err(|e| format!("添加游戏失败: {}", e))?;

        // Re-fetch to get the fully populated model
        let game = Game::get_by_id(&mut *db, &id)
            .await
            .map_err(|e| format!("查询新游戏失败: {}", e))?;

        Ok(game)
    }

    /// 更新游戏信息
    pub async fn update_game(&self, id: &str, input: UpdateGameInput) -> Result<Game, String> {
        let mut db = self.db.lock().await;

        // 检查游戏是否存在
        let mut game = Game::filter_by_id(id)
            .first()
            .exec(&mut *db)
            .await
            .map_err(|e| format!("查询游戏失败: {}", e))?
            .ok_or_else(|| format!("游戏不存在: {}", id))?;

        // 更新字段
        if let Some(title) = input.title {
            game.title = title;
        }
        if let Some(engine_type) = input.engine_type {
            game.engine_type = engine_type;
        }
        if let Some(ref path) = input.path {
            // 规范化并保存路径，并检查冲突
            let normalized = crate::utils::path::canonicalize(Path::new(path))
                .to_string_lossy()
                .to_string();
            let conflict = Game::filter_by_normalized_path(&normalized)
                .first()
                .exec(&mut *db)
                .await
                .map_err(|e| format!("查询游戏失败: {}", e))?;
            if let Some(existing) = conflict {
                if existing.id != game.id {
                    return Err("目标路径已被其它游戏占用".to_string());
                }
            }
            game.game_path = normalized.clone();
            game.normalized_path = normalized;
        }
        if let Some(game_type) = input.game_type {
            game.game_type = game_type;
        }
        if let Some(confidence) = input.detection_confidence {
            game.detection_confidence = confidence.clamp(0, 100);
        }
        if let Some(metadata_json) = input.metadata_json {
            game.metadata_json = Some(metadata_json);
        }
        if let Some(runtime_version) = input.runtime_version {
            game.runtime_version = Some(runtime_version);
        }

        game.updated_at = crate::utils::now_unix_ms();

        // 更新数据库 — clone values before update() consumes the model
        let title = game.title.clone();
        let engine_type = game.engine_type.clone();
        let path = game.game_path.clone();
        let normalized_path = game.normalized_path.clone();
        let game_type = game.game_type.clone();
        let detection_confidence = game.detection_confidence;
        let runtime_version = game.runtime_version.clone();
        let metadata_json = game.metadata_json.clone();
        let updated_at = game.updated_at;

        game.update()
            .title(title)
            .engine_type(engine_type)
            .game_path(path)
            .normalized_path(normalized_path)
            .game_type(game_type)
            .detection_confidence(detection_confidence)
            .runtime_version(runtime_version)
            .metadata_json(metadata_json)
            .updated_at(updated_at)
            .exec(&mut *db)
            .await
            .map_err(|e| format!("更新游戏失败: {}", e))?;

        // Re-fetch to return updated model
        let game = Game::get_by_id(&mut *db, id)
            .await
            .map_err(|e| format!("查询游戏失败: {}", e))?;

        Ok(game)
    }

    /// 删除游戏
    pub async fn delete_game(&self, id: &str) -> Result<(), String> {
        let mut db = self.db.lock().await;
        Game::delete_by_id(&mut *db, id)
            .await
            .map_err(|e| format!("删除游戏失败: {}", e))?;

        Ok(())
    }

    /// 清空游戏库记录，不删除实际游戏文件或容器目录。
    pub async fn delete_all_games(&self) -> Result<u32, String> {
        let mut db = self.db.lock().await;
        let games = Game::all()
            .exec(&mut *db)
            .await
            .map_err(|e| format!("查询游戏列表失败: {}", e))?;
        let count = games.len() as u32;
        for game in games {
            Game::delete_by_id(&mut *db, &game.id)
                .await
                .map_err(|e| format!("清空游戏库失败: {}", e))?;
        }
        Ok(count)
    }

    /// 更新游戏最后游玩时间
    pub async fn update_last_played(&self, id: &str) -> Result<(), String> {
        let mut db = self.db.lock().await;
        let mut game = Game::get_by_id(&mut *db, id)
            .await
            .map_err(|e| format!("查询游戏失败: {}", e))?;

        let now = crate::utils::now_unix_ms();
        let new_play_count = game.play_count + 1;
        game.update()
            .last_played_at(Some(now))
            .play_count(new_play_count)
            .updated_at(now)
            .exec(&mut *db)
            .await
            .map_err(|e| format!("更新游玩时间失败: {}", e))?;

        Ok(())
    }

    /// 更新游戏封面路径
    pub async fn update_cover_path(
        &self,
        id: &str,
        cover_path: Option<String>,
    ) -> Result<(), String> {
        let mut db = self.db.lock().await;
        let mut game = Game::get_by_id(&mut *db, id)
            .await
            .map_err(|e| format!("查询游戏失败: {}", e))?;

        game.update()
            .cover_path(cover_path)
            .updated_at(crate::utils::now_unix_ms())
            .exec(&mut *db)
            .await
            .map_err(|e| format!("更新封面路径失败: {}", e))?;

        Ok(())
    }

    /// 转换为DTO
    pub fn to_dto(&self, game: Game) -> GameDto {
        let path_valid = Path::new(&game.game_path).exists();
        GameDto {
            id: game.id,
            title: game.title,
            engine_type: game.engine_type,
            path: game.game_path,
            game_type: game.game_type,
            detection_confidence: game.detection_confidence,
            path_valid,
            runtime_version: game.runtime_version,
            cover_path: game.cover_path,
            play_count: game.play_count,
            created_at: game.created_at,
            last_played_at: game.last_played_at,
            updated_at: game.updated_at,
        }
    }

    /// 迁移profile目录命名（从UUID迁移到可读格式）
    pub async fn migrate_profile_keys(&self, container_root: &Path) -> Result<(), String> {
        let games = self.get_all_games().await?;
        if games.is_empty() {
            return Ok(());
        }

        let mut used: HashSet<String> = games.iter().map(|g| g.profile_key.clone()).collect();
        let file_service = FileService::new();

        for game in games {
            if !self.needs_profile_key_migration(&game.profile_key) {
                continue;
            }

            let new_key = self.generate_profile_key_from_used(&game.title, &used);
            if new_key == game.profile_key {
                continue;
            }

            file_service.migrate_profile_dir(container_root, &game.profile_key, &new_key)?;

            let mut db = self.db.lock().await;
            let mut g = Game::get_by_id(&mut *db, &game.id)
                .await
                .map_err(|e| format!("查询游戏失败: {}", e))?;
            g.update()
                .profile_key(new_key.clone())
                .exec(&mut *db)
                .await
                .map_err(|e| format!("更新profile_key失败: {}", e))?;

            used.insert(new_key);
        }

        Ok(())
    }

    fn needs_profile_key_migration(&self, key: &str) -> bool {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            return true;
        }
        let is_uuid_like =
            trimmed.len() == 36 && trimmed.chars().filter(|c| *c == '-').count() == 4;
        is_uuid_like
    }

    fn generate_profile_key_from_used(&self, title: &str, used: &HashSet<String>) -> String {
        let base = sanitize_profile_base(title);
        let mut max_num = 0u32;
        for key in used {
            if let Some(num) = parse_profile_suffix(key, &base) {
                max_num = max_num.max(num);
            }
        }
        format_profile_key(&base, max_num + 1)
    }

    async fn generate_profile_key(&self, title: &str) -> Result<String, String> {
        let base = sanitize_profile_base(title);

        let mut db = self.db.lock().await;
        let all_games = Game::all()
            .exec(&mut *db)
            .await
            .map_err(|e| format!("读取profile_key失败: {}", e))?;

        let mut max_num = 0u32;
        for game in all_games {
            if let Some(num) = parse_profile_suffix(&game.profile_key, &base) {
                max_num = max_num.max(num);
            }
        }

        Ok(format_profile_key(&base, max_num + 1))
    }
}

fn sanitize_profile_base(title: &str) -> String {
    let mut base = String::new();
    for ch in title.chars() {
        if ch.is_control() || ch == '/' || ch == '\\' || ch == ':' {
            continue;
        }
        if ch.is_whitespace() {
            base.push('_');
        } else {
            base.push(ch);
        }
    }
    let trimmed = base.trim_matches('_');
    let mut normalized = trimmed.to_string();
    if normalized.is_empty() {
        normalized = "game".to_string();
    }
    normalized.chars().take(40).collect()
}

fn parse_profile_suffix(key: &str, base: &str) -> Option<u32> {
    let prefix = format!("{}-", base);
    if !key.starts_with(&prefix) {
        return None;
    }
    key[prefix.len()..].parse::<u32>().ok()
}

fn format_profile_key(base: &str, num: u32) -> String {
    format!("{}-{:03}", base, num)
}

fn infer_game_type_from_engine(engine_type: &str) -> String {
    match EngineType::from_str(engine_type) {
        EngineType::RenPy => "visual_novel".to_string(),
        EngineType::RpgMakerVX
        | EngineType::RpgMakerVXAce
        | EngineType::RpgMakerMV
        | EngineType::RpgMakerMZ => "rpg".to_string(),
        EngineType::Unity | EngineType::Godot => "game".to_string(),
        EngineType::Html | EngineType::Other => "unknown".to_string(),
    }
}
