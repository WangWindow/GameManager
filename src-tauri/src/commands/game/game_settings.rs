use super::game::{
    default_game_config, is_linux_native_entry, is_supported_runner, normalize_engine_type,
    resolve_concrete_runner,
};
use crate::commands::state::{AppState, cached_read_config, cached_write_config};
use crate::models::{EngineType, GameConfig};
use crate::services::FileService;
use std::path::{Path, PathBuf};
use tauri::State;

/// 获取游戏设置（settings.toml）
#[tauri::command]
pub async fn get_game_settings(
    id: String,
    state: State<'_, AppState>,
) -> Result<GameConfig, String> {
    let game = {
        let service = state.game_service.lock().await;
        service
            .get_game_by_id(&id)
            .await?
            .ok_or_else(|| format!("游戏不存在: {}", id))?
    };

    let root = state.container_root_path().await;

    let file_service = FileService::new();
    let config_path = file_service.game_config_path(&root, &game.profile_key);
    if let Some(mut config) = cached_read_config(
        &state.config_cache,
        &file_service,
        &config_path,
        &game.profile_key,
    ) {
        if config.engine_type == "nwjs" {
            config.engine_type = normalize_engine_type(&game);
        }

        if config.runner == "auto" {
            let entry = resolve_entry_path(&game.game_path, &config.entry_path);
            let (strategy, sandbox_home) = {
                let registry = state.engine_registry.lock().await;
                registry
                    .get_entry(&game.engine_type)
                    .map(|entry| {
                        (
                            entry.profile.launch.strategy.clone(),
                            entry.profile.launch.sandbox_home,
                        )
                    })
                    .unwrap_or_else(|| ("bottles".to_string(), true))
            };
            let native_entry = entry.as_deref().is_some_and(is_linux_native_entry);
            config.runner = resolve_concrete_runner(Some(&strategy), native_entry).to_string();
            config.sandbox_home = if native_entry { true } else { sandbox_home };
            cached_write_config(
                &state.config_cache,
                &file_service,
                &config_path,
                &game.profile_key,
                &config,
            )?;
        }
        return Ok(config);
    }

    let mut config = default_game_config(&game);
    let (strategy, sandbox_home, entry_patterns, exclude_patterns) = {
        let registry = state.engine_registry.lock().await;
        registry
            .get_entry(&game.engine_type)
            .map(|entry| {
                (
                    entry.profile.launch.strategy.clone(),
                    entry.profile.launch.sandbox_home,
                    entry.profile.launch.entry_patterns.clone(),
                    entry.profile.launch.exclude_patterns.clone(),
                )
            })
            .unwrap_or_else(|| ("bottles".to_string(), true, Vec::new(), Vec::new()))
    };
    if let Some(entry) = crate::engines::find_executable(
        Path::new(&game.game_path),
        &entry_patterns,
        &exclude_patterns,
    ) {
        config.entry_path = entry.to_string_lossy().to_string();
        let native_entry = is_linux_native_entry(&entry);
        config.runner = resolve_concrete_runner(Some(&strategy), native_entry).to_string();
        config.sandbox_home = if native_entry { true } else { sandbox_home };
    } else if Path::new(&game.game_path)
        .join("www")
        .join("package.json")
        .exists()
    {
        config.entry_path = "www".to_string();
        config.runner = resolve_concrete_runner(Some(&strategy), false).to_string();
        config.sandbox_home = sandbox_home;
    }
    file_service.ensure_game_dirs(&root, &game.profile_key)?;
    cached_write_config(
        &state.config_cache,
        &file_service,
        &config_path,
        &game.profile_key,
        &config,
    )?;
    Ok(config)
}

/// 保存游戏设置（settings.toml）
#[tauri::command]
pub async fn save_game_settings(
    id: String,
    input: GameConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let game = {
        let service = state.game_service.lock().await;
        service
            .get_game_by_id(&id)
            .await?
            .ok_or_else(|| format!("游戏不存在: {}", id))?
    };

    let root = state.container_root_path().await;

    let file_service = FileService::new();
    let config_path = file_service.game_config_path(&root, &game.profile_key);
    file_service.ensure_game_dirs(&root, &game.profile_key)?;

    let mut config = input;
    config.engine_type = normalize_engine_type(&game);

    if !is_supported_runner(&config.runner) {
        return Err("请选择有效的启动方式".to_string());
    }

    let engine = EngineType::from_str(&config.engine_type);
    if config.runner == "mkxpz"
        && !matches!(engine, EngineType::RpgMakerVX | EngineType::RpgMakerVXAce)
    {
        return Err("mkxp-z 仅支持 RPG Maker VX / VX Ace 游戏".to_string());
    }
    let requires_entry = matches!(engine, EngineType::Other);
    if requires_entry && config.entry_path.trim().is_empty() {
        return Err("入口文件不能为空".to_string());
    }

    if let Some(cover_file) = config.cover_file.clone() {
        let profile_dir = file_service.game_profile_dir(&root, &game.profile_key);
        let cover_path = if Path::new(&cover_file).is_absolute() {
            PathBuf::from(&cover_file)
        } else {
            let in_profile = profile_dir.join(&cover_file);
            if in_profile.exists() {
                in_profile
            } else {
                PathBuf::from(&game.game_path).join(&cover_file)
            }
        };
        if cover_path.exists() {
            if let Ok(saved) =
                file_service.save_cover_to_profile(&root, &game.profile_key, &cover_path)
            {
                let svc = state.game_service.lock().await;
                let _ = svc
                    .update_cover_path(&game.id, Some(saved.to_string_lossy().to_string()))
                    .await;
                // 同步 cover_file 为实际保存的文件名
                if let Some(name) = saved.file_name().and_then(|n| n.to_str()) {
                    config.cover_file = Some(name.to_string());
                }
            }
        }
    }

    cached_write_config(
        &state.config_cache,
        &file_service,
        &config_path,
        &game.profile_key,
        &config,
    )
}

fn resolve_entry_path(game_path: &str, entry_path: &str) -> Option<PathBuf> {
    let entry = entry_path.trim();
    if entry.is_empty() {
        return None;
    }

    let candidate = PathBuf::from(entry);
    if candidate.is_absolute() {
        candidate.exists().then_some(candidate)
    } else {
        let candidate = Path::new(game_path).join(candidate);
        candidate.exists().then_some(candidate)
    }
}
