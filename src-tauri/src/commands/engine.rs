use crate::models::*;
use crate::services::EngineService;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// 引擎状态
pub struct EngineState {
    pub engine_service: Arc<Mutex<EngineService>>,
}

/// 获取所有引擎
#[tauri::command]
pub async fn get_engines(state: State<'_, EngineState>) -> Result<Vec<EngineDto>, String> {
    let service = state.engine_service.lock().await;
    let engines = service.get_all_engines().await?;
    let dtos = engines.into_iter().map(|e| service.to_dto(e)).collect();
    Ok(dtos)
}

/// 查找引擎
#[tauri::command]
pub async fn find_engine(
    engine_type: String,
    version: Option<String>,
    state: State<'_, EngineState>,
) -> Result<Option<EngineDto>, String> {
    let service = state.engine_service.lock().await;
    let engine = service
        .find_engine(&engine_type, version.as_deref())
        .await?;
    Ok(engine.map(|e| service.to_dto(e)))
}

/// 添加引擎
#[tauri::command]
pub async fn add_engine(
    name: String,
    version: String,
    engine_type: String,
    path: String,
    state: State<'_, EngineState>,
) -> Result<EngineDto, String> {
    let service = state.engine_service.lock().await;
    let engine = service.add_engine(name, version, engine_type, path).await?;
    Ok(service.to_dto(engine))
}

/// 删除引擎
#[tauri::command]
pub async fn delete_engine(id: String, state: State<'_, EngineState>) -> Result<(), String> {
    let service = state.engine_service.lock().await;
    service.delete_engine(&id).await
}
