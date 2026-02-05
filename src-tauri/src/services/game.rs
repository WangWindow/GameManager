use crate::models::*;
use crate::services::FileService;
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::path::Path;
use uuid::Uuid;

/// 游戏管理服务
pub struct GameService {
    pool: SqlitePool,
}

impl GameService {
    /// 创建游戏服务实例
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 获取所有游戏列表
    pub async fn get_all_games(&self) -> Result<Vec<Game>, String> {
        let games = sqlx::query_as::<_, Game>(
            "SELECT id, profile_key, title, engine_type, path, runtime_version, cover_path, created_at, last_played_at
             FROM games ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("查询游戏列表失败: {}", e))?;

        Ok(games)
    }

    /// 根据ID获取游戏
    pub async fn get_game_by_id(&self, id: &str) -> Result<Option<Game>, String> {
        let game = sqlx::query_as::<_, Game>(
            "SELECT id, profile_key, title, engine_type, path, runtime_version, cover_path, created_at, last_played_at
             FROM games WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("查询游戏失败: {}", e))?;

        Ok(game)
    }

    /// 添加新游戏
    pub async fn add_game(&self, input: AddGameInput) -> Result<Game, String> {
        // 生成游戏ID
        let id = Uuid::new_v4().to_string();

        // 如果没有提供标题，从路径提取
        let title = input.title.unwrap_or_else(|| {
            Path::new(&input.path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("未命名游戏")
                .to_string()
        });

        // 生成profile目录名
        let profile_key = self.generate_profile_key(&title).await?;

        // 创建游戏对象
        let engine_type = EngineType::from_str(&input.engine_type);
        let game = Game::new(
            id,
            profile_key,
            title,
            engine_type,
            input.path,
            input.runtime_version,
        );

        // 插入数据库
        sqlx::query(
              "INSERT INTO games (id, profile_key, title, engine_type, path, runtime_version, cover_path, created_at, last_played_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&game.id)
           .bind(&game.profile_key)
        .bind(&game.title)
        .bind(&game.engine_type)
        .bind(&game.path)
        .bind(&game.runtime_version)
        .bind(&game.cover_path)
        .bind(game.created_at)
        .bind(game.last_played_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("添加游戏失败: {}", e))?;

        Ok(game)
    }

    /// 更新游戏信息
    pub async fn update_game(&self, id: &str, input: UpdateGameInput) -> Result<Game, String> {
        // 检查游戏是否存在
        let mut game = self
            .get_game_by_id(id)
            .await?
            .ok_or_else(|| format!("游戏不存在: {}", id))?;

        // 更新字段
        if let Some(title) = input.title {
            game.title = title;
        }
        if let Some(engine_type) = input.engine_type {
            game.engine_type = engine_type;
        }
        if let Some(path) = input.path {
            game.path = path;
        }
        if let Some(runtime_version) = input.runtime_version {
            game.runtime_version = Some(runtime_version);
        }

        // 更新数据库
        sqlx::query(
            "UPDATE games SET title = ?, engine_type = ?, path = ?, runtime_version = ? WHERE id = ?"
        )
        .bind(&game.title)
        .bind(&game.engine_type)
        .bind(&game.path)
        .bind(&game.runtime_version)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("更新游戏失败: {}", e))?;

        Ok(game)
    }

    /// 删除游戏
    pub async fn delete_game(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM games WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("删除游戏失败: {}", e))?;

        Ok(())
    }

    /// 更新游戏最后游玩时间
    pub async fn update_last_played(&self, id: &str) -> Result<(), String> {
        let now = crate::utils::now_unix_ms();
        sqlx::query("UPDATE games SET last_played_at = ? WHERE id = ?")
            .bind(now)
            .bind(id)
            .execute(&self.pool)
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
        sqlx::query("UPDATE games SET cover_path = ? WHERE id = ?")
            .bind(cover_path)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("更新封面路径失败: {}", e))?;

        Ok(())
    }

    /// 转换为DTO
    pub fn to_dto(&self, game: Game) -> GameDto {
        let path_valid = Path::new(&game.path).exists();
        GameDto {
            id: game.id,
            title: game.title,
            engine_type: game.engine_type,
            path: game.path,
            path_valid,
            runtime_version: game.runtime_version,
            cover_path: game.cover_path,
            created_at: game.created_at,
            last_played_at: game.last_played_at,
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

            sqlx::query("UPDATE games SET profile_key = ? WHERE id = ?")
                .bind(&new_key)
                .bind(&game.id)
                .execute(&self.pool)
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
        let pattern = format!("{}-%", base);
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT profile_key FROM games WHERE profile_key LIKE ?")
                .bind(pattern)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| format!("读取profile_key失败: {}", e))?;

        let mut max_num = 0u32;
        for (key,) in rows {
            if let Some(num) = parse_profile_suffix(&key, &base) {
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
