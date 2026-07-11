use super::cover_resolver::fill_cover_from_config;
use crate::commands::state::{AppState, cache_remove};
use crate::db::schema::Game;
use crate::models::{AddGameInput, GameConfig, GameDto, UpdateGameInput};
use crate::services::FileService;
use std::path::{Path, PathBuf};
use tauri::State;

/// 获取所有游戏
#[tauri::command]
pub async fn get_games(state: State<'_, AppState>) -> Result<Vec<GameDto>, String> {
    let games = {
        let service = state.game_service.lock().await;
        service.get_all_games().await?
    };
    let root = state.container_root_path().await;

    let cache = state.config_cache.clone();
    let dtos = tokio::task::spawn_blocking(move || {
        let file_service = FileService::new();
        games
            .into_iter()
            .map(|g| {
                let path_valid = std::path::Path::new(&g.game_path).exists();
                let dto = GameDto {
                    id: g.id.clone(),
                    title: g.title.clone(),
                    engine_type: g.engine_type.clone(),
                    path: g.game_path.clone(),
                    game_type: g.game_type.clone(),
                    detection_confidence: g.detection_confidence,
                    path_valid,
                    runtime_version: g.runtime_version.clone(),
                    cover_path: g.cover_path.clone(),
                    play_count: g.play_count,
                    created_at: g.created_at,
                    last_played_at: g.last_played_at,
                    updated_at: g.updated_at,
                };
                fill_cover_from_config(&cache, &file_service, &root, &g, dto)
            })
            .collect()
    })
    .await
    .map_err(|e| format!("封面解析失败: {}", e))?;

    Ok(dtos)
}

/// 获取单个游戏
#[tauri::command]
pub async fn get_game(id: String, state: State<'_, AppState>) -> Result<Option<GameDto>, String> {
    let game = {
        let service = state.game_service.lock().await;
        service.get_game_by_id(&id).await?
    };
    let Some(game) = game else {
        return Ok(None);
    };

    let root = state.container_root_path().await;

    let cache = state.config_cache.clone();
    let dto = tokio::task::spawn_blocking(move || {
        let file_service = FileService::new();
        let path_valid = std::path::Path::new(&game.game_path).exists();
        let dto = GameDto {
            id: game.id.clone(),
            title: game.title.clone(),
            engine_type: game.engine_type.clone(),
            path: game.game_path.clone(),
            game_type: game.game_type.clone(),
            detection_confidence: game.detection_confidence,
            path_valid,
            runtime_version: game.runtime_version.clone(),
            cover_path: game.cover_path.clone(),
            play_count: game.play_count,
            created_at: game.created_at,
            last_played_at: game.last_played_at,
            updated_at: game.updated_at,
        };
        fill_cover_from_config(&cache, &file_service, &root, &game, dto)
    })
    .await
    .map_err(|e| format!("封面解析失败: {}", e))?;

    Ok(Some(dto))
}

/// 添加游戏
#[tauri::command]
pub async fn add_game(input: AddGameInput, state: State<'_, AppState>) -> Result<GameDto, String> {
    let service = state.game_service.lock().await;
    let game = service.add_game(input).await?;
    Ok(service.to_dto(game))
}

/// 更新游戏
#[tauri::command]
pub async fn update_game(
    id: String,
    input: UpdateGameInput,
    state: State<'_, AppState>,
) -> Result<GameDto, String> {
    let service = state.game_service.lock().await;
    let game = service.update_game(&id, input).await?;
    Ok(service.to_dto(game))
}

/// 删除游戏
#[tauri::command]
pub async fn delete_game(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let service = state.game_service.lock().await;
    let game = service
        .get_game_by_id(&id)
        .await?
        .ok_or_else(|| format!("游戏不存在: {}", id))?;
    let profile_key = game.profile_key.clone();
    service.delete_game(&id).await?;
    drop(service);
    cache_remove(&state.config_cache, &profile_key);
    Ok(())
}

/// 移除游戏库中的全部条目，不删除实际游戏文件。
#[tauri::command]
pub async fn remove_all_games(state: State<'_, AppState>) -> Result<u32, String> {
    let service = state.game_service.lock().await;
    let removed = service.delete_all_games().await?;
    state.config_cache.lock().unwrap().clear();
    Ok(removed)
}

/// 获取游戏 profile 目录路径
#[tauri::command]
pub async fn get_game_profile_dir(
    id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let service = state.game_service.lock().await;
    let game = service
        .get_game_by_id(&id)
        .await?
        .ok_or_else(|| format!("游戏不存在: {}", id))?;

    let container_root = state.container_root.lock().await;
    let root = PathBuf::from(container_root.as_str());
    drop(container_root);

    let file_service = FileService::new();
    let dir = file_service.game_profile_dir(&root, &game.profile_key);
    Ok(dir.to_string_lossy().to_string())
}

/// 打开本地路径（文件或目录）
#[tauri::command]
pub async fn open_path(path: String) -> Result<(), String> {
    let target = PathBuf::from(path);
    if !target.exists() {
        return Err("路径不存在".to_string());
    }

    #[cfg(target_os = "windows")]
    let mut cmd = std::process::Command::new("explorer");
    #[cfg(target_os = "macos")]
    let mut cmd = std::process::Command::new("open");
    #[cfg(target_os = "linux")]
    let mut cmd = std::process::Command::new("xdg-open");

    cmd.arg(&target)
        .spawn()
        .map_err(|e| format!("打开路径失败: {}", e))?;

    Ok(())
}

// ── Shared utility functions ──

/// 从数据库 Game 记录构建默认游戏配置，包含归一化后的引擎类型和空入口路径。
pub(crate) fn default_game_config(game: &Game) -> GameConfig {
    GameConfig {
        engine_type: normalize_engine_type(game),
        entry_path: String::new(),
        runtime_version: game.runtime_version.clone(),
        runner: "auto".to_string(),
        args: Vec::new(),
        sandbox_home: true,
        use_bottles: false,
        bottle_name: None,
        cover_file: None,
    }
}

/// 归一化引擎类型：保持数据库里记录的引擎类型，不再使用硬编码兜底检测。
pub(crate) fn normalize_engine_type(game: &Game) -> String {
    game.engine_type.clone()
}

/// 判断目录是否为 NW.js 运行时目录（非游戏目录）：检查目录名前缀及 nw/nwjs 可执行文件和 .pak/.dat 特征文件。
pub(crate) fn is_nwjs_runtime_dir(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if name.starts_with("nwjs-") || name.starts_with("nwjs-sdk-") {
        return true;
    }

    let has_exe = path.join("nw").exists() || path.join("nwjs").exists();
    let has_pak = path.join("nw.pak").exists() || path.join("nw_100_percent.pak").exists();
    let has_icudtl = path.join("icudtl.dat").exists();
    let has_locales = path.join("locales").is_dir();

    has_exe && has_pak && has_icudtl && has_locales
}

/// 将路径规范化为绝对路径（解析符号链接）并以字符串形式返回。
pub(crate) fn normalize_path(path: &Path) -> String {
    crate::utils::path::canonicalize(path)
        .to_string_lossy()
        .to_string()
}

/// 判断入口是否应作为 Linux 原生程序或可执行脚本启动。
/// Windows `.exe` 即使位于将所有文件标记为可执行的挂载盘，也不视为原生程序。
pub(crate) fn is_linux_native_entry(path: &Path) -> bool {
    crate::utils::path::is_linux_native_executable(path)
}
