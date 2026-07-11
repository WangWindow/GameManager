use crate::db::schema::Engine;
use crate::models::{
    AppSettings, CleanupResult, SETTING_CONTAINER_ROOT, SetContainerRootInput,
};
use crate::services::{EngineService, GameService, download::mkxpz, download::nwjs};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;

/// 设置状态
pub struct SettingsState {
    pub db: Arc<Mutex<toasty::Db>>,
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
        crate::utils::path::ensure_dir(path)?;
    }

    // 保存到数据库
    let mut db_lock = state.db.lock().await;
    crate::db::set_setting(&mut *db_lock, SETTING_CONTAINER_ROOT, &input.container_root).await?;
    drop(db_lock);

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
    let all = engine_service.get_all_engines().await?;
    let target_name = nwjs_flavor_name(result.flavor);

    let mut current_id: Option<String> = None;
    for engine in &all {
        if engine.engine_type != "nwjs" {
            continue;
        }
        if !is_same_nwjs_flavor(engine, result.flavor) {
            continue;
        }
        if engine.version == result.version {
            current_id = Some(engine.id.clone());
            break;
        }
    }

    if current_id.is_none() {
        let added = engine_service
            .add_engine(
                target_name.to_string(),
                result.version.clone(),
                "nwjs".to_string(),
                result.install_dir.clone(),
            )
            .await?;
        current_id = Some(added.id);
    }

    // 默认清理旧版，仅保留最新版本
    prune_old_nwjs_engines(&engine_service, &app, current_id.as_deref(), result.flavor).await?;

    Ok(result)
}

fn nwjs_flavor_name(flavor: nwjs::NwjsFlavor) -> &'static str {
    match flavor {
        nwjs::NwjsFlavor::Sdk => "NW.js (SDK)",
        nwjs::NwjsFlavor::Normal => "NW.js",
    }
}

fn is_same_nwjs_flavor(engine: &Engine, flavor: nwjs::NwjsFlavor) -> bool {
    let lower = engine.name.to_lowercase();
    match flavor {
        nwjs::NwjsFlavor::Sdk => lower.contains("sdk"),
        nwjs::NwjsFlavor::Normal => !lower.contains("sdk"),
    }
}

/// 清理旧版 NW.js（按 flavor 仅保留最新）。
async fn prune_old_nwjs_engines(
    engine_service: &EngineService,
    app: &AppHandle,
    keep_id: Option<&str>,
    keep_flavor: nwjs::NwjsFlavor,
) -> Result<(), String> {
    let engines = engine_service.get_all_engines().await?;

    for engine in engines {
        if engine.engine_type != "nwjs" {
            continue;
        }
        if !is_same_nwjs_flavor(&engine, keep_flavor) {
            continue;
        }
        if keep_id == Some(engine.id.as_str()) {
            continue;
        }
        remove_engine_path_if_owned(app, &engine.engine_path);
        engine_service.delete_engine(&engine.id).await?;
    }

    Ok(())
}

fn remove_engine_path_if_owned(app: &AppHandle, path: &str) {
    if let Ok(app_data_dir) = app.path().app_data_dir() {
        let engine_path = crate::utils::path::canonicalize(std::path::Path::new(path));
        if crate::utils::path::is_within(&engine_path, &app_data_dir) {
            if engine_path.is_dir() {
                let _ = std::fs::remove_dir_all(&engine_path);
                if let Some(parent) = engine_path.parent() {
                    let _ = std::fs::remove_dir(parent);
                }
            } else if engine_path.is_file() {
                let _ = std::fs::remove_file(&engine_path);
            }
        }
    }
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

// ── mkxp-z 命令 ──────────────────────────────────────────

/// 从本地 ZIP 文件导入安装 mkxp-z。
#[tauri::command]
pub async fn import_mkxpz_archive(
    archive_path: String,
    app: AppHandle,
    state: State<'_, SettingsState>,
) -> Result<mkxpz::MkxpzImportResult, String> {
    let result = mkxpz::import_from_archive(&app, std::path::Path::new(&archive_path))?;

    let engine_service = state.engine_service.lock().await;
    let all = engine_service.get_all_engines().await?;

    // 查找是否已安装相同版本
    let mut current_id: Option<String> = None;
    for engine in &all {
        if engine.engine_type != "mkxpz" {
            continue;
        }
        if engine.version == result.version {
            current_id = Some(engine.id.clone());
            break;
        }
    }

    if current_id.is_none() {
        let added = engine_service
            .add_engine(
                "mkxp-z".to_string(),
                result.version.clone(),
                "mkxpz".to_string(),
                result.install_dir.clone(),
            )
            .await?;
        current_id = Some(added.id);
    } else if let Some(id) = &current_id {
        // 已存在同版本，覆盖安装目录
        engine_service
            .update_engine_install(id, result.version.clone(), result.install_dir.clone())
            .await?;
    }

    // 清理旧版本（仅保留最新）
    prune_old_mkxpz_engines(&engine_service, &app, current_id.as_deref()).await?;

    Ok(result)
}

/// 删除旧版 mkxp-z（仅保留最新安装项）。
async fn prune_old_mkxpz_engines(
    engine_service: &EngineService,
    app: &AppHandle,
    keep_id: Option<&str>,
) -> Result<(), String> {
    let engines = engine_service.get_all_engines().await?;

    for engine in engines {
        if engine.engine_type != "mkxpz" {
            continue;
        }
        if keep_id == Some(engine.id.as_str()) {
            continue;
        }
        remove_engine_path_if_owned(app, &engine.engine_path);
        engine_service.delete_engine(&engine.id).await?;
    }

    Ok(())
}
