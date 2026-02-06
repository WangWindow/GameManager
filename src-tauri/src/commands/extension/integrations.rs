use crate::commands::settings::SettingsState;
use crate::models::{Capabilities, IntegrationSettingsInput, IntegrationStatus};
use crate::services::extension::integrations::{
    get_bottles_integration_status, set_bottles_integration_settings,
};
use tauri::State;

/// 获取全部能力（按集成返回）
#[tauri::command]
pub async fn get_capabilities(state: State<'_, SettingsState>) -> Result<Capabilities, String> {
    let mut integrations = Vec::new();
    integrations.push(get_bottles_integration_status(&state.pool).await?);
    Ok(Capabilities { integrations })
}

/// 获取单个集成状态
#[tauri::command]
pub async fn get_integration_status(
    key: String,
    state: State<'_, SettingsState>,
) -> Result<IntegrationStatus, String> {
    match key.as_str() {
        "bottles" => get_bottles_integration_status(&state.pool).await,
        _ => Err("未知集成".to_string()),
    }
}

/// 更新集成设置
#[tauri::command]
pub async fn set_integration_settings(
    input: IntegrationSettingsInput,
    state: State<'_, SettingsState>,
) -> Result<(), String> {
    match input.key.as_str() {
        "bottles" => set_bottles_integration_settings(input, &state.pool).await,
        _ => Err("未知集成".to_string()),
    }
}
