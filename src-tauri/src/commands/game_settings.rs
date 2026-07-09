use crate::commands::game::{default_game_config, normalize_engine_type};
use crate::commands::state::{AppState, cached_read_config, cached_write_config};
use crate::model::{EngineType, GameConfig};
use crate::service::FileService;
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

    let container_root = state.container_root.lock().await;
    let root = crate::util::path::canonicalize(Path::new(container_root.as_str()));
    drop(container_root);

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
        return Ok(config);
    }

    Ok(default_game_config(&game))
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

    let container_root = state.container_root.lock().await;
    let root = crate::util::path::canonicalize(Path::new(container_root.as_str()));
    drop(container_root);

    let file_service = FileService::new();
    let config_path = file_service.game_config_path(&root, &game.profile_key);
    file_service.ensure_game_dirs(&root, &game.profile_key)?;

    let mut config = input;
    config.engine_type = normalize_engine_type(&game);

    let engine = EngineType::from_str(&config.engine_type);
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
