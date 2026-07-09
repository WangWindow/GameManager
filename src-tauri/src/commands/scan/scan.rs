use super::scan_walk::count_dirs;
use crate::commands::game::cover::update_game_cover;
use crate::commands::game::game::{default_game_config, is_nwjs_runtime_dir, normalize_path};
use crate::commands::game::game_executable::find_renpy_launch_script;
use crate::commands::state::{AppState, cached_write_config};
use crate::engines::context::FsDetectionContext;
use crate::models::{
    AddGameInput, EngineType, SETTING_BOTTLES_ENABLED, ScanGamesInput, ScanGamesResult,
};
use crate::services::FileService;
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

/// 扫描游戏目录
#[tauri::command]
pub async fn scan_games(
    input: ScanGamesInput,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<ScanGamesResult, String> {
    let scan_start = std::time::Instant::now();

    // 记录扫描开始
    crate::services::logger::log_scan_start(&input.root, input.max_depth);

    let service = state.game_service.lock().await;
    let file_service = FileService::new();

    let root_path = state.container_root_path().await;

    let existing = service.get_all_games().await?;
    let mut existing_paths: HashSet<String> = existing
        .into_iter()
        .map(|g| normalize_path(Path::new(&g.game_path)))
        .collect();

    let root = PathBuf::from(input.root);
    if !root.exists() {
        return Err("扫描根目录不存在".to_string());
    }

    let scan_root = root.clone();
    let scan_max_depth = input.max_depth;
    let total_dirs = tokio::task::spawn_blocking(move || count_dirs(&scan_root, scan_max_depth))
        .await
        .map_err(|e| format!("目录计数失败: {}", e))?;
    let mut scanned_dirs: u32 = 0;
    let mut found_games: u32 = 0;
    let mut imported: u32 = 0;
    let mut skipped_existing: u32 = 0;
    let task_id = Uuid::new_v4().to_string();

    let mut queue: VecDeque<(PathBuf, u32)> = VecDeque::new();
    queue.push_back((root.clone(), 0));

    while let Some((dir, depth)) = queue.pop_front() {
        scanned_dirs += 1;

        if total_dirs > 0 && (scanned_dirs % 20 == 0 || scanned_dirs == total_dirs) {
            let progress = ((scanned_dirs as f64 / total_dirs as f64) * 100.0).floor() as u8;
            let _ = app.emit(
                "scan_progress",
                serde_json::json!({
                    "taskId": task_id,
                    "label": format!("扫描中… 已扫描 {}", scanned_dirs),
                    "progress": progress.min(100),
                }),
            );
        }

        if depth > input.max_depth {
            continue;
        }

        if is_nwjs_runtime_dir(&dir) {
            continue;
        }

        let detection = {
            let registry = state.engine_registry.lock().await;
            let ctx = FsDetectionContext::new(dir.clone());
            registry
                .detect(&ctx)
                .filter(|(id, _)| !registry.should_skip_scan(id))
                .map(|(id, confidence)| (id.to_string(), confidence))
        };
        if let Some((engine_type, confidence)) = detection {
            found_games += 1;
            let path_str = normalize_path(&dir);
            if existing_paths.contains(&path_str) {
                skipped_existing += 1;
            } else {
                let input = AddGameInput {
                    title: None,
                    engine_type: engine_type.clone(),
                    path: path_str.clone(),
                    game_type: None,
                    detection_confidence: Some(confidence),
                    metadata_json: None,
                    runtime_version: None,
                };

                let game = service.add_game(input).await?;
                existing_paths.insert(path_str);
                imported += 1;

                let mut entry_exe: Option<PathBuf> = None;
                if EngineType::from_str(&engine_type) == EngineType::RenPy {
                    entry_exe = find_renpy_launch_script(&dir);
                }

                if entry_exe.is_none() {
                    let registry = state.engine_registry.lock().await;
                    if let Some(engine_entry) = registry.get_entry(&engine_type) {
                        let patterns = &engine_entry.profile.launch.entry_patterns;
                        let excludes = &engine_entry.profile.launch.exclude_patterns;
                        entry_exe = crate::engines::find_executable(&dir, patterns, excludes);
                    }
                }

                let config_path = file_service.game_config_path(&root_path, &game.profile_key);
                if file_service
                    .ensure_game_dirs(&root_path, &game.profile_key)
                    .is_ok()
                {
                    let mut config = default_game_config(&game);
                    let entry_patterns = {
                        let registry = state.engine_registry.lock().await;
                        registry
                            .get_entry(&engine_type)
                            .map(|e| e.profile.launch.entry_patterns.clone())
                            .unwrap_or_default()
                    };
                    if entry_patterns.is_empty() {
                        if dir.join("www").join("package.json").exists() {
                            config.entry_path = "www".to_string();
                        } else {
                            config.entry_path = "".to_string();
                        }
                    } else if let Some(entry) = entry_exe.as_deref() {
                        config.entry_path = normalize_path(entry);
                    }

                    // 继承全局 Bottles 设置（仅 .exe）
                    if let Some(entry) = entry_exe.as_deref() {
                        let mut db_lock = state.db.lock().await;
                        if let Ok(Some(val)) =
                            crate::db::get_setting(&mut *db_lock, SETTING_BOTTLES_ENABLED).await
                        {
                            config.use_bottles = val == "1"
                                && entry.to_string_lossy().to_lowercase().ends_with(".exe");
                        }
                    }
                    let _ = cached_write_config(
                        &state.config_cache,
                        &file_service,
                        &config_path,
                        &game.profile_key,
                        &config,
                    );
                }

                update_game_cover(
                    &service,
                    &root_path,
                    &game,
                    &engine_type,
                    &dir,
                    entry_exe.as_deref(),
                    false,
                )
                .await;
            }

            // 已识别为游戏目录，跳过更深层扫描
            continue;
        }

        if depth == input.max_depth {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if let Ok(ty) = entry.file_type() {
                    if ty.is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with('.') {
                            continue;
                        }
                        queue.push_back((entry.path(), depth + 1));
                    }
                }
            }
        }
    }

    let _ = app.emit(
        "scan_progress",
        serde_json::json!({
            "taskId": task_id,
            "label": "扫描完成",
            "progress": 100,
        }),
    );

    // 记录扫描完成
    let duration_ms = scan_start.elapsed().as_millis() as u64;
    crate::services::logger::log_scan_complete(
        imported as usize,
        skipped_existing as usize,
        duration_ms,
    );

    Ok(ScanGamesResult {
        scanned_dirs,
        found_games,
        imported,
        skipped_existing,
    })
}
