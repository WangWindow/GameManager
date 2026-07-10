use crate::commands::game::cover::update_game_cover;
use crate::commands::game::game::{
    default_game_config, is_linux_native_entry, is_nwjs_runtime_dir, normalize_path,
};
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

/// 引擎的递归规则可能在集合目录中命中某个子游戏的特征。
/// 若根目录下至少有两个可识别的直接子目录，则始终把它视为集合容器。
fn is_game_collection_root(registry: &crate::engines::EngineRegistry, root: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(root) else {
        return false;
    };
    let mut detected_children = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir()
            || entry.file_name().to_string_lossy().starts_with('.')
            || is_nwjs_runtime_dir(&path)
        {
            continue;
        }
        let ctx = FsDetectionContext::new(path);
        if registry
            .detect(&ctx)
            .is_some_and(|(id, _)| !registry.should_skip_scan(id))
        {
            detected_children += 1;
            if detected_children >= 2 {
                tracing::debug!(detected_children, "扫描根目录识别为游戏集合");
                return true;
            }
        }
    }
    false
}

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

    let mut scanned_dirs: u32 = 0;
    let mut found_games: u32 = 0;
    let mut imported: u32 = 0;
    let mut skipped_existing: u32 = 0;
    let mut cover_jobs = Vec::new();
    let task_id = Uuid::new_v4().to_string();

    let mut queue: VecDeque<(PathBuf, u32)> = VecDeque::new();
    queue.push_back((root.clone(), 0));

    while let Some((dir, depth)) = queue.pop_front() {
        scanned_dirs += 1;

        if scanned_dirs % 20 == 0 {
            let pending = queue.len() as f64;
            let progress =
                ((scanned_dirs as f64 / (scanned_dirs as f64 + pending).max(1.0)) * 90.0) as u8;
            let _ = app.emit(
                "scan_progress",
                serde_json::json!({
                    "taskId": task_id,
                    "label": format!("扫描中… 已扫描 {}", scanned_dirs),
                    "progress": progress.min(90),
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
            let detected = registry
                .detect(&ctx)
                .filter(|(id, _)| !registry.should_skip_scan(id))
                .map(|(id, confidence)| (id.to_string(), confidence));
            if depth == 0 && is_game_collection_root(&registry, &dir) {
                None
            } else {
                detected
            }
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
                    let (entry_patterns, runner, sandbox_home) = {
                        let registry = state.engine_registry.lock().await;
                        registry
                            .get_entry(&engine_type)
                            .map(|e| {
                                let runner = match e.profile.launch.strategy.as_str() {
                                    "nwjs" => "nwjs",
                                    "bottles" => "bottles",
                                    _ => "native",
                                };
                                (
                                    e.profile.launch.entry_patterns.clone(),
                                    runner.to_string(),
                                    e.profile.launch.sandbox_home,
                                )
                            })
                            .unwrap_or_else(|| (Vec::new(), "auto".to_string(), true))
                    };
                    config.runner = runner;
                    config.sandbox_home = sandbox_home;
                    if let Some(entry) = entry_exe.as_deref()
                        && is_linux_native_entry(entry)
                    {
                        config.runner = "native".to_string();
                        config.sandbox_home = true;
                    }
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
                    if entry_exe.is_some() {
                        let mut db_lock = state.db.lock().await;
                        if let Ok(Some(val)) =
                            crate::db::get_setting(&mut *db_lock, SETTING_BOTTLES_ENABLED).await
                        {
                            config.use_bottles = val == "1" && config.runner == "bottles";
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

                cover_jobs.push((game, engine_type.clone(), dir.clone(), entry_exe));
            }

            // 自动扫描中，一个游戏目录就是扫描边界。子目录只能通过手动导入添加。
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

    // 图标提取可能需要读取大型 PE 文件，不应阻塞扫描结果返回。
    if !cover_jobs.is_empty() {
        let cover_service = service.clone();
        let cover_root = root_path.clone();
        let cover_app = app.clone();
        tauri::async_runtime::spawn(async move {
            for (game, engine_type, game_dir, entry_exe) in cover_jobs {
                update_game_cover(
                    &cover_service,
                    &cover_root,
                    &game,
                    &engine_type,
                    &game_dir,
                    entry_exe.as_deref(),
                    false,
                )
                .await;
            }
            let _ = cover_app.emit("game_covers_updated", ());
        });
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
