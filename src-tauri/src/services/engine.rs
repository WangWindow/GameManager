use crate::models::*;
use sqlx::SqlitePool;
use uuid::Uuid;

/// 引擎管理服务
pub struct EngineService {
    pool: SqlitePool,
}

impl EngineService {
    /// 创建引擎服务实例
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 获取所有已安装引擎
    pub async fn get_all_engines(&self) -> Result<Vec<Engine>, String> {
        let engines = sqlx::query_as::<_, Engine>(
            "SELECT id, name, version, engine_type, path, installed_at
             FROM engines ORDER BY installed_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("查询引擎列表失败: {}", e))?;

        Ok(engines)
    }

    /// 根据类型和版本查找引擎
    pub async fn find_engine(
        &self,
        engine_type: &str,
        version: Option<&str>,
    ) -> Result<Option<Engine>, String> {
        let engine = if let Some(ver) = version {
            sqlx::query_as::<_, Engine>(
                "SELECT id, name, version, engine_type, path, installed_at
                 FROM engines WHERE engine_type = ? AND version = ? LIMIT 1",
            )
            .bind(engine_type)
            .bind(ver)
            .fetch_optional(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, Engine>(
                "SELECT id, name, version, engine_type, path, installed_at
                 FROM engines WHERE engine_type = ? LIMIT 1",
            )
            .bind(engine_type)
            .fetch_optional(&self.pool)
            .await
        }
        .map_err(|e| format!("查询引擎失败: {}", e))?;

        Ok(engine)
    }

    /// 获取指定类型的最新引擎
    pub async fn find_latest_engine_by_type(
        &self,
        engine_type: &str,
    ) -> Result<Option<Engine>, String> {
        let engine = sqlx::query_as::<_, Engine>(
            "SELECT id, name, version, engine_type, path, installed_at
             FROM engines WHERE engine_type = ? ORDER BY installed_at DESC LIMIT 1",
        )
        .bind(engine_type)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("查询引擎失败: {}", e))?;

        Ok(engine)
    }

    /// 添加引擎
    pub async fn add_engine(
        &self,
        name: String,
        version: String,
        engine_type: String,
        path: String,
    ) -> Result<Engine, String> {
        let id = Uuid::new_v4().to_string();
        let engine = Engine::new(id, name, version, engine_type, path);

        sqlx::query(
            "INSERT INTO engines (id, name, version, engine_type, path, installed_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&engine.id)
        .bind(&engine.name)
        .bind(&engine.version)
        .bind(&engine.engine_type)
        .bind(&engine.path)
        .bind(engine.installed_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("添加引擎失败: {}", e))?;

        Ok(engine)
    }

    /// 删除引擎
    pub async fn delete_engine(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM engines WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("删除引擎失败: {}", e))?;

        Ok(())
    }

    /// 转换为DTO
    pub fn to_dto(&self, engine: Engine) -> EngineDto {
        EngineDto {
            id: engine.id,
            name: engine.name,
            version: engine.version,
            engine_type: engine.engine_type,
            path: engine.path,
            installed_at: engine.installed_at,
        }
    }
}
