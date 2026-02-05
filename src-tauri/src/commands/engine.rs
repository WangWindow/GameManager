use crate::models::*;
use crate::services::{EngineService, download::nwjs};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
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
pub async fn delete_engine(
    id: String,
    state: State<'_, EngineState>,
    app: AppHandle,
) -> Result<(), String> {
    let service = state.engine_service.lock().await;
    let engine = service.get_engine_by_id(&id).await?;

    if let Some(engine) = engine {
        if let Ok(app_data_dir) = app.path().app_data_dir() {
            let engine_path =
                crate::services::path::canonicalize_path(std::path::Path::new(&engine.path));
            if crate::services::path::is_within_dir(&engine_path, &app_data_dir) {
                if engine_path.is_dir() {
                    let _ = std::fs::remove_dir_all(&engine_path);
                } else if engine_path.is_file() {
                    let _ = std::fs::remove_file(&engine_path);
                }
            }
        }
    }

    service.delete_engine(&id).await
}

/// 获取引擎更新信息
#[tauri::command]
pub async fn get_engine_update_info(
    id: String,
    state: State<'_, EngineState>,
) -> Result<EngineUpdateInfo, String> {
    let service = state.engine_service.lock().await;
    let engine = service
        .get_engine_by_id(&id)
        .await?
        .ok_or_else(|| format!("运行器不存在: {}", id))?;

    if engine.engine_type != "nwjs" {
        return Ok(EngineUpdateInfo {
            engine_id: engine.id,
            current_version: engine.version.clone(),
            latest_version: engine.version,
            update_available: false,
        });
    }

    let info = nwjs::get_stable_info().await?;
    let update_available = is_newer_version(&engine.version, &info.version);

    Ok(EngineUpdateInfo {
        engine_id: engine.id,
        current_version: engine.version,
        latest_version: info.version,
        update_available,
    })
}

/// 更新引擎
#[tauri::command]
pub async fn update_engine(
    id: String,
    app: AppHandle,
    state: State<'_, EngineState>,
) -> Result<EngineUpdateResult, String> {
    let service = state.engine_service.lock().await;
    let engine = service
        .get_engine_by_id(&id)
        .await?
        .ok_or_else(|| format!("运行器不存在: {}", id))?;

    if engine.engine_type != "nwjs" {
        return Ok(EngineUpdateResult {
            engine_id: engine.id,
            updated: false,
            from_version: engine.version.clone(),
            to_version: engine.version,
            install_dir: None,
        });
    }

    let info = nwjs::get_stable_info().await?;
    if !is_newer_version(&engine.version, &info.version) {
        return Ok(EngineUpdateResult {
            engine_id: engine.id,
            updated: false,
            from_version: engine.version.clone(),
            to_version: engine.version,
            install_dir: None,
        });
    }

    let result = nwjs::download_and_install(
        &app,
        info.version.clone(),
        nwjs::NwjsFlavor::Normal,
        info.target,
    )
    .await?;

    if let Ok(app_data_dir) = app.path().app_data_dir() {
        let engine_path =
            crate::services::path::canonicalize_path(std::path::Path::new(&engine.path));
        if crate::services::path::is_within_dir(&engine_path, &app_data_dir) {
            if engine_path.is_dir() {
                let _ = std::fs::remove_dir_all(&engine_path);
            } else if engine_path.is_file() {
                let _ = std::fs::remove_file(&engine_path);
            }
        }
    }

    service
        .update_engine_install(&engine.id, info.version.clone(), result.install_dir.clone())
        .await?;

    Ok(EngineUpdateResult {
        engine_id: engine.id,
        updated: true,
        from_version: engine.version,
        to_version: info.version,
        install_dir: Some(result.install_dir),
    })
}

fn is_newer_version(current: &str, latest: &str) -> bool {
    fn parse(v: &str) -> Vec<u32> {
        v.split('.')
            .map(|p| p.parse::<u32>().unwrap_or(0))
            .collect()
    }

    let a = parse(current);
    let b = parse(latest);
    let max_len = a.len().max(b.len());
    for i in 0..max_len {
        let av = *a.get(i).unwrap_or(&0);
        let bv = *b.get(i).unwrap_or(&0);
        if bv > av {
            return true;
        }
        if bv < av {
            return false;
        }
    }
    false
}
