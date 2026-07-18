use tauri::Manager;
use tauri::menu::MenuBuilder;
use tauri::tray::TrayIconBuilder;

pub fn setup_tray(app: &tauri::AppHandle) -> Result<(), String> {
    let _ = app.remove_tray_by_id("main-tray");

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

pub fn remove_tray(app: &tauri::AppHandle) {
    let _ = app.remove_tray_by_id("main-tray");
}
