#[cfg(target_os = "linux")]
use super::BottlesService;
use crate::commands::settings::SettingsState;
use crate::models::{
    Capabilities, IntegrationOptions, IntegrationSettingsInput, IntegrationStatus,
    SETTING_BOTTLES_DEFAULT, SETTING_BOTTLES_ENABLED,
};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

// ── Service helpers ──────────────────────────────────────────

pub async fn get_bottles_integration_status(
    db: Arc<Mutex<toasty::Db>>,
) -> Result<IntegrationStatus, String> {
    #[cfg(target_os = "linux")]
    {
        // 只在读取配置时持有数据库锁，不能跨越 Bottles CLI 探测。
        let (default_bottle, enabled_setting) = {
            let mut db_lock = db.lock().await;
            let default_bottle = crate::db::get_setting(&mut *db_lock, SETTING_BOTTLES_DEFAULT)
                .await?
                .and_then(|v| if v.trim().is_empty() { None } else { Some(v) });
            let enabled_setting = crate::db::get_setting(&mut *db_lock, SETTING_BOTTLES_ENABLED)
                .await?
                .map(|v| v == "1")
                .unwrap_or(false);
            (default_bottle, enabled_setting)
        };

        let cli = BottlesService::detect_cli().await;
        let installed = cli.is_some();
        let bottles = if let Some(cli) = cli {
            BottlesService::list_bottles(&cli).await.unwrap_or_default()
        } else {
            Vec::new()
        };

        let options = IntegrationOptions {
            installed: Some(installed),
            bottles: Some(bottles),
            default_bottle,
        };

        return Ok(IntegrationStatus {
            key: "bottles".to_string(),
            available: true,
            enabled: enabled_setting && installed,
            options: Some(options),
        });
    }

    #[cfg(not(target_os = "linux"))]
    {
        let options = IntegrationOptions {
            installed: Some(false),
            bottles: None,
            default_bottle: None,
        };
        return Ok(IntegrationStatus {
            key: "bottles".to_string(),
            available: false,
            enabled: false,
            options: Some(options),
        });
    }
}

pub async fn set_bottles_integration_settings(
    input: IntegrationSettingsInput,
    db: &mut toasty::Db,
) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        if let Some(enabled) = input.enabled {
            let value = if enabled { "1" } else { "0" };
            crate::db::set_setting(db, SETTING_BOTTLES_ENABLED, value).await?;
        }

        if let Some(options) = input.options {
            if let Some(default_bottle) = options.default_bottle {
                crate::db::set_setting(db, SETTING_BOTTLES_DEFAULT, &default_bottle).await?;
            }
        }
        return Ok(());
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = input;
        let _ = db;
        return Err("Bottles 仅支持在 Linux 上运行".to_string());
    }
}

// ── Tauri commands ───────────────────────────────────────────

/// 获取全部能力（按集成返回）
#[tauri::command]
pub async fn get_capabilities(state: State<'_, SettingsState>) -> Result<Capabilities, String> {
    let mut integrations = Vec::new();
    integrations.push(get_bottles_integration_status(state.db.clone()).await?);
    Ok(Capabilities { integrations })
}

/// 获取单个集成状态
#[tauri::command]
pub async fn get_integration_status(
    key: String,
    state: State<'_, SettingsState>,
) -> Result<IntegrationStatus, String> {
    match key.as_str() {
        "bottles" => get_bottles_integration_status(state.db.clone()).await,
        _ => Err("未知集成".to_string()),
    }
}

/// 更新集成设置
#[tauri::command]
pub async fn set_integration_settings(
    input: IntegrationSettingsInput,
    state: State<'_, SettingsState>,
) -> Result<(), String> {
    let mut db_lock = state.db.lock().await;
    match input.key.as_str() {
        "bottles" => set_bottles_integration_settings(input, &mut *db_lock).await,
        _ => Err("未知集成".to_string()),
    }
}
