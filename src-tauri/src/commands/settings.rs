use crate::models::*;
use crate::services::{EngineService, GameService, db, download::nwjs};
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

/// 设置状态
pub struct SettingsState {
    pub pool: SqlitePool,
    pub game_service: Arc<Mutex<GameService>>,
    pub engine_service: Arc<Mutex<EngineService>>,
    pub container_root: Arc<Mutex<String>>,
}

/// 获取应用设置
#[tauri::command]
pub async fn get_app_settings(state: State<'_, SettingsState>) -> Result<AppSettings, String> {
    let container_root = state.container_root.lock().await;
    Ok(AppSettings {
        container_root: container_root.clone(),
    })
}

/// 更新容器根目录
#[tauri::command]
pub async fn set_container_root(
    input: SetContainerRootInput,
    state: State<'_, SettingsState>,
) -> Result<(), String> {
    // 验证路径
    let path = std::path::Path::new(&input.container_root);
    if !path.exists() {
        crate::services::path::ensure_dir(path)?;
    }

    // 保存到数据库
    db::set_setting(&state.pool, SETTING_CONTAINER_ROOT, &input.container_root).await?;

    // 更新状态
    let mut container_root = state.container_root.lock().await;
    *container_root = input.container_root;

    Ok(())
}

/// 获取 NW.js 稳定版信息
#[tauri::command]
pub async fn get_nwjs_stable_info() -> Result<nwjs::NwjsStableInfo, String> {
    nwjs::get_stable_info().await
}

/// 下载 NW.js 稳定版
#[tauri::command]
pub async fn download_nwjs_stable(
    flavor: String,
    app: AppHandle,
    state: State<'_, SettingsState>,
) -> Result<nwjs::NwjsInstallResult, String> {
    let info = nwjs::get_stable_info().await?;
    let flavor = match flavor.as_str() {
        "sdk" => nwjs::NwjsFlavor::Sdk,
        _ => nwjs::NwjsFlavor::Normal,
    };

    let result = nwjs::download_and_install(&app, info.version, flavor, info.target).await?;

    let engine_service = state.engine_service.lock().await;
    let exists = engine_service
        .find_engine("nwjs", Some(&result.version))
        .await?
        .is_some();

    if !exists {
        let name = match result.flavor {
            nwjs::NwjsFlavor::Sdk => "NW.js (SDK)",
            nwjs::NwjsFlavor::Normal => "NW.js",
        };
        let _ = engine_service
            .add_engine(
                name.to_string(),
                result.version.clone(),
                "nwjs".to_string(),
                result.install_dir.clone(),
            )
            .await;
    }

    Ok(result)
}

/// 清理无用容器
#[tauri::command]
pub async fn cleanup_unused_containers(
    state: State<'_, SettingsState>,
) -> Result<CleanupResult, String> {
    let container_root = state.container_root.lock().await;
    let root = std::path::PathBuf::from(container_root.as_str());
    drop(container_root);

    let service = state.game_service.lock().await;
    let games = service.get_all_games().await?;
    let valid_ids: std::collections::HashSet<String> =
        games.into_iter().map(|g| g.profile_key).collect();

    let profiles_dir = root.join("profiles");
    if !profiles_dir.exists() {
        return Ok(CleanupResult { deleted: 0 });
    }

    let mut deleted: u32 = 0;
    if let Ok(entries) = std::fs::read_dir(&profiles_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let key = entry.file_name().to_string_lossy().to_string();
            if !valid_ids.contains(&key) {
                std::fs::remove_dir_all(&path).map_err(|e| format!("清理容器失败: {}", e))?;
                deleted += 1;
            }
        }
    }

    Ok(CleanupResult { deleted })
}
