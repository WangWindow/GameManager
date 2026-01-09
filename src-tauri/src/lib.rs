use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Emitter;
use tauri::Manager;
use tauri::menu::MenuBuilder;
use tauri::tray::TrayIconBuilder;
use uuid::Uuid;

mod nwjs;

const SETTING_CONTAINER_ROOT: &str = "container_root";

#[derive(Clone)]
struct AppState {
    pool: SqlitePool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    container_root: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GameEntryDto {
    id: String,
    title: String,
    engine_type: String,
    path: String,
    path_valid: bool,
    runtime_version: Option<String>,
    cover_path: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddGameInput {
    title: Option<String>,
    engine_type: String,
    path: String,
    runtime_version: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetContainerRootInput {
    container_root: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameToml {
    engine_type: String,
    entry_path: String,
    runtime_version: Option<String>,
    args: Vec<String>,
    #[serde(default = "default_true")]
    sandbox_home: bool,
    #[serde(default)]
    cover_file: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LaunchGameResult {
    pid: u32,
}

fn now_unix_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn default_container_root(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to resolve app data dir: {e}"))?;
    Ok(app_data_dir.join("containers"))
}

async fn get_setting(pool: &SqlitePool, key: &str) -> Result<Option<String>, String> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("db error: {e}"))?;
    Ok(row.map(|r| r.0))
}

async fn set_setting(pool: &SqlitePool, key: &str, value: &str) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO settings(key, value) VALUES(?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await
    .map_err(|e| format!("db error: {e}"))?;
    Ok(())
}

async fn resolve_container_root(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
) -> Result<PathBuf, String> {
    if let Some(value) = get_setting(pool, SETTING_CONTAINER_ROOT).await? {
        return Ok(PathBuf::from(value));
    }
    Ok(default_container_root(app)?)
}

fn ensure_dir(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path)
        .map_err(|e| format!("failed to create dir {}: {e}", path.display()))
}

fn game_profile_dir(container_root: &Path, game_id: &str) -> PathBuf {
    container_root.join("profiles").join(game_id)
}

fn game_user_data_dir(container_root: &Path, game_id: &str) -> PathBuf {
    game_profile_dir(container_root, game_id).join("User Data")
}

fn game_toml_path(container_root: &Path, game_id: &str) -> PathBuf {
    game_profile_dir(container_root, game_id).join("settings.toml")
}

fn pick_icon_dirs(game_dir: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();

    // 1) game 根目录下的 icon/icons
    for name in ["icon", "icons"] {
        let p = game_dir.join(name);
        if p.is_dir() {
            dirs.push(p);
        }
    }

    // 2) 兼容：www/icon 或 www/icons
    let www = game_dir.join("www");
    if www.is_dir() {
        for name in ["icon", "icons"] {
            let p = www.join(name);
            if p.is_dir() {
                dirs.push(p);
            }
        }
    }

    // 3) 同时也扫一遍根目录，兼容大小写（Icon/ICONS）
    if let Ok(entries) = std::fs::read_dir(game_dir) {
        for e in entries.flatten() {
            let Ok(ty) = e.file_type() else {
                continue;
            };
            if !ty.is_dir() {
                continue;
            }
            let p = e.path();
            let Some(name) = p.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            let lower = name.to_ascii_lowercase();
            if lower == "icon" || lower == "icons" {
                if !dirs.iter().any(|d| d == &p) {
                    dirs.push(p);
                }
            }
        }
    }

    dirs
}

fn is_supported_image_ext(ext: &str) -> bool {
    matches!(ext, "png" | "jpg" | "jpeg" | "webp")
}

fn score_cover_candidate(path: &Path) -> u64 {
    let mut score: u64 = 0;
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if name.contains("cover") {
        score += 300;
    }
    if name.contains("poster") {
        score += 200;
    }
    if name.contains("banner") {
        score += 180;
    }
    if name.contains("title") {
        score += 120;
    }
    if name.contains("icon") {
        score += 80;
    }

    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        match ext.to_ascii_lowercase().as_str() {
            "png" => score += 30,
            "webp" => score += 25,
            "jpg" | "jpeg" => score += 20,
            _ => {}
        }
    }

    if let Ok(meta) = std::fs::metadata(path) {
        if meta.is_file() {
            // 文件越大越可能是封面图；上限避免极端值影响
            score += (meta.len() / 1024).min(1024);
        }
    }

    score
}

fn find_cover_candidate_from_icons(game_dir: &Path) -> Option<PathBuf> {
    let icon_dirs = pick_icon_dirs(game_dir);
    if icon_dirs.is_empty() {
        return None;
    }

    let mut best: Option<(u64, PathBuf)> = None;
    for dir in icon_dirs {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            let Ok(ty) = e.file_type() else {
                continue;
            };
            if !ty.is_file() {
                continue;
            }
            let Some(ext) = p.extension().and_then(|s| s.to_str()) else {
                continue;
            };
            if !is_supported_image_ext(&ext.to_ascii_lowercase()) {
                continue;
            }

            let score = score_cover_candidate(&p);
            match &best {
                None => best = Some((score, p)),
                Some((best_score, _)) if score > *best_score => best = Some((score, p)),
                _ => {}
            }
        }
    }

    best.map(|(_, p)| p)
}

fn write_cover_file(
    container_root: &Path,
    game_id: &str,
    src_path: &Path,
) -> Result<String, String> {
    let profile_dir = game_profile_dir(container_root, game_id);
    ensure_dir(&profile_dir)?;

    let ext = src_path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .ok_or_else(|| "cover source has no extension".to_string())?;
    if !is_supported_image_ext(&ext) {
        return Err(format!("unsupported cover extension: {ext}"));
    }

    let cover_file = format!("cover.{ext}");
    let dest = profile_dir.join(&cover_file);
    std::fs::copy(src_path, &dest)
        .map_err(|e| format!("failed to copy cover to {}: {e}", dest.display()))?;
    Ok(cover_file)
}

fn resolve_cover_path(container_root: &Path, game_id: &str) -> Option<PathBuf> {
    let toml_path = game_toml_path(container_root, game_id);
    if toml_path.exists() {
        if let Ok(cfg) = read_game_toml(&toml_path) {
            if let Some(file) = cfg.cover_file {
                let p = game_profile_dir(container_root, game_id).join(file);
                if p.exists() {
                    return Some(p);
                }
            }
        }
    }

    // 兼容：如果用户手动放了 cover.* 但没写入 settings.toml
    let profile_dir = game_profile_dir(container_root, game_id);
    for ext in ["png", "webp", "jpg", "jpeg"] {
        let p = profile_dir.join(format!("cover.{ext}"));
        if p.exists() {
            return Some(p);
        }
    }

    None
}

fn write_game_toml(path: &Path, config: &GameToml) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        ensure_dir(dir)?;
    }
    let content =
        toml::to_string_pretty(config).map_err(|e| format!("toml serialize error: {e}"))?;
    std::fs::write(path, content).map_err(|e| format!("failed to write {}: {e}", path.display()))
}

fn read_game_toml(path: &Path) -> Result<GameToml, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    toml::from_str(&content).map_err(|e| format!("toml parse error: {e}"))
}

fn nwjs_executable_path(install_dir: &Path) -> Result<PathBuf, String> {
    // 中文说明：不同平台下 NW.js 可执行文件路径不同。
    let os = std::env::consts::OS;
    let exe = match os {
        "windows" => install_dir.join("nw.exe"),
        "linux" => install_dir.join("nw"),
        "macos" => install_dir
            .join("nwjs.app")
            .join("Contents")
            .join("MacOS")
            .join("nwjs"),
        _ => {
            return Err(format!("unsupported OS for launch: {os}"));
        }
    };

    if !exe.exists() {
        return Err(format!("NW.js executable not found: {}", exe.display()));
    }

    Ok(exe)
}

fn ensure_user_data_arg(args: &mut Vec<String>, user_data_dir: &Path) {
    // 中文说明：如果 settings.toml 没配置 --user-data-dir，则自动补上默认容器路径。
    let has = args
        .iter()
        .any(|a| a == "--user-data-dir" || a.starts_with("--user-data-dir="));
    if !has {
        args.push(format!(
            "--user-data-dir={}",
            user_data_dir.to_string_lossy()
        ));
    }
}

fn runtime_install_dir(
    app: &tauri::AppHandle,
    version: &str,
    flavor: &str,
    target: &str,
) -> Result<PathBuf, String> {
    // 中文说明：与 nwjs.rs 的安装目录保持一致：appData/runtimes/nwjs/<version>/<flavor>/<target>
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to resolve app data dir: {e}"))?;
    Ok(app_data_dir
        .join("runtimes")
        .join("nwjs")
        .join(version)
        .join(flavor)
        .join(target))
}

async fn find_game_by_path(pool: &SqlitePool, path: &str) -> Result<Option<GameEntryDto>, String> {
    let row: Option<(String, String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, title, engine_type, path, runtime_version FROM games WHERE path = ? LIMIT 1",
    )
    .bind(path)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("db error: {e}"))?;

    Ok(row.map(
        |(id, title, engine_type, path, runtime_version)| GameEntryDto {
            id,
            title,
            engine_type,
            path_valid: Path::new(&path).exists(),
            path,
            runtime_version,
            cover_path: None,
        },
    ))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetGameCoverInput {
    game_id: String,
    source_path: String,
}

fn detect_rpg_maker_engine(game_dir: &Path) -> Result<Option<String>, String> {
    let www = game_dir.join("www");
    let index = www.join("index.html");
    let pkg = game_dir.join("package.json");
    if !(index.exists() && pkg.exists()) {
        return Ok(None);
    }

    let js_dir = www.join("js");
    if js_dir.join("rmmz_core.js").exists() {
        return Ok(Some("rmmz".to_string()));
    }
    if js_dir.join("rpg_core.js").exists() {
        return Ok(Some("rmmv".to_string()));
    }

    // It's still a valid NW.js RPG Maker layout but unknown engine.
    Ok(Some("unknown".to_string()))
}

fn read_game_title(game_dir: &Path) -> String {
    // Per requirement: default display name uses the folder name.
    game_dir
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "未命名游戏".to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GameContainerInfo {
    profile_dir: String,
    user_data_dir: String,
    settings_toml: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GameLaunchConfigDto {
    args: Vec<String>,
    sandbox_home: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateGameLaunchConfigInput {
    game_id: String,
    args: Vec<String>,
    sandbox_home: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CleanupContainersResult {
    deleted: u64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ScanProgress {
    task_id: String,
    label: String,
    progress: u8,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ScanResult {
    task_id: String,
    scanned_dirs: u64,
    found_games: u64,
    imported: u64,
    skipped_existing: u64,
}

async fn import_game_dir_internal(
    pool: &SqlitePool,
    app: &tauri::AppHandle,
    path: &str,
) -> Result<GameEntryDto, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("path is empty".to_string());
    }
    let game_dir = PathBuf::from(trimmed);
    if !game_dir.is_dir() {
        return Err(format!("not a directory: {}", game_dir.display()));
    }

    if let Some(existing) = find_game_by_path(pool, trimmed).await? {
        return Ok(existing);
    }

    let engine_type = detect_rpg_maker_engine(&game_dir)?.ok_or_else(|| {
        "not a RPG Maker NW.js directory (missing www/index.html or package.json)".to_string()
    })?;
    let title = read_game_title(&game_dir);

    let id = Uuid::new_v4().to_string();
    let now = now_unix_ms();
    sqlx::query(
        "INSERT INTO games(id, title, engine_type, path, runtime_version, created_at, updated_at) VALUES(?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&title)
    .bind(&engine_type)
    .bind(trimmed)
    .bind(Option::<String>::None)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| format!("db error: {e}"))?;

    let container_root = resolve_container_root(app, pool).await?;
    ensure_dir(&container_root)?;
    let toml_path = game_toml_path(&container_root, &id);
    let mut config = GameToml {
        engine_type: engine_type.clone(),
        entry_path: "www/index.html".to_string(),
        runtime_version: None,
        args: vec![],
        sandbox_home: true,
        cover_file: None,
    };

    // 中文说明：导入时尝试从 icon/ 或 icons/ 目录里找一张图片当封面，并拷贝进容器目录。
    if let Some(src) = find_cover_candidate_from_icons(&game_dir) {
        if let Ok(cover_file) = write_cover_file(&container_root, &id, &src) {
            config.cover_file = Some(cover_file);
        }
    }
    write_game_toml(&toml_path, &config)?;

    let cover_path =
        resolve_cover_path(&container_root, &id).map(|p| p.to_string_lossy().to_string());

    Ok(GameEntryDto {
        id,
        title,
        engine_type,
        path: trimmed.to_string(),
        path_valid: true,
        runtime_version: None,
        cover_path,
    })
}

#[tauri::command]
async fn update_game_title(
    state: tauri::State<'_, AppState>,
    game_id: String,
    title: String,
) -> Result<(), String> {
    let t = title.trim();
    if t.is_empty() {
        return Err("title is empty".to_string());
    }
    let now = now_unix_ms();
    sqlx::query("UPDATE games SET title = ?, updated_at = ? WHERE id = ?")
        .bind(t)
        .bind(now)
        .bind(&game_id)
        .execute(&state.pool)
        .await
        .map_err(|e| format!("db error: {e}"))?;
    Ok(())
}

#[tauri::command]
async fn get_game_container_info(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    game_id: String,
) -> Result<GameContainerInfo, String> {
    let container_root = resolve_container_root(&app, &state.pool).await?;
    let profile_dir = game_profile_dir(&container_root, &game_id);
    let user_data_dir = game_user_data_dir(&container_root, &game_id);
    let settings_toml = game_toml_path(&container_root, &game_id);

    Ok(GameContainerInfo {
        profile_dir: profile_dir.to_string_lossy().to_string(),
        user_data_dir: user_data_dir.to_string_lossy().to_string(),
        settings_toml: settings_toml.to_string_lossy().to_string(),
    })
}

#[tauri::command]
async fn get_game_launch_config(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    game_id: String,
) -> Result<GameLaunchConfigDto, String> {
    let container_root = resolve_container_root(&app, &state.pool).await?;
    ensure_dir(&container_root)?;
    let toml_path = game_toml_path(&container_root, &game_id);

    let cfg = if toml_path.exists() {
        read_game_toml(&toml_path)?
    } else {
        GameToml {
            engine_type: "unknown".to_string(),
            entry_path: "www/index.html".to_string(),
            runtime_version: None,
            args: vec![],
            sandbox_home: true,
            cover_file: None,
        }
    };

    Ok(GameLaunchConfigDto {
        args: cfg.args,
        sandbox_home: cfg.sandbox_home,
    })
}

#[tauri::command]
async fn update_game_launch_config(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    input: UpdateGameLaunchConfigInput,
) -> Result<(), String> {
    let container_root = resolve_container_root(&app, &state.pool).await?;
    ensure_dir(&container_root)?;
    let toml_path = game_toml_path(&container_root, &input.game_id);

    let mut cfg = if toml_path.exists() {
        read_game_toml(&toml_path)?
    } else {
        GameToml {
            engine_type: "unknown".to_string(),
            entry_path: "www/index.html".to_string(),
            runtime_version: None,
            args: vec![],
            sandbox_home: true,
            cover_file: None,
        }
    };

    cfg.args = input
        .args
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    cfg.sandbox_home = input.sandbox_home;

    // sandbox_home=true 时，HOME/XDG 已重定向到容器目录，默认无需 --user-data-dir。
    // 当 sandbox_home=false 时，仍强制补齐，避免污染真实 ~/.config。
    if !cfg.sandbox_home {
        let user_data_dir = game_user_data_dir(&container_root, &input.game_id);
        ensure_dir(&user_data_dir)?;
        ensure_user_data_arg(&mut cfg.args, &user_data_dir);
    }

    write_game_toml(&toml_path, &cfg)
}

#[tauri::command]
async fn set_game_cover(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    input: SetGameCoverInput,
) -> Result<(), String> {
    let container_root = resolve_container_root(&app, &state.pool).await?;
    ensure_dir(&container_root)?;
    let toml_path = game_toml_path(&container_root, &input.game_id);

    let mut cfg = if toml_path.exists() {
        read_game_toml(&toml_path)?
    } else {
        GameToml {
            engine_type: "unknown".to_string(),
            entry_path: "www/index.html".to_string(),
            runtime_version: None,
            args: vec![],
            sandbox_home: true,
            cover_file: None,
        }
    };

    let src = PathBuf::from(input.source_path);
    if !src.is_file() {
        return Err(format!("cover source not found: {}", src.display()));
    }

    let cover_file = write_cover_file(&container_root, &input.game_id, &src)?;
    cfg.cover_file = Some(cover_file);
    write_game_toml(&toml_path, &cfg)
}

#[tauri::command]
async fn clear_game_cover(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    game_id: String,
) -> Result<(), String> {
    let container_root = resolve_container_root(&app, &state.pool).await?;
    ensure_dir(&container_root)?;
    let toml_path = game_toml_path(&container_root, &game_id);

    let mut cfg = if toml_path.exists() {
        read_game_toml(&toml_path)?
    } else {
        GameToml {
            engine_type: "unknown".to_string(),
            entry_path: "www/index.html".to_string(),
            runtime_version: None,
            args: vec![],
            sandbox_home: true,
            cover_file: None,
        }
    };

    if let Some(file) = cfg.cover_file.take() {
        let p = game_profile_dir(&container_root, &game_id).join(file);
        if p.exists() {
            let _ = std::fs::remove_file(&p);
        }
    }

    write_game_toml(&toml_path, &cfg)
}

#[tauri::command]
async fn cleanup_unused_containers(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<CleanupContainersResult, String> {
    let container_root = resolve_container_root(&app, &state.pool).await?;
    let profiles_dir = container_root.join("profiles");
    if !profiles_dir.exists() {
        return Ok(CleanupContainersResult { deleted: 0 });
    }

    let rows: Vec<(String,)> = sqlx::query_as("SELECT id FROM games")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| format!("db error: {e}"))?;
    let keep: std::collections::HashSet<String> = rows.into_iter().map(|r| r.0).collect();

    let mut deleted: u64 = 0;
    for entry in std::fs::read_dir(&profiles_dir)
        .map_err(|e| format!("read_dir error: {e}"))?
        .flatten()
    {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if keep.contains(name) {
            continue;
        }
        std::fs::remove_dir_all(&path)
            .map_err(|e| format!("failed to remove {}: {e}", path.display()))?;
        deleted += 1;
    }

    Ok(CleanupContainersResult { deleted })
}

#[tauri::command]
async fn import_game_dir(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    path: String,
) -> Result<GameEntryDto, String> {
    import_game_dir_internal(&state.pool, &app, &path).await
}

#[tauri::command]
async fn scan_games(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    root: String,
    max_depth: Option<u8>,
) -> Result<ScanResult, String> {
    let task_id = Uuid::new_v4().to_string();
    let root = root.trim().to_string();
    if root.is_empty() {
        return Err("root is empty".to_string());
    }

    let root_path = PathBuf::from(&root);
    if !root_path.is_dir() {
        return Err(format!("not a directory: {}", root_path.display()));
    }

    let depth = max_depth.unwrap_or(6).min(20) as usize;
    let ignore_names: std::collections::HashSet<&'static str> = [
        ".git",
        "node_modules",
        "dist",
        "target",
        "runtimes",
        "containers",
        ".cache",
    ]
    .into_iter()
    .collect();

    let (candidates, scanned_dirs) = tokio::task::spawn_blocking(move || {
        let mut scanned: u64 = 0;
        let mut found: Vec<PathBuf> = Vec::new();
        let mut stack: Vec<(PathBuf, usize)> = vec![(root_path, 0)];

        while let Some((dir, d)) = stack.pop() {
            scanned += 1;

            // candidate check
            if let Ok(Some(_)) = detect_rpg_maker_engine(&dir) {
                found.push(dir);
                continue;
            }

            if d >= depth {
                continue;
            }

            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };

            for e in entries.flatten() {
                let Ok(ty) = e.file_type() else {
                    continue;
                };
                if !ty.is_dir() {
                    continue;
                }

                let p = e.path();
                let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if name.starts_with('.') || ignore_names.contains(name) {
                    continue;
                }
                stack.push((p, d + 1));
            }
        }

        (found, scanned)
    })
    .await
    .map_err(|e| format!("scan task join error: {e}"))?;

    let found_games = candidates.len() as u64;
    let mut imported: u64 = 0;
    let mut skipped_existing: u64 = 0;

    for (i, dir) in candidates.into_iter().enumerate() {
        let p = dir.to_string_lossy().to_string();
        if find_game_by_path(&state.pool, &p).await?.is_some() {
            skipped_existing += 1;
        } else {
            if import_game_dir_internal(&state.pool, &app, &p)
                .await
                .is_ok()
            {
                imported += 1;
            }
        }

        let progress = if found_games == 0 {
            0
        } else {
            (((i + 1) as f64 / found_games as f64) * 100.0).floor() as u8
        };

        let label = format!(
            "扫描中：发现 {found_games} 个，已处理 {} 个（新增 {imported}）",
            i + 1
        );
        let _ = app.emit(
            "scan_progress",
            ScanProgress {
                task_id: task_id.clone(),
                label,
                progress: progress.min(100),
            },
        );
    }

    let _ = app.emit(
        "scan_progress",
        ScanProgress {
            task_id: task_id.clone(),
            label: format!(
                "扫描完成：新增 {imported}，已存在 {skipped_existing}（共发现 {found_games}）"
            ),
            progress: 100,
        },
    );

    Ok(ScanResult {
        task_id,
        scanned_dirs,
        found_games,
        imported,
        skipped_existing,
    })
}

fn setup_tray(app: &tauri::AppHandle) -> Result<(), String> {
    let menu = MenuBuilder::new(app)
        .text("toggle_window", "显示/隐藏")
        .separator()
        .text("quit", "退出")
        .build()
        .map_err(|e| format!("failed to build tray menu: {e}"))?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| "missing default window icon".to_string())?;

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .tooltip("GameManager")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "toggle_window" => {
                if let Some(window) = app.get_webview_window("main") {
                    let visible = window.is_visible().unwrap_or(true);
                    if visible {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)
        .map_err(|e| format!("failed to build tray icon: {e}"))?;

    Ok(())
}

#[tauri::command]
async fn get_app_settings(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<AppSettings, String> {
    let container_root = resolve_container_root(&app, &state.pool).await?;
    ensure_dir(&container_root)?;
    Ok(AppSettings {
        container_root: container_root.to_string_lossy().to_string(),
    })
}

#[tauri::command]
async fn set_container_root(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    input: SetContainerRootInput,
) -> Result<AppSettings, String> {
    let resolved = if input.container_root.trim().is_empty() {
        default_container_root(&app)?
    } else {
        PathBuf::from(input.container_root.trim())
    };
    ensure_dir(&resolved)?;
    set_setting(
        &state.pool,
        SETTING_CONTAINER_ROOT,
        &resolved.to_string_lossy(),
    )
    .await?;
    Ok(AppSettings {
        container_root: resolved.to_string_lossy().to_string(),
    })
}

#[tauri::command]
async fn list_games(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<Vec<GameEntryDto>, String> {
    let rows: Vec<(String, String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, title, engine_type, path, runtime_version FROM games ORDER BY title COLLATE NOCASE",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| format!("db error: {e}"))?;

    // 为了在列表里显示封面：读取每个 gameId 对应的容器 settings.toml / cover.*。
    // 这里按行同步读取（文件很小）；后续如有性能压力可改成缓存或批量策略。
    let container_root = resolve_container_root(&app, &state.pool).await.ok();

    Ok(rows
        .into_iter()
        .map(|(id, title, engine_type, path, runtime_version)| {
            let path_valid = Path::new(&path).exists();
            let cover_path = container_root
                .as_ref()
                .and_then(|root| resolve_cover_path(root, &id))
                .map(|p| p.to_string_lossy().to_string());
            GameEntryDto {
                id,
                title,
                engine_type,
                path,
                path_valid,
                runtime_version,
                cover_path,
            }
        })
        .collect())
}

#[tauri::command]
async fn add_game(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    input: AddGameInput,
) -> Result<GameEntryDto, String> {
    let id = Uuid::new_v4().to_string();
    let title = input
        .title
        .and_then(|t| {
            let trimmed = t.trim().to_string();
            (!trimmed.is_empty()).then_some(trimmed)
        })
        .or_else(|| {
            Path::new(&input.path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "未命名游戏".to_string());

    let now = now_unix_ms();
    sqlx::query(
        "INSERT INTO games(id, title, engine_type, path, runtime_version, created_at, updated_at) VALUES(?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&title)
    .bind(&input.engine_type)
    .bind(&input.path)
    .bind(&input.runtime_version)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| format!("db error: {e}"))?;

    let container_root = resolve_container_root(&app, &state.pool).await?;
    ensure_dir(&container_root)?;
    let toml_path = game_toml_path(&container_root, &id);
    let config = GameToml {
        engine_type: input.engine_type.clone(),
        entry_path: input.path.clone(),
        runtime_version: input.runtime_version.clone(),
        args: vec![],
        sandbox_home: true,
        cover_file: None,
    };
    write_game_toml(&toml_path, &config)?;

    let cover_path =
        resolve_cover_path(&container_root, &id).map(|p| p.to_string_lossy().to_string());

    Ok(GameEntryDto {
        id,
        title,
        engine_type: input.engine_type,
        path: input.path.clone(),
        path_valid: Path::new(&input.path).exists(),
        runtime_version: input.runtime_version,
        cover_path,
    })
}

#[tauri::command]
async fn launch_game(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    game_id: String,
) -> Result<LaunchGameResult, String> {
    // 中文说明：读取游戏信息 -> 读取 settings.toml -> 选择 NW.js 运行时 -> 启动进程。

    let row: Option<(String, String, Option<String>)> =
        sqlx::query_as("SELECT path, title, runtime_version FROM games WHERE id = ? LIMIT 1")
            .bind(&game_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| format!("db error: {e}"))?;

    let Some((game_path, _title, runtime_version_db)) = row else {
        return Err("game not found".to_string());
    };

    let game_dir = PathBuf::from(&game_path);
    if !game_dir.is_dir() {
        return Err(format!("game directory not found: {}", game_dir.display()));
    }

    let container_root = resolve_container_root(&app, &state.pool).await?;
    ensure_dir(&container_root)?;
    let toml_path = game_toml_path(&container_root, &game_id);

    // 读取 settings.toml；如果不存在则自动生成一个最小配置。
    let mut cfg = if toml_path.exists() {
        read_game_toml(&toml_path)?
    } else {
        GameToml {
            engine_type: "unknown".to_string(),
            entry_path: "www/index.html".to_string(),
            runtime_version: None,
            args: vec![],
            sandbox_home: true,
            cover_file: None,
        }
    };

    if !cfg.sandbox_home {
        let user_data_dir = game_user_data_dir(&container_root, &game_id);
        ensure_dir(&user_data_dir)?;
        ensure_user_data_arg(&mut cfg.args, &user_data_dir);
    }

    let version = cfg
        .runtime_version
        .clone()
        .or(runtime_version_db)
        .unwrap_or(nwjs::fetch_stable_version().await?);

    let target = nwjs::current_target()?;
    let install_dir = runtime_install_dir(&app, &version, "normal", &target)?;
    if !install_dir.exists() {
        return Err(format!(
            "NW.js runtime not installed: {} (version={version}, target={target}). Please download it first.",
            install_dir.display()
        ));
    }

    let exe = nwjs_executable_path(&install_dir)?;

    // 中文说明：RPG Maker MV/MZ 的 NW.js 项目通常以 game_dir 为 app root（包含 package.json）。
    // 所以我们把 game_dir 作为最后一个参数传给 NW.js。
    let pid = tokio::task::spawn_blocking(move || {
        let mut cmd = Command::new(exe);
        cmd.current_dir(&game_dir);

        // 把 HOME / XDG 目录重定向到容器中，避免在真实 ~/.config/<包名> 创建目录。
        if cfg.sandbox_home {
            let profile_dir = game_profile_dir(&container_root, &game_id);
            ensure_dir(&profile_dir)?;

            // 中文说明：直接把 HOME/XDG 指到容器目录本身，避免额外创建 home/xdg 子目录。
            cmd.env("HOME", &profile_dir);
            cmd.env("XDG_CONFIG_HOME", &profile_dir);
            cmd.env("XDG_DATA_HOME", &profile_dir);
            cmd.env("XDG_CACHE_HOME", &profile_dir);
        }

        cmd.args(cfg.args);
        cmd.arg(&game_dir);

        let child = cmd
            .spawn()
            .map_err(|e| format!("failed to launch NW.js: {e}"))?;
        Ok::<u32, String>(child.id())
    })
    .await
    .map_err(|e| format!("launch task join error: {e}"))??;

    Ok(LaunchGameResult { pid })
}

#[tauri::command]
async fn delete_game(state: tauri::State<'_, AppState>, game_id: String) -> Result<(), String> {
    sqlx::query("DELETE FROM games WHERE id = ?")
        .bind(&game_id)
        .execute(&state.pool)
        .await
        .map_err(|e| format!("db error: {e}"))?;
    Ok(())
}

#[tauri::command]
async fn get_nwjs_stable_info() -> Result<nwjs::NwjsStableInfo, String> {
    nwjs::get_stable_info().await
}

#[tauri::command]
async fn download_nwjs_stable(
    app: tauri::AppHandle,
    flavor: Option<nwjs::NwjsFlavor>,
) -> Result<nwjs::NwjsInstallResult, String> {
    let info = nwjs::get_stable_info().await?;
    let flavor = flavor.unwrap_or(nwjs::NwjsFlavor::Normal);
    nwjs::download_and_install(&app, info.version, flavor, info.target).await
}

#[tauri::command]
async fn download_nwjs_version(
    app: tauri::AppHandle,
    version: String,
    flavor: nwjs::NwjsFlavor,
) -> Result<nwjs::NwjsInstallResult, String> {
    let target = nwjs::current_target()?;
    nwjs::download_and_install(&app, version, flavor, target).await
}

async fn init_db(app: &tauri::AppHandle) -> Result<SqlitePool, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to resolve app data dir: {e}"))?;
    ensure_dir(&app_data_dir)?;
    let db_path = app_data_dir.join("db").join("app.sqlite");
    if let Some(dir) = db_path.parent() {
        ensure_dir(dir)?;
    }

    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .map_err(|e| format!("failed to open sqlite: {e}"))?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| format!("migration error: {e}"))?;

    Ok(pool)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let pool = tauri::async_runtime::block_on(async move { init_db(&handle).await })
                .expect("failed to init db");
            app.manage(AppState { pool });

            setup_tray(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_settings,
            set_container_root,
            list_games,
            add_game,
            delete_game,
            import_game_dir,
            scan_games,
            update_game_title,
            get_game_container_info,
            get_game_launch_config,
            update_game_launch_config,
            set_game_cover,
            clear_game_cover,
            cleanup_unused_containers,
            launch_game,
            get_nwjs_stable_info,
            download_nwjs_stable,
            download_nwjs_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
