// 新版本lib.rs - 采用模块化架构
//
// 架构说明：
// 1. models - 定义数据结构
// 2. services - 实现业务逻辑
// 3. commands - 暴露Tauri命令
// 4. utils - 工具函数

mod commands;
mod models;
mod services;

use services::db;
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tauri::menu::MenuBuilder;
use tauri::tray::TrayIconBuilder;
use tokio::sync::Mutex;

/// 初始化数据库
async fn init_database(app: &tauri::AppHandle) -> Result<SqlitePool, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?;

    let db_path = app_data_dir.join("db").join("app.sqlite");
    db::init_database(&db_path).await
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
    pool: &SqlitePool,
) -> Result<PathBuf, String> {
    if let Some(value) = db::get_setting(pool, models::SETTING_CONTAINER_ROOT).await? {
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
            let handle = app.handle().clone();

            // 初始化数据库
            let pool = tauri::async_runtime::block_on(async move { init_database(&handle).await })
                .expect("数据库初始化失败");

            // 解析容器根目录
            let handle2 = app.handle().clone();
            let pool2 = pool.clone();
            let container_root = tauri::async_runtime::block_on(async move {
                resolve_container_root(&handle2, &pool2).await
            })
            .unwrap_or_else(|_| default_container_root(app.handle()).unwrap());

            // 迁移profile目录命名
            let pool3 = pool.clone();
            let migrate_root = container_root.clone();
            tauri::async_runtime::block_on(async move {
                let service = services::GameService::new(pool3);
                let _ = service.migrate_profile_keys(&migrate_root).await;
            });

            // 创建服务
            let game_service = services::GameService::new(pool.clone());
            let engine_service = services::EngineService::new(pool.clone());
            let launcher_service = services::LauncherService::new();

            // 管理状态
            app.manage(commands::game::AppState {
                game_service: Arc::new(Mutex::new(game_service)),
                engine_service: Arc::new(Mutex::new(services::EngineService::new(pool.clone()))),
                launcher_service: Arc::new(Mutex::new(launcher_service)),
                pool: pool.clone(),
                container_root: Arc::new(Mutex::new(container_root.to_string_lossy().to_string())),
            });

            app.manage(commands::engine::EngineState {
                engine_service: Arc::new(Mutex::new(engine_service)),
            });

            app.manage(commands::settings::SettingsState {
                pool: pool.clone(),
                game_service: Arc::new(Mutex::new(services::GameService::new(pool.clone()))),
                engine_service: Arc::new(Mutex::new(services::EngineService::new(pool.clone()))),
                container_root: Arc::new(Mutex::new(container_root.to_string_lossy().to_string())),
            });

            // 设置托盘
            setup_tray(app.handle())?;

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
            // 设置相关命令
            commands::get_app_settings,
            commands::set_container_root,
            commands::get_bottles_status,
            commands::set_default_bottle,
            commands::set_bottles_enabled,
            commands::get_nwjs_stable_info,
            commands::download_nwjs_stable,
            commands::cleanup_unused_containers,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri应用运行失败");
}
