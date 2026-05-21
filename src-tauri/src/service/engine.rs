use crate::db::schema::Engine;
use crate::model::EngineDto;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// 引擎管理服务
pub struct EngineService {
    db: Arc<Mutex<toasty::Db>>,
}

impl EngineService {
    /// 创建引擎服务实例
    pub fn new(db: Arc<Mutex<toasty::Db>>) -> Self {
        Self { db }
    }

    /// 获取所有已安装引擎
    pub async fn get_all_engines(&self) -> Result<Vec<Engine>, String> {
        let mut db = self.db.lock().await;
        let engines = Engine::all()
            .exec(&mut *db)
            .await
            .map_err(|e| format!("查询引擎列表失败: {}", e))?;

        Ok(engines)
    }

    /// 根据ID获取引擎
    pub async fn get_engine_by_id(&self, id: &str) -> Result<Option<Engine>, String> {
        let mut db = self.db.lock().await;
        let engine = Engine::filter_by_id(id)
            .first()
            .exec(&mut *db)
            .await
            .map_err(|e| format!("查询引擎失败: {}", e))?;

        Ok(engine)
    }

    /// 根据类型和版本查找引擎
    pub async fn find_engine(
        &self,
        engine_type: &str,
        version: Option<&str>,
    ) -> Result<Option<Engine>, String> {
        let mut db = self.db.lock().await;
        let engines = Engine::filter(Engine::fields().engine_type().eq(engine_type))
            .exec(&mut *db)
            .await
            .map_err(|e| format!("查询引擎失败: {}", e))?;

        if let Some(ver) = version {
            Ok(engines.into_iter().find(|e| e.version == ver))
        } else {
            Ok(engines.into_iter().next())
        }
    }

    /// 获取指定类型的最新引擎
    pub async fn find_latest_engine_by_type(
        &self,
        engine_type: &str,
    ) -> Result<Option<Engine>, String> {
        let mut db = self.db.lock().await;
        let mut engines = Engine::filter(Engine::fields().engine_type().eq(engine_type))
            .exec(&mut *db)
            .await
            .map_err(|e| format!("查询引擎失败: {}", e))?;

        engines.sort_by(|a, b| b.installed_at.cmp(&a.installed_at));
        Ok(engines.into_iter().next())
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
        let now = crate::util::now_unix_ms();

        let mut db = self.db.lock().await;
        toasty::create!(Engine {
            id: id.clone(),
            name: name.clone(),
            version: version.clone(),
            engine_type: engine_type.clone(),
            engine_path: path.clone(),
            installed_at: now,
        })
        .exec(&mut *db)
        .await
        .map_err(|e| format!("添加引擎失败: {}", e))?;

        let engine = Engine::get_by_id(&mut *db, &id)
            .await
            .map_err(|e| format!("查询引擎失败: {}", e))?;

        Ok(engine)
    }

    /// 删除引擎
    pub async fn delete_engine(&self, id: &str) -> Result<(), String> {
        let mut db = self.db.lock().await;
        Engine::delete_by_id(&mut *db, id)
            .await
            .map_err(|e| format!("删除引擎失败: {}", e))?;

        Ok(())
    }

    /// 更新引擎安装信息
    pub async fn update_engine_install(
        &self,
        id: &str,
        version: String,
        path: String,
    ) -> Result<(), String> {
        let mut db = self.db.lock().await;
        let mut engine = Engine::get_by_id(&mut *db, id)
            .await
            .map_err(|e| format!("查询引擎失败: {}", e))?;

        engine
            .update()
            .version(version)
            .engine_path(path)
            .installed_at(crate::util::now_unix_ms())
            .exec(&mut *db)
            .await
            .map_err(|e| format!("更新引擎失败: {}", e))?;

        Ok(())
    }

    /// 转换为DTO
    pub fn to_dto(&self, engine: Engine) -> EngineDto {
        EngineDto {
            id: engine.id,
            name: engine.name,
            version: engine.version,
            engine_type: engine.engine_type,
            path: engine.engine_path,
            installed_at: engine.installed_at,
        }
    }
}
