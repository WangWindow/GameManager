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
