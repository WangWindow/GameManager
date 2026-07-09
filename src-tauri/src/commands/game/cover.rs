use super::cover_resolver::{
    resolve_cover_for_game, resolve_entry_path_for_cover, resolve_existing_cover,
};
use super::game::{default_game_config, normalize_engine_type};
use crate::commands::state::AppState;
use crate::db::schema::Game;
use crate::models::GameDto;
use crate::services::{FileService, GameService};
use std::path::Path;
use tauri::State;

/// 按优先级更新封面图标
pub(crate) async fn update_game_cover(
    service: &GameService,
    root: &Path,
    game: &Game,
    engine_type: &str,
    game_dir: &Path,
    entry_exe: Option<&Path>,
    force_extract: bool,
) -> bool {
    let file_service = FileService::new();

    // 快速同步检查：已有封面是否依然有效（仅做路径存在性检查）
    if !force_extract && let Some(existing) = resolve_existing_cover(&file_service, root, game) {
        let _ = service
            .update_cover_path(&game.id, Some(existing.to_string_lossy().to_string()))
            .await;
        return true;
    }

    // 重量级同步操作：遍历目录/PE 提取 → 放到 spawn_blocking 避免阻塞 Tauri 命令线程
    let root_buf = root.to_path_buf();
    let profile_key = game.profile_key.clone();
    let engine_type_str = engine_type.to_string();
    let game_dir_buf = game_dir.to_path_buf();
    let entry_exe_buf = entry_exe.map(|p| p.to_path_buf());

    let saved = tokio::task::spawn_blocking(move || {
        let fs = FileService::new();
        resolve_cover_for_game(
            &fs,
            &root_buf,
            &profile_key,
            &engine_type_str,
            &game_dir_buf,
            entry_exe_buf.as_deref(),
        )
    })
    .await
    .unwrap_or(None);

    if let Some(saved) = saved {
        let _ = service
            .update_cover_path(&game.id, Some(saved.to_string_lossy().to_string()))
            .await;
        let config_path = file_service.game_config_path(root, &game.profile_key);
        let mut config = if config_path.exists() {
            file_service
                .read_game_config(&config_path)
                .unwrap_or_default()
        } else {
            default_game_config(game)
        };
        if let Some(name) = saved.file_name().and_then(|n| n.to_str()) {
            if !name.trim().is_empty() {
                config.cover_file = Some(name.to_string());
                let _ = file_service.write_game_config(&config_path, &config);
            }
        }
        return true;
    }

    false
}

/// 重新提取图标/封面
#[tauri::command]
pub async fn refresh_game_cover(id: String, state: State<'_, AppState>) -> Result<GameDto, String> {
    let service = state.game_service.lock().await;
    let game = service
        .get_game_by_id(&id)
        .await?
        .ok_or_else(|| format!("游戏不存在: {}", id))?;

    let root = state.container_root_path().await;

    let file_service = FileService::new();
    let config_path = file_service.game_config_path(&root, &game.profile_key);
    let entry_exe = if config_path.exists() {
        file_service
            .read_game_config(&config_path)
            .ok()
            .and_then(|cfg| {
                resolve_entry_path_for_cover(Path::new(&game.game_path), &cfg.entry_path)
            })
    } else {
        None
    };

    let resolved_engine = normalize_engine_type(&game);
    let refreshed = update_game_cover(
        &service,
        &root,
        &game,
        &resolved_engine,
        Path::new(&game.game_path),
        entry_exe.as_deref(),
        true,
    )
    .await;

    if !refreshed {
        return Err("未找到可提取的图标，请确认入口或同目录 .exe 是否存在".to_string());
    }

    let updated = service
        .get_game_by_id(&id)
        .await?
        .ok_or_else(|| format!("游戏不存在: {}", id))?;

    Ok(service.to_dto(updated))
}
