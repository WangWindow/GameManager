use crate::models::{
    IntegrationOptions, IntegrationSettingsInput, IntegrationStatus, SETTING_BOTTLES_DEFAULT,
    SETTING_BOTTLES_ENABLED,
};
#[cfg(target_os = "linux")]
use crate::services::BottlesService;
use crate::services::db;
use sqlx::SqlitePool;

pub async fn get_bottles_integration_status(
    pool: &SqlitePool,
) -> Result<IntegrationStatus, String> {
    #[cfg(target_os = "linux")]
    {
        let default_bottle = db::get_setting(pool, SETTING_BOTTLES_DEFAULT)
            .await?
            .and_then(|v| if v.trim().is_empty() { None } else { Some(v) });
        let enabled_setting = db::get_setting(pool, SETTING_BOTTLES_ENABLED)
            .await?
            .map(|v| v == "1")
            .unwrap_or(false);

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
    pool: &SqlitePool,
) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        if let Some(enabled) = input.enabled {
            let value = if enabled { "1" } else { "0" };
            db::set_setting(pool, SETTING_BOTTLES_ENABLED, value).await?;
        }

        if let Some(options) = input.options {
            if let Some(default_bottle) = options.default_bottle {
                db::set_setting(pool, SETTING_BOTTLES_DEFAULT, &default_bottle).await?;
            }
        }
        return Ok(());
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = input;
        let _ = pool;
        return Err("Bottles 仅支持在 Linux 上运行".to_string());
    }
}
