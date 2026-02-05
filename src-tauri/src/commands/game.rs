use crate::models::*;
use crate::services::{EngineService, FileService, GameService, LauncherService};
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
    pub container_root: Arc<Mutex<String>>,
}

/// 获取所有游戏
#[tauri::command]
pub async fn get_games(state: State<'_, AppState>) -> Result<Vec<GameDto>, String> {
    let service = state.game_service.lock().await;
    let games = service.get_all_games().await?;
    let dtos = games.into_iter().map(|g| service.to_dto(g)).collect();
    Ok(dtos)
}

/// 获取单个游戏
#[tauri::command]
pub async fn get_game(id: String, state: State<'_, AppState>) -> Result<Option<GameDto>, String> {
    let service = state.game_service.lock().await;
    let game = service.get_game_by_id(&id).await?;
    Ok(game.map(|g| service.to_dto(g)))
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
    let config = if config_path.exists() {
        Some(file_service.read_game_config(&config_path)?)
    } else {
        None
    };

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

    let import_path = normalize_path(Path::new(&input.path));
    let engine_type = input.engine_type;

    if !Path::new(&import_path).exists() {
        return Err("游戏路径不存在".to_string());
    }

    if is_nwjs_runtime_dir(Path::new(&import_path)) {
        return Err("检测到 NW.js 运行器目录，无法作为游戏导入".to_string());
    }

    let input = AddGameInput {
        title: None,
        engine_type,
        path: import_path.clone(),
        runtime_version: None,
    };

    let game = service.add_game(input).await?;

    let container_root = state.container_root.lock().await;
    let root = crate::services::path::canonicalize_path(Path::new(container_root.as_str()));
    drop(container_root);

    // 尝试自动发现封面
    let file_service = FileService::new();
    let cover_path = file_service.find_cover_image(Path::new(&import_path));
    if let Some(cover) = cover_path {
        if let Ok(saved) = file_service.save_cover_to_profile(&root, &game.profile_key, &cover) {
            let _ = service
                .update_cover_path(&game.id, Some(saved.to_string_lossy().to_string()))
                .await;
        }
    }

    Ok(service.to_dto(game))
}

/// 扫描游戏目录
#[tauri::command]
pub async fn scan_games(
    input: ScanGamesInput,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<ScanGamesResult, String> {
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
                    engine_type,
                    path: path_str.clone(),
                    runtime_version: None,
                };

                let game = service.add_game(input).await?;
                existing_paths.insert(path_str);
                imported += 1;

                if let Some(cover) = file_service.find_cover_image(&dir) {
                    if let Ok(saved) =
                        file_service.save_cover_to_profile(&root_path, &game.profile_key, &cover)
                    {
                        let _ = service
                            .update_cover_path(&game.id, Some(saved.to_string_lossy().to_string()))
                            .await;
                    }
                }
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

    Ok(ScanGamesResult {
        scanned_dirs,
        found_games,
        imported,
        skipped_existing,
    })
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
        let cover_path = if Path::new(&cover_file).is_absolute() {
            PathBuf::from(cover_file)
        } else {
            PathBuf::from(&game.path).join(cover_file)
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
    let mz_score = score_rpg_maker_mz(path);
    let mv_score = score_rpg_maker_mv(path);
    let renpy_score = score_renpy(path);
    let vxace_score = score_rpg_maker_vxace(path);
    let vx_score = score_rpg_maker_vx(path);

    let candidates = [
        ("rpgmakermz", mz_score, 1),
        ("rpgmakermv", mv_score, 2),
        ("renpy", renpy_score, 0),
        ("rpgmakervxace", vxace_score, 3),
        ("rpgmakervx", vx_score, 4),
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

    best.map(|(engine, _, _)| engine.to_string())
}

fn min_engine_score(engine: &str) -> i32 {
    match engine {
        "rpgmakermz" | "rpgmakermv" => 4,
        "renpy" => 4,
        "rpgmakervxace" | "rpgmakervx" => 5,
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
