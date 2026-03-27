use crate::models::*;
use crate::services::{EngineService, FileService, GameService, LauncherService, db};
use sqlx::SqlitePool;
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;
use uuid::Uuid;

/// 应用状态
pub struct AppState {
    pub game_service: Arc<Mutex<GameService>>,
    pub engine_service: Arc<Mutex<EngineService>>,
    pub launcher_service: Arc<Mutex<LauncherService>>,
    pub pool: SqlitePool,
    pub container_root: Arc<Mutex<String>>,
}

/// 获取所有游戏
#[tauri::command]
pub async fn get_games(state: State<'_, AppState>) -> Result<Vec<GameDto>, String> {
    let service = state.game_service.lock().await;
    let games = service.get_all_games().await?;
    let container_root = state.container_root.lock().await;
    let root = crate::services::path::canonicalize_path(Path::new(container_root.as_str()));
    drop(container_root);
    let file_service = FileService::new();
    let dtos = games
        .into_iter()
        .map(|g| {
            let dto = service.to_dto(g.clone());
            fill_cover_from_config(&file_service, &root, &g, dto)
        })
        .collect();
    Ok(dtos)
}

/// 获取单个游戏
#[tauri::command]
pub async fn get_game(id: String, state: State<'_, AppState>) -> Result<Option<GameDto>, String> {
    let service = state.game_service.lock().await;
    let game = service.get_game_by_id(&id).await?;
    if let Some(game) = game {
        let container_root = state.container_root.lock().await;
        let root = crate::services::path::canonicalize_path(Path::new(container_root.as_str()));
        drop(container_root);
        let file_service = FileService::new();
        let dto = service.to_dto(game.clone());
        return Ok(Some(fill_cover_from_config(
            &file_service,
            &root,
            &game,
            dto,
        )));
    }
    Ok(None)
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
    service.delete_game(&id).await
}

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

    // 获取容器根目录
    let container_root = state.container_root.lock().await;
    let container_path =
        crate::services::path::canonicalize_path(Path::new(container_root.as_str()));
    drop(container_root);

    // 获取 NW.js 运行时（用于 MV/MZ）
    let mut engine_type = game.get_engine_type();
    if engine_type == EngineType::Other && game.engine_type == "nwjs" {
        if let Some(detected) = detect_engine_type(Path::new(&game.path)) {
            engine_type = EngineType::from_str(&detected);
        }
    }
    let nwjs_runtime_dir = if matches!(engine_type, EngineType::RpgMakerMV | EngineType::RpgMakerMZ)
    {
        let engine_service = state.engine_service.lock().await;
        let engine = if let Some(version) = game.runtime_version.as_deref() {
            engine_service.find_engine("nwjs", Some(version)).await?
        } else {
            engine_service.find_latest_engine_by_type("nwjs").await?
        };
        engine.map(|e| PathBuf::from(e.path))
    } else {
        None
    };

    if matches!(engine_type, EngineType::RpgMakerMV | EngineType::RpgMakerMZ)
        && nwjs_runtime_dir.is_none()
    {
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
        let enabled = db::get_setting(&state.pool, SETTING_BOTTLES_ENABLED)
            .await?
            .map(|v| v == "1")
            .unwrap_or(false);
        if !enabled {
            cfg.use_bottles = false;
            cfg.bottle_name = None;
        } else if cfg.use_bottles && cfg.bottle_name.as_deref().unwrap_or("").is_empty() {
            let default_bottle = db::get_setting(&state.pool, SETTING_BOTTLES_DEFAULT)
                .await?
                .and_then(|v| if v.trim().is_empty() { None } else { Some(v) });
            if let Some(name) = default_bottle {
                cfg.bottle_name = Some(name);
            } else {
                return Err("请选择 Bottles bottle".to_string());
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

/// 导入游戏目录
#[tauri::command]
pub async fn import_game_dir(
    input: ImportGameInput,
    state: State<'_, AppState>,
) -> Result<GameDto, String> {
    let service = state.game_service.lock().await;

    let executable_path = normalize_path(Path::new(&input.executable_path));
    let engine_type = input.engine_type;

    let exe_path = Path::new(&executable_path);
    if !exe_path.exists() || !exe_path.is_file() {
        return Err("可执行文件不存在".to_string());
    }

    let game_dir = exe_path
        .parent()
        .ok_or_else(|| "无法解析游戏目录".to_string())?;

    if is_nwjs_runtime_dir(game_dir) {
        return Err("检测到 NW.js 运行器目录，无法作为游戏导入".to_string());
    }

    let title = derive_game_title(exe_path, game_dir);

    let input = AddGameInput {
        title: Some(title),
        engine_type: engine_type.clone(),
        path: normalize_path(game_dir),
        game_type: None,
        detection_confidence: None,
        metadata_json: None,
        runtime_version: None,
    };

    let game = service.add_game(input).await?;

    let container_root = state.container_root.lock().await;
    let root = crate::services::path::canonicalize_path(Path::new(container_root.as_str()));
    drop(container_root);

    // 写入默认配置，记录入口文件
    let file_service = FileService::new();
    let config_path = file_service.game_config_path(&root, &game.profile_key);
    if let Err(e) = file_service.ensure_game_dirs(&root, &game.profile_key) {
        return Err(e);
    }
    let mut config = default_game_config(&game);
    config.entry_path = executable_path.clone();
    let _ = file_service.write_game_config(&config_path, &config);

    // 按优先级提取图标/封面
    let entry_exe = Some(exe_path);
    update_game_cover(
        &service,
        &file_service,
        &root,
        &game,
        &engine_type,
        game_dir,
        entry_exe,
        false,
    )
    .await;

    Ok(service.to_dto(game))
}

fn derive_game_title(exe_path: &Path, game_dir: &Path) -> String {
    let stem = exe_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .trim();
    let stem_lower = stem.to_lowercase();
    let invalid_names = ["game", "nw", "nwjs", "rpg_rt"];

    if !stem.is_empty() && !invalid_names.iter().any(|n| *n == stem_lower) {
        return stem.to_string();
    }

    let dir_name = game_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .trim();

    if !dir_name.is_empty() {
        return dir_name.to_string();
    }

    "未命名游戏".to_string()
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

    let container_root = state.container_root.lock().await;
    let root_path = crate::services::path::canonicalize_path(Path::new(container_root.as_str()));
    drop(container_root);

    let existing = service.get_all_games().await?;
    let mut existing_paths: HashSet<String> = existing
        .into_iter()
        .map(|g| normalize_path(Path::new(&g.path)))
        .collect();

    let root = PathBuf::from(input.root);
    if !root.exists() {
        return Err("扫描根目录不存在".to_string());
    }

    let total_dirs = count_dirs(&root, input.max_depth);
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

        if let Some(engine_type) = detect_engine_type(&dir) {
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
                    detection_confidence: None,
                    metadata_json: None,
                    runtime_version: None,
                };

                let game = service.add_game(input).await?;
                existing_paths.insert(path_str);
                imported += 1;

                let mut entry_exe: Option<PathBuf> = None;
                if EngineType::from_str(&engine_type) == EngineType::RenPy {
                    entry_exe = find_renpy_launch_script(&dir);
                    if let Some(entry) = entry_exe.as_deref() {
                        let config_path =
                            file_service.game_config_path(&root_path, &game.profile_key);
                        if file_service
                            .ensure_game_dirs(&root_path, &game.profile_key)
                            .is_ok()
                        {
                            let mut config = default_game_config(&game);
                            config.entry_path = normalize_path(entry);
                            let _ = file_service.write_game_config(&config_path, &config);
                        }
                    }
                }

                update_game_cover(
                    &service,
                    &file_service,
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
    crate::services::logger::log_scan_complete(imported as usize, skipped_existing as usize, duration_ms);

    Ok(ScanGamesResult {
        scanned_dirs,
        found_games,
        imported,
        skipped_existing,
    })
}

/// 按优先级更新封面图标
async fn update_game_cover(
    service: &GameService,
    file_service: &FileService,
    root: &Path,
    game: &Game,
    engine_type: &str,
    game_dir: &Path,
    entry_exe: Option<&Path>,
    force_extract: bool,
) -> bool {
    if !force_extract
        && let Some(existing) = resolve_existing_cover(file_service, root, game)
    {
        let _ = service
            .update_cover_path(&game.id, Some(existing.to_string_lossy().to_string()))
            .await;
        return true;
    }

    let saved = resolve_cover_for_game(
        file_service,
        root,
        &game.profile_key,
        engine_type,
        game_dir,
        entry_exe,
    );
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

fn resolve_cover_for_game(
    file_service: &FileService,
    root: &Path,
    profile_key: &str,
    engine_type: &str,
    game_dir: &Path,
    entry_exe: Option<&Path>,
) -> Option<PathBuf> {
    let engine = EngineType::from_str(engine_type);
    let exe_candidate = resolve_exe_candidate_for_icon(engine.clone(), game_dir, entry_exe);

    let save_image = |path: &Path| {
        file_service
            .save_cover_to_profile(root, profile_key, path)
            .ok()
    };
    let save_exe_icon =
        |path: &Path| file_service.save_exe_icon_to_profile(root, profile_key, path);

    match engine {
        EngineType::RpgMakerVX
        | EngineType::RpgMakerVXAce
        | EngineType::RpgMakerMV
        | EngineType::RpgMakerMZ => {
            if let Some(icon) = file_service.find_icon_dir_image(game_dir) {
                if let Some(saved) = save_image(&icon) {
                    return Some(saved);
                }
            }
            if let Some(exe) = exe_candidate.as_deref() {
                if let Some(saved) = save_exe_icon(exe) {
                    return Some(saved);
                }
            }
            if let Some(cover) = file_service.find_cover_image(game_dir) {
                return save_image(&cover);
            }
        }
        // Unity、Godot、RenPy 和其他引擎使用相同的封面提取策略
        EngineType::RenPy | EngineType::Unity | EngineType::Godot | EngineType::Other => {
            if let Some(exe) = exe_candidate.as_deref() {
                if let Some(saved) = save_exe_icon(exe) {
                    return Some(saved);
                }
            }
            if let Some(cover) = file_service.find_cover_image(game_dir) {
                return save_image(&cover);
            }
        }
    }

    None
}

fn resolve_existing_cover(file_service: &FileService, root: &Path, game: &Game) -> Option<PathBuf> {
    if let Some(current) = game.cover_path.as_deref() {
        let path = PathBuf::from(current);
        if path.exists() && path.is_file() {
            return Some(path);
        }
    }

    let config_path = file_service.game_config_path(root, &game.profile_key);
    if config_path.exists()
        && let Ok(config) = file_service.read_game_config(&config_path)
        && let Some(cover_file) = config.cover_file
    {
        let profile_dir = file_service.game_profile_dir(root, &game.profile_key);
        let cover_path = if Path::new(&cover_file).is_absolute() {
            PathBuf::from(&cover_file)
        } else {
            profile_dir.join(&cover_file)
        };
        if cover_path.exists() && cover_path.is_file() {
            return Some(cover_path);
        }
    }

    let profile_dir = file_service.game_profile_dir(root, &game.profile_key);
    if !profile_dir.exists() || !profile_dir.is_dir() {
        return None;
    }

    if let Ok(entries) = std::fs::read_dir(&profile_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if matches!(
                ext.as_str(),
                "png" | "jpg" | "jpeg" | "webp" | "bmp" | "ico"
            ) {
                return Some(path);
            }
        }
    }

    None
}

fn resolve_exe_candidate_for_icon(
    engine: EngineType,
    game_dir: &Path,
    entry_exe: Option<&Path>,
) -> Option<PathBuf> {
    let entry = entry_exe
        .filter(|p| p.exists() && p.is_file())
        .map(|p| p.to_path_buf());

    match engine {
        EngineType::RenPy => {
            if let Some(entry_path) = entry.as_deref() {
                if let Some(path) = resolve_renpy_icon_exe(entry_path, game_dir) {
                    return Some(path);
                }
            }
            find_executable_for_icon(EngineType::RenPy, game_dir)
        }
        _ => entry.or_else(|| find_executable_for_icon(engine, game_dir)),
    }
}

fn resolve_renpy_icon_exe(entry_exe: &Path, game_dir: &Path) -> Option<PathBuf> {
    let ext = entry_exe
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext == "exe" {
        return Some(entry_exe.to_path_buf());
    }

    if ext == "sh" {
        let sibling_exe = entry_exe.with_extension("exe");
        if sibling_exe.exists() && sibling_exe.is_file() {
            return Some(sibling_exe);
        }
    }

    let sibling_dir = entry_exe.parent().unwrap_or(game_dir);
    find_root_windows_exe(sibling_dir, &["renpy", "python"])
        .or_else(|| find_root_windows_exe(game_dir, &["renpy", "python"]))
}

fn find_executable_for_icon(engine: EngineType, game_dir: &Path) -> Option<PathBuf> {
    match engine {
        EngineType::RpgMakerVX
        | EngineType::RpgMakerVXAce
        | EngineType::RpgMakerMV
        | EngineType::RpgMakerMZ => find_executable_by_candidates(
            game_dir,
            &[
                "Game.exe",
                "Game",
                "RPG_RT.exe",
                "RPG_RT",
                "nw.exe",
                "nwjs.exe",
            ],
        )
        .or_else(|| find_root_windows_exe(game_dir, &[])),
        EngineType::RenPy => find_root_windows_exe(game_dir, &["renpy", "python"]),
        EngineType::Unity => find_unity_executable(game_dir),
        EngineType::Godot => find_godot_executable(game_dir),
        EngineType::Other => find_root_windows_exe(game_dir, &[]),
    }
}

/// 查找 Unity 游戏可执行文件
fn find_unity_executable(game_dir: &Path) -> Option<PathBuf> {
    // 首先尝试查找与 *_Data 目录同名的可执行文件
    if let Ok(entries) = std::fs::read_dir(game_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with("_Data") && entry.path().is_dir() {
                let exe_name = name.trim_end_matches("_Data");
                let exe_path = game_dir.join(format!("{}.exe", exe_name));
                if exe_path.exists() {
                    return Some(exe_path);
                }
                // Linux 可执行文件
                let linux_exe = game_dir.join(exe_name);
                if linux_exe.exists() && linux_exe.is_file() {
                    return Some(linux_exe);
                }
            }
        }
    }
    // Fallback: 查找任意 Windows 可执行文件
    find_root_windows_exe(game_dir, &["UnityCrashHandler", "CrashHandler"])
}

/// 查找 Godot 游戏可执行文件
fn find_godot_executable(game_dir: &Path) -> Option<PathBuf> {
    // 查找与 .pck 文件同名的可执行文件
    if let Ok(entries) = std::fs::read_dir(game_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".pck") {
                let exe_name = name.trim_end_matches(".pck");
                let exe_path = game_dir.join(format!("{}.exe", exe_name));
                if exe_path.exists() {
                    return Some(exe_path);
                }
                // Linux 可执行文件
                let linux_exe = game_dir.join(exe_name);
                if linux_exe.exists() && linux_exe.is_file() {
                    return Some(linux_exe);
                }
            }
        }
    }
    // Fallback: 查找任意 Windows 可执行文件
    find_root_windows_exe(game_dir, &[])
}

fn find_renpy_launch_script(game_dir: &Path) -> Option<PathBuf> {
    let dir_name = game_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut fallback: Option<PathBuf> = None;
    if let Ok(entries) = std::fs::read_dir(game_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if ext != "sh" {
                continue;
            }

            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();

            if !dir_name.is_empty() && stem == dir_name {
                return Some(path);
            }
            if stem == "renpy" {
                continue;
            }
            if fallback.is_none() {
                fallback = Some(path);
            }
        }
    }

    if fallback.is_some() {
        return fallback;
    }

    let renpy_sh = game_dir.join("renpy.sh");
    if renpy_sh.exists() && renpy_sh.is_file() {
        return Some(renpy_sh);
    }

    None
}

fn find_executable_by_candidates(game_dir: &Path, candidates: &[&str]) -> Option<PathBuf> {
    for candidate in candidates {
        let path = game_dir.join(candidate);
        if path.exists() && path.is_file() {
            return Some(path);
        }
    }
    None
}

fn find_root_windows_exe(game_dir: &Path, excluded: &[&str]) -> Option<PathBuf> {
    let dir_name = game_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mut fallback = None;

    if let Ok(entries) = std::fs::read_dir(game_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if ext != "exe" {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            if excluded.iter().any(|e| *e == stem) {
                continue;
            }
            if !dir_name.is_empty() && stem == dir_name {
                return Some(path);
            }
            if fallback.is_none() {
                fallback = Some(path);
            }
        }
    }

    fallback
}

fn resolve_entry_path_for_cover(game_path: &Path, entry_path: &str) -> Option<PathBuf> {
    let entry = entry_path.trim();
    if entry.is_empty() {
        return None;
    }

    let candidate = PathBuf::from(entry);
    let resolved = if candidate.is_absolute() {
        candidate
    } else {
        game_path.join(entry)
    };

    if resolved.exists() && resolved.is_file() {
        Some(resolved)
    } else {
        None
    }
}

fn fill_cover_from_config(
    file_service: &FileService,
    root: &Path,
    game: &Game,
    mut dto: GameDto,
) -> GameDto {
    if let Some(path) = dto.cover_path.as_deref() {
        let exists = Path::new(path).exists();
        if exists {
            return dto;
        }
        dto.cover_path = None;
    }

    let config_path = file_service.game_config_path(root, &game.profile_key);
    if !config_path.exists() {
        return dto;
    }

    let config = match file_service.read_game_config(&config_path) {
        Ok(config) => config,
        Err(_) => return dto,
    };
    let cover_file = config.cover_file.unwrap_or_default();
    if cover_file.trim().is_empty() {
        return dto;
    }

    let profile_dir = file_service.game_profile_dir(root, &game.profile_key);
    let cover_path = if Path::new(&cover_file).is_absolute() {
        PathBuf::from(&cover_file)
    } else {
        profile_dir.join(&cover_file)
    };

    if cover_path.exists() {
        dto.cover_path = Some(cover_path.to_string_lossy().to_string());
    }

    dto
}

/// 获取游戏设置（settings.toml）
#[tauri::command]
pub async fn get_game_settings(
    id: String,
    state: State<'_, AppState>,
) -> Result<GameConfig, String> {
    let service = state.game_service.lock().await;
    let game = service
        .get_game_by_id(&id)
        .await?
        .ok_or_else(|| format!("游戏不存在: {}", id))?;

    let container_root = state.container_root.lock().await;
    let root = crate::services::path::canonicalize_path(Path::new(container_root.as_str()));
    drop(container_root);

    let file_service = FileService::new();
    let config_path = file_service.game_config_path(&root, &game.profile_key);
    if config_path.exists() {
        let mut config = file_service.read_game_config(&config_path)?;
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
    let service = state.game_service.lock().await;
    let game = service
        .get_game_by_id(&id)
        .await?
        .ok_or_else(|| format!("游戏不存在: {}", id))?;

    let container_root = state.container_root.lock().await;
    let root = crate::services::path::canonicalize_path(Path::new(container_root.as_str()));
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
                PathBuf::from(&game.path).join(&cover_file)
            }
        };
        if cover_path.exists() {
            if let Ok(saved) =
                file_service.save_cover_to_profile(&root, &game.profile_key, &cover_path)
            {
                let _ = service
                    .update_cover_path(&game.id, Some(saved.to_string_lossy().to_string()))
                    .await;
            }
        }
    }

    file_service.write_game_config(&config_path, &config)
}

/// 重新提取图标/封面
#[tauri::command]
pub async fn refresh_game_cover(id: String, state: State<'_, AppState>) -> Result<GameDto, String> {
    let service = state.game_service.lock().await;
    let game = service
        .get_game_by_id(&id)
        .await?
        .ok_or_else(|| format!("游戏不存在: {}", id))?;

    let container_root = state.container_root.lock().await;
    let root = crate::services::path::canonicalize_path(Path::new(container_root.as_str()));
    drop(container_root);

    let file_service = FileService::new();
    let config_path = file_service.game_config_path(&root, &game.profile_key);
    let entry_exe = if config_path.exists() {
        file_service
            .read_game_config(&config_path)
            .ok()
            .and_then(|cfg| resolve_entry_path_for_cover(Path::new(&game.path), &cfg.entry_path))
    } else {
        None
    };

    let resolved_engine = normalize_engine_type(&game);
    let refreshed = update_game_cover(
        &service,
        &file_service,
        &root,
        &game,
        &resolved_engine,
        Path::new(&game.path),
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

/// 获取profile目录路径
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

fn count_dirs(root: &Path, max_depth: u32) -> u32 {
    let mut count = 0;
    let mut queue: VecDeque<(PathBuf, u32)> = VecDeque::new();
    queue.push_back((root.to_path_buf(), 0));

    while let Some((dir, depth)) = queue.pop_front() {
        count += 1;
        if depth >= max_depth {
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

    count.max(1)
}

fn detect_engine_type(path: &Path) -> Option<String> {
    detect_engine_with_score(path).map(|(engine, _)| engine)
}

fn detect_engine_with_score(path: &Path) -> Option<(String, i32)> {
    let mz_score = score_rpg_maker_mz(path);
    let mv_score = score_rpg_maker_mv(path);
    let renpy_score = score_renpy(path);
    let vxace_score = score_rpg_maker_vxace(path);
    let vx_score = score_rpg_maker_vx(path);
    let unity_score = score_unity(path);
    let godot_score = score_godot(path);

    let candidates = [
        ("rpgmakermz", mz_score, 1),
        ("rpgmakermv", mv_score, 2),
        ("renpy", renpy_score, 0),
        ("rpgmakervxace", vxace_score, 3),
        ("rpgmakervx", vx_score, 4),
        ("unity", unity_score, 5),
        ("godot", godot_score, 6),
    ];

    let mut best: Option<(&str, i32, i32)> = None;
    for (engine, score, priority) in candidates {
        if score < min_engine_score(engine) {
            continue;
        }
        match best {
            None => best = Some((engine, score, priority)),
            Some((_, best_score, best_priority)) => {
                if score > best_score || (score == best_score && priority < best_priority) {
                    best = Some((engine, score, priority));
                }
            }
        }
    }

    best.map(|(engine, score, _)| (engine.to_string(), (score * 16).min(100)))
}

fn min_engine_score(engine: &str) -> i32 {
    match engine {
        "rpgmakermz" | "rpgmakermv" => 4,
        "renpy" => 4,
        "rpgmakervxace" | "rpgmakervx" => 5,
        "unity" => 4,
        "godot" => 4,
        _ => 0,
    }
}

fn score_rpg_maker_mz(path: &Path) -> i32 {
    let mut score = 0;
    let base = path;
    let www = path.join("www");
    if has_mz_core(base) || has_mz_core(&www) {
        score += 3;
    }
    if has_rpg_data(base) || has_rpg_data(&www) {
        score += 2;
    }
    if has_package_json(base) || has_package_json(&www) {
        score += 1;
    }
    score
}

fn score_rpg_maker_mv(path: &Path) -> i32 {
    let mut score = 0;
    let base = path;
    let www = path.join("www");
    if has_mv_core(base) || has_mv_core(&www) {
        score += 3;
    }
    if has_rpg_data(base) || has_rpg_data(&www) {
        score += 2;
    }
    if has_package_json(base) || has_package_json(&www) {
        score += 1;
    }
    score
}

fn score_renpy(path: &Path) -> i32 {
    let mut score = 0;
    if path.join("renpy").is_dir()
        || path.join("renpy.sh").exists()
        || path.join("renpy.exe").exists()
    {
        score += 3;
    }

    let game_dir = path.join("game");
    if game_dir.is_dir() {
        score += 1;
        if has_renpy_scripts(&game_dir) {
            score += 3;
        }
        if has_renpy_marker_files(&game_dir) {
            score += 1;
        }
    }

    if has_renpy_lib(path) {
        score += 1;
    }

    score
}

fn score_rpg_maker_vxace(path: &Path) -> i32 {
    let mut score = 0;
    if has_vx_executable(path) {
        score += 2;
    }
    if has_rgss_dll(path, "RGSS3") {
        score += 3;
    }
    if path.join("Game.ini").exists() {
        score += 1;
    }
    score
}

fn score_rpg_maker_vx(path: &Path) -> i32 {
    let mut score = 0;
    if has_vx_executable(path) {
        score += 2;
    }
    if has_rgss_dll(path, "RGSS2") || has_rgss_dll(path, "RGSS1") {
        score += 3;
    }
    if path.join("Game.ini").exists() {
        score += 1;
    }
    score
}

/// Unity 游戏检测评分
///
/// 检测特征:
/// - UnityPlayer.dll (Windows)
/// - *_Data 目录 (包含 Managed, Resources 等)
/// - MonoBleedingEdge 目录
/// - globalgamemanagers 文件
fn score_unity(path: &Path) -> i32 {
    let mut score = 0;

    // 检测 UnityPlayer.dll (Windows)
    if path.join("UnityPlayer.dll").exists() {
        score += 3;
    }

    // 检测 *_Data 目录
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with("_Data") && entry.path().is_dir() {
                let data_dir = entry.path();
                score += 2;

                // 检测 Data 目录中的特征文件
                if data_dir.join("Managed").is_dir() {
                    score += 1;
                }
                if data_dir.join("globalgamemanagers").exists()
                    || data_dir.join("mainData").exists()
                {
                    score += 1;
                }
                break;
            }
        }
    }

    // 检测 MonoBleedingEdge 目录
    if path.join("MonoBleedingEdge").is_dir() {
        score += 1;
    }

    // 检测 IL2CPP 后端
    if path.join("GameAssembly.dll").exists() {
        score += 2;
    }

    score
}

/// Godot 游戏检测评分
///
/// 检测特征:
/// - .pck 文件 (游戏资源包)
/// - godot 相关 DLL/库文件
/// - .import 目录
fn score_godot(path: &Path) -> i32 {
    let mut score = 0;

    // 检测 .pck 文件
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.ends_with(".pck") {
                score += 3;
                break;
            }
        }
    }

    // 检测 Godot 库文件
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.contains("godot") && (name.ends_with(".dll") || name.ends_with(".so")) {
                score += 2;
                break;
            }
        }
    }

    // 检测 .import 目录 (编辑器项目)
    if path.join(".import").is_dir() || path.join(".godot").is_dir() {
        score += 1;
    }

    // 检测 project.godot 文件 (编辑器项目)
    if path.join("project.godot").exists() {
        score += 2;
    }

    score
}

fn has_mz_core(base: &Path) -> bool {
    let js = base.join("js");
    js.join("rmmz_core.js").exists() || js.join("rmmz_managers.js").exists()
}

fn has_mv_core(base: &Path) -> bool {
    let js = base.join("js");
    js.join("rpg_core.js").exists() || js.join("rpg_managers.js").exists()
}

fn has_rpg_data(base: &Path) -> bool {
    base.join("data").join("System.json").exists()
}

fn has_package_json(base: &Path) -> bool {
    base.join("package.json").exists()
}

fn has_vx_executable(path: &Path) -> bool {
    ["Game.exe", "Game", "RPG_RT.exe", "RPG_RT"]
        .iter()
        .any(|name| path.join(name).exists())
}

fn has_rgss_dll(path: &Path, prefix: &str) -> bool {
    let prefix = prefix.to_lowercase();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy().to_lowercase();
            if name.starts_with(&prefix) && name.ends_with(".dll") {
                return true;
            }
        }
    }
    false
}

fn has_renpy_marker_files(game_dir: &Path) -> bool {
    let marker_files = ["script.rpy", "options.rpy", "gui.rpy", "screens.rpy"];
    marker_files.iter().any(|name| game_dir.join(name).exists())
}

fn has_renpy_scripts(game_dir: &Path) -> bool {
    if let Ok(entries) = std::fs::read_dir(game_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension()
                .and_then(|e| e.to_str())
                .map(|e| matches!(e.to_lowercase().as_str(), "rpy" | "rpyc"))
                == Some(true)
            {
                return true;
            }
        }
    }
    false
}

fn has_renpy_lib(path: &Path) -> bool {
    let lib_dir = path.join("lib");
    if !lib_dir.is_dir() {
        return false;
    }

    if let Ok(entries) = std::fs::read_dir(&lib_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.starts_with("py") || name.contains("python") {
                return true;
            }
        }
    }

    false
}

fn default_game_config(game: &Game) -> GameConfig {
    GameConfig {
        engine_type: normalize_engine_type(game),
        entry_path: game.path.clone(),
        runtime_version: game.runtime_version.clone(),
        args: Vec::new(),
        sandbox_home: true,
        use_bottles: false,
        bottle_name: None,
        cover_file: None,
    }
}

fn normalize_engine_type(game: &Game) -> String {
    if game.engine_type == "nwjs" {
        if let Some(detected) = detect_engine_type(Path::new(&game.path)) {
            return detected;
        }
    }
    game.engine_type.clone()
}

fn is_nwjs_runtime_dir(path: &Path) -> bool {
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

fn normalize_path(path: &Path) -> String {
    crate::services::path::canonicalize_path(path)
        .to_string_lossy()
        .to_string()
}
