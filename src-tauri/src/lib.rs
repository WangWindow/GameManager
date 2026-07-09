#![recursion_limit = "2048"]

mod commands;
mod db;
mod engines;
mod models;
mod services;
mod utils;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tauri::menu::MenuBuilder;
use tauri::tray::TrayIconBuilder;
use tokio::sync::Mutex;

/// 初始化日志系统
fn init_logger(app: &tauri::AppHandle) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?;

    let log_dir = app_data_dir.join("logs");
    let is_debug = cfg!(debug_assertions);

    crate::services::logger::init_logger(&log_dir, is_debug)
}

/// 初始化数据库
async fn init_database(app: &tauri::AppHandle) -> Result<toasty::Db, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?;

    let db_path = app_data_dir.join("db").join("app.sqlite");
    crate::db::init_db(&db_path).await
}

/// 获取默认容器根目录
fn default_container_root(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?;
    Ok(app_data_dir.join("containers"))
}

/// 解析容器根目录
async fn resolve_container_root(
    app: &tauri::AppHandle,
    db: &mut toasty::Db,
) -> Result<PathBuf, String> {
    if let Some(value) = crate::db::get_setting(db, models::SETTING_CONTAINER_ROOT).await? {
        return Ok(PathBuf::from(value));
    }
    Ok(default_container_root(app)?)
}

/// 设置系统托盘
fn setup_tray(app: &tauri::AppHandle) -> Result<(), String> {
    let menu = MenuBuilder::new(app)
        .text("toggle_window", "显示/隐藏")
        .separator()
        .text("quit", "退出")
        .build()
        .map_err(|e| format!("创建托盘菜单失败: {}", e))?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| "缺少默认窗口图标".to_string())?;

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
        .map_err(|e| format!("创建托盘图标失败: {}", e))?;

    Ok(())
}

/// 应用程序入口
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 初始化日志系统（最先执行）
            if let Err(e) = init_logger(app.handle()) {
                eprintln!("日志系统初始化失败: {}", e);
            }

            tracing::info!("GameManager 启动中...");

            let handle = app.handle().clone();

            // 初始化数据库
            let db = Arc::new(Mutex::new(
                tauri::async_runtime::block_on(async move { init_database(&handle).await })
                    .expect("数据库初始化失败"),
            ));

            tracing::info!("数据库初始化完成");

            // 解析容器根目录
            let db2 = db.clone();
            let handle2 = app.handle().clone();
            let container_root = tauri::async_runtime::block_on(async move {
                let mut db_lock = db2.lock().await;
                resolve_container_root(&handle2, &mut *db_lock).await
            })
            .unwrap_or_else(|_| default_container_root(app.handle()).unwrap());

            tracing::debug!(container_root = %container_root.display(), "容器根目录");

            // 迁移profile目录命名
            let db3 = db.clone();
            let migrate_root = container_root.clone();
            tauri::async_runtime::block_on(async move {
                let service = crate::services::GameService::new(db3);
                let _ = service.migrate_profile_keys(&migrate_root).await;
            });

            // 创建服务
            let game_service = crate::services::GameService::new(db.clone());
            let engine_service = crate::services::EngineService::new(db.clone());
            let launcher_service = crate::services::LauncherService::new();

            // 初始化引擎注册表
            let engine_registry = {
                let mut registry = crate::engines::EngineRegistry::new();
                let engines_dir = app
                    .path()
                    .app_data_dir()
                    .map(|p| p.join("engines"))
                    .unwrap_or_else(|_| PathBuf::from("engines"));

                // 每次启动同步内置 TOML
                let _ = std::fs::create_dir_all(&engines_dir);
                let source_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("engines");
                if source_dir.exists() {
                    for entry in std::fs::read_dir(&source_dir).into_iter().flatten() {
                        if let Ok(entry) = entry {
                            let path = entry.path();
                            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                                if let Some(file_name) = path.file_name() {
                                    let _ = std::fs::copy(&path, &engines_dir.join(file_name));
                                }
                            }
                        }
                    }
                }

                let _warnings = registry.load(&engines_dir, &HashMap::new());

                // 恢复持久化的启用/禁用状态
                let db_clone = db.clone();
                let registry = tauri::async_runtime::block_on(async move {
                    let mut db_lock = db_clone.lock().await;
                    let ids: Vec<String> = registry
                        .engine_ids()
                        .iter()
                        .map(|s| s.to_string())
                        .collect();
                    for id in &ids {
                        let key = format!("engine.{}.enabled", id);
                        if let Ok(Some(val)) = crate::db::get_setting(&mut db_lock, &key).await {
                            let _ = registry.set_enabled(id, val == "1");
                        }
                    }
                    registry
                });

                Arc::new(Mutex::new(registry))
            };

            // 管理状态
            app.manage(commands::state::AppState {
                game_service: Arc::new(Mutex::new(game_service)),
                engine_service: Arc::new(Mutex::new(crate::services::EngineService::new(
                    db.clone(),
                ))),
                launcher_service: Arc::new(Mutex::new(launcher_service)),
                db: db.clone(),
                container_root: Arc::new(Mutex::new(container_root.to_string_lossy().to_string())),
                engine_registry,
                config_cache: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            });

            app.manage(commands::engine::EngineState {
                engine_service: Arc::new(Mutex::new(engine_service)),
                db: db.clone(),
            });

            app.manage(commands::settings::SettingsState {
                db: db.clone(),
                game_service: Arc::new(Mutex::new(crate::services::GameService::new(db.clone()))),
                engine_service: Arc::new(Mutex::new(crate::services::EngineService::new(
                    db.clone(),
                ))),
                container_root: Arc::new(Mutex::new(container_root.to_string_lossy().to_string())),
            });

            // 设置托盘
            setup_tray(app.handle())?;

            tracing::info!("GameManager 启动完成");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 游戏相关命令
            commands::get_games,
            commands::get_game,
            commands::add_game,
            commands::update_game,
            commands::delete_game,
            commands::launch_game,
            commands::import_game_dir,
            commands::scan_games,
            commands::get_game_settings,
            commands::save_game_settings,
            commands::refresh_game_cover,
            commands::get_game_profile_dir,
            commands::open_path,
            // 引擎相关命令
            commands::get_engines,
            commands::find_engine,
            commands::add_engine,
            commands::delete_engine,
            commands::get_engine_update_info,
            commands::update_engine,
            commands::get_engine_registry,
            commands::get_engine_registry_detail,
            commands::set_engine_enabled,
            commands::get_engine_profile_detail,
            // 设置相关命令
            commands::get_app_settings,
            commands::set_container_root,
            commands::set_nwjs_keep_latest_only,
            commands::get_platform,
            commands::get_system_theme,
            commands::get_capabilities,
            commands::get_integration_status,
            commands::set_integration_settings,
            commands::get_nwjs_stable_info,
            commands::download_nwjs_stable,
            commands::cleanup_old_nwjs_versions,
            commands::cleanup_unused_containers,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri应用运行失败");
}
