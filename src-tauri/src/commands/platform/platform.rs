/// 检测当前运行的操作系统类型
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
    let detected = match dark_light::detect() {
        Ok(dark_light::Mode::Dark) => Some("dark"),
        Ok(dark_light::Mode::Light) => Some("light"),
        Ok(dark_light::Mode::Unspecified) | Err(_) => None,
    };
    if let Some(theme) = detected {
        return Ok(theme.to_string());
    }

    #[cfg(target_os = "linux")]
    if let Some(is_dark) = detect_gnome_color_scheme() {
        return Ok(if is_dark { "dark" } else { "light" }.to_string());
    }

    Ok("light".to_string())
}

#[cfg(target_os = "linux")]
fn detect_gnome_color_scheme() -> Option<bool> {
    let output = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    crate::utils::system_theme::parse_color_scheme(&String::from_utf8_lossy(&output.stdout))
}
