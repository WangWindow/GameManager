use crate::commands::game::normalize_path;
use crate::commands::state::AppState;
use crate::model::{EngineType, LaunchResult, SETTING_BOTTLES_DEFAULT, SETTING_BOTTLES_ENABLED};
use crate::service::FileService;
use std::path::{Path, PathBuf};
use tauri::State;

/// 启动游戏
#[tauri::command]
pub async fn launch_game(id: String, state: State<'_, AppState>) -> Result<LaunchResult, String> {
    let game_service = state.game_service.lock().await;
    let game = game_service
        .get_game_by_id(&id)
        .await?
        .ok_or_else(|| format!("游戏不存在: {}", id))?;

    // 记录启动日志
    crate::service::logger::log_game_launch(&id, &game.title, &game.engine_type);

    // 更新最后游玩时间
    game_service.update_last_played(&id).await?;
    drop(game_service);

    // 获取容器根目录
    let container_root = state.container_root.lock().await;
    let container_path = crate::util::path::canonicalize(Path::new(container_root.as_str()));
    drop(container_root);

    // 获取 NW.js 运行时（MV/MZ 及所有 nwjs 策略的引擎，如 HTML）
    let engine_type = EngineType::from_str(&game.engine_type);
    let needs_nwjs = {
        let registry = state.engine_registry.lock().await;
        if let Some(entry) = registry.get_entry(&game.engine_type) {
            entry.profile.launch.strategy == "nwjs"
        } else {
            matches!(engine_type, EngineType::RpgMakerMV | EngineType::RpgMakerMZ)
        }
    };
    let nwjs_runtime_dir = if needs_nwjs {
        let engine_service = state.engine_service.lock().await;
        let engine = if let Some(version) = game.runtime_version.as_deref() {
            engine_service.find_engine("nwjs", Some(version)).await?
        } else {
            engine_service.find_latest_engine_by_type("nwjs").await?
        };
        engine.map(|e| PathBuf::from(e.engine_path))
    } else {
        None
    };

    if needs_nwjs && nwjs_runtime_dir.is_none() {
        return Err("未安装 NW.js 运行时，请先下载并安装".to_string());
    }

    let file_service = FileService::new();
    let config_path = file_service.game_config_path(&container_path, &game.profile_key);
    let mut config = if config_path.exists() {
        Some(file_service.read_game_config(&config_path)?)
    } else {
        None
    };

    if let Some(cfg) = config.as_mut() {
        if cfg.entry_path.trim().is_empty() {
            let entry_patterns = {
                let registry = state.engine_registry.lock().await;
                registry
                    .get_entry(&game.engine_type)
                    .map(|e| e.profile.launch.entry_patterns.clone())
                    .unwrap_or_default()
            };
            if entry_patterns.is_empty() {
                if Path::new(&game.game_path)
                    .join("www")
                    .join("package.json")
                    .exists()
                {
                    cfg.entry_path = "www".to_string();
                } else {
                    cfg.entry_path = "".to_string();
                }
            } else {
                let registry = state.engine_registry.lock().await;
                if let Some(entry) = registry.get_entry(&game.engine_type) {
                    let patterns = &entry.profile.launch.entry_patterns;
                    let excludes = &entry.profile.launch.exclude_patterns;
                    if let Some(exe) = crate::engine::find_executable(
                        Path::new(&game.game_path),
                        patterns,
                        excludes,
                    ) {
                        cfg.entry_path = normalize_path(&exe);
                    }
                }
            }
        }

        let mut db_lock = state.db.lock().await;
        // 非 Windows .exe 不需要 Bottles（Linux 原生应用）
        if !cfg.entry_path.to_lowercase().ends_with(".exe") {
            cfg.use_bottles = false;
            cfg.bottle_name = None;
        } else {
            let enabled = crate::db::get_setting(&mut *db_lock, SETTING_BOTTLES_ENABLED)
                .await?
                .map(|v| v == "1")
                .unwrap_or(false);
            if !enabled {
                cfg.use_bottles = false;
                cfg.bottle_name = None;
            } else if cfg.use_bottles && cfg.bottle_name.as_deref().unwrap_or("").is_empty() {
                let default_bottle = crate::db::get_setting(&mut *db_lock, SETTING_BOTTLES_DEFAULT)
                    .await?
                    .and_then(|v| if v.trim().is_empty() { None } else { Some(v) });
                if let Some(name) = default_bottle {
                    cfg.bottle_name = Some(name);
                } else {
                    return Err("请选择 Bottles bottle".to_string());
                }
            }
        }
    }

    // 启动游戏
    let launcher_service = state.launcher_service.lock().await;
    launcher_service
        .launch_game(
            &game,
            &container_path,
            nwjs_runtime_dir.as_deref(),
            config.as_ref(),
        )
        .await
}
