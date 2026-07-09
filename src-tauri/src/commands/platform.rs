/// 平台检测
#[tauri::command]
pub async fn get_platform() -> Result<String, String> {
    #[cfg(target_os = "linux")]
    {
        return Ok("linux".to_string());
    }
    #[cfg(target_os = "windows")]
    {
        return Ok("windows".to_string());
    }
    #[cfg(target_os = "macos")]
    {
        return Ok("macos".to_string());
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        return Ok("unknown".to_string());
    }
}

/// 获取系统主题（由 Rust 从 OS 原生接口获取）
#[tauri::command]
pub async fn get_system_theme() -> Result<String, String> {
    let mode = dark_light::detect().unwrap_or(dark_light::Mode::Light);
    Ok(format!("{:?}", mode).to_lowercase())
}
