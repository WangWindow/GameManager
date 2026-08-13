use super::game::{
    default_game_config, is_linux_native_entry, normalize_path, resolve_concrete_runner,
};
use crate::commands::state::{AppState, cached_write_config};
use crate::models::{LaunchResult, SETTING_BOTTLES_DEFAULT, SETTING_BOTTLES_ENABLED};
use crate::services::FileService;
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
    crate::services::logger::log_game_launch(&id, &game.title, &game.engine_type);

    // 更新最后游玩时间
    game_service.update_last_played(&id).await?;
    drop(game_service);

    let container_path = state.container_root_path().await;
    let file_service = FileService::new();
    let config_path = file_service.game_config_path(&container_path, &game.profile_key);
    let config_exists = config_path.exists();
    let mut config = if config_exists {
        Some(file_service.read_game_config(&config_path)?)
    } else {
        Some(default_game_config(&game))
    };
    let mut config_changed = !config_exists;

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
                config_changed = true;
            } else {
                let registry = state.engine_registry.lock().await;
                if let Some(entry) = registry.get_entry(&game.engine_type) {
                    let patterns = &entry.profile.launch.entry_patterns;
                    let excludes = &entry.profile.launch.exclude_patterns;
                    if let Some(exe) = crate::engines::find_executable(
                        Path::new(&game.game_path),
                        patterns,
                        excludes,
                    ) {
                        cfg.entry_path = normalize_path(&exe);
                        config_changed = true;
                    }
                }
            }
        }

        if cfg.runner == "auto" {
            let configured_entry = PathBuf::from(&cfg.entry_path);
            let entry = if configured_entry.is_absolute() {
                configured_entry
            } else {
                Path::new(&game.game_path).join(configured_entry)
            };
            let native_entry = is_linux_native_entry(&entry);
            let (strategy, sandbox_home) = {
                let registry = state.engine_registry.lock().await;
                registry
                    .get_entry(&game.engine_type)
                    .map(|engine| {
                        (
                            engine.profile.launch.strategy.clone(),
                            engine.profile.launch.sandbox_home,
                        )
                    })
                    .unwrap_or_else(|| ("bottles".to_string(), true))
            };
            cfg.runner = resolve_concrete_runner(Some(&strategy), native_entry).to_string();
            cfg.sandbox_home = if native_entry { true } else { sandbox_home };
            config_changed = true;
        }

        let mut db_lock = state.db.lock().await;
        if cfg.runner == "bottles" {
            let enabled = crate::db::get_setting(&mut *db_lock, SETTING_BOTTLES_ENABLED)
                .await?
                .map(|v| v == "1")
                .unwrap_or(false);
            if !enabled {
                return Err("当前游戏指定使用 Bottles，但 Bottles 集成未启用".to_string());
            }
            if cfg.bottle_name.as_deref().unwrap_or("").is_empty() {
                let default_bottle = crate::db::get_setting(&mut *db_lock, SETTING_BOTTLES_DEFAULT)
                    .await?
                    .and_then(|v| if v.trim().is_empty() { None } else { Some(v) });
                if let Some(name) = default_bottle {
                    cfg.bottle_name = Some(name);
                    config_changed = true;
                } else {
                    return Err("请选择 Bottles bottle".to_string());
                }
            }
        }
    }

    if config_changed {
        file_service.ensure_game_dirs(&container_path, &game.profile_key)?;
        let config_to_write = config
            .as_ref()
            .ok_or_else(|| "无法保存游戏启动配置".to_string())?;
        cached_write_config(
            &state.config_cache,
            &file_service,
            &config_path,
            &game.profile_key,
            config_to_write,
        )?;
    }

    if config.as_ref().is_some_and(|cfg| cfg.runner == "mkxpz")
        && !matches!(
            crate::models::EngineType::from_str(&game.engine_type),
            crate::models::EngineType::RpgMakerVX | crate::models::EngineType::RpgMakerVXAce
        )
    {
        return Err("mkxp-z 仅支持 RPG Maker VX / VX Ace 游戏".to_string());
    }

    // 获取 NW.js 运行时仅取决于保存的具体启动方式。
    let needs_nwjs = config.as_ref().is_some_and(|cfg| cfg.runner == "nwjs");
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

    let mkxpz_executable = if config.as_ref().is_some_and(|cfg| cfg.runner == "mkxpz") {
        let engine_service = state.engine_service.lock().await;
        let runtime = engine_service.find_latest_engine_by_type("mkxpz").await?;
        let Some(runtime) = runtime else {
            return Err("未安装 mkxp-z 运行时，请先在运行时管理中导入".to_string());
        };
        let executable = PathBuf::from(runtime.engine_path);
        if !executable.is_file() {
            return Err("mkxp-z 运行时损坏，请在运行时管理中重新导入".to_string());
        }
        Some(executable)
    } else {
        None
    };

    // 启动游戏
    let launcher_service = state.launcher_service.lock().await;
    launcher_service
        .launch_game_with_runtimes(
            &game,
            &container_path,
            nwjs_runtime_dir.as_deref(),
            mkxpz_executable.as_deref(),
            config.as_ref(),
        )
        .await
}
