use serde_json::Value;
use std::path::Path;
use std::process::Command as StdCommand;

const FLATPAK_APP_ID: &str = "com.usebottles.bottles";

#[derive(Debug, Clone)]
pub struct BottlesCli {
    program: String,
    args_prefix: Vec<String>,
}

impl BottlesCli {
    pub fn new(program: String, args_prefix: Vec<String>) -> Self {
        Self {
            program,
            args_prefix,
        }
    }

    fn with_args_sync(&self, args: &[&str]) -> StdCommand {
        let mut cmd = StdCommand::new(&self.program);
        cmd.args(&self.args_prefix);
        cmd.args(args);
        cmd
    }
}

pub struct BottlesService;

impl BottlesService {
    pub async fn detect_cli() -> Option<BottlesCli> {
        if Self::is_flatpak_bottles_installed().await {
            return Some(BottlesCli::new(
                "flatpak".to_string(),
                vec![
                    "run".to_string(),
                    "--command=bottles-cli".to_string(),
                    FLATPAK_APP_ID.to_string(),
                ],
            ));
        }

        if Self::is_bottles_cli_available().await {
            return Some(BottlesCli::new("bottles-cli".to_string(), Vec::new()));
        }

        None
    }

    pub fn detect_cli_sync() -> Option<BottlesCli> {
        if Self::is_flatpak_bottles_installed_sync() {
            return Some(BottlesCli::new(
                "flatpak".to_string(),
                vec![
                    "run".to_string(),
                    "--command=bottles-cli".to_string(),
                    FLATPAK_APP_ID.to_string(),
                ],
            ));
        }

        if Self::is_bottles_cli_available_sync() {
            return Some(BottlesCli::new("bottles-cli".to_string(), Vec::new()));
        }

        None
    }

    pub async fn list_bottles(cli: &BottlesCli) -> Result<Vec<String>, String> {
        let cli_json = cli.clone();
        let output = tokio::task::spawn_blocking(move || {
            cli_json
                .with_args_sync(&["--json", "list", "bottles"]) // prefer json
                .output()
        })
        .await
        .map_err(|e| format!("无法执行 bottles-cli: {e}"))?
        .map_err(|e| format!("无法执行 bottles-cli: {e}"))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let parsed = Self::parse_bottles_json(&stdout);
            if !parsed.is_empty() {
                return Ok(parsed);
            }
        }

        // fallback to text mode
        let cli_text = cli.clone();
        let text_output = tokio::task::spawn_blocking(move || {
            cli_text.with_args_sync(&["list", "bottles"]).output()
        })
        .await
        .map_err(|e| format!("无法执行 bottles-cli: {e}"))?
        .map_err(|e| format!("无法执行 bottles-cli: {e}"))?;

        if !text_output.status.success() {
            let err = String::from_utf8_lossy(&text_output.stderr);
            return Err(format!("bottles-cli 运行失败: {err}"));
        }

        let text = String::from_utf8_lossy(&text_output.stdout);
        Ok(Self::parse_bottles_text(&text))
    }

    pub fn run_executable(
        cli: &BottlesCli,
        bottle: &str,
        exe_path: &str,
        args: &[String],
    ) -> Result<std::process::Child, String> {
        let use_exec_path = Path::new(exe_path).is_absolute();
        let mut cmd = if use_exec_path {
            cli.with_args_sync(&["run", "-e", exe_path, "-b", bottle])
        } else {
            cli.with_args_sync(&["run", "-p", exe_path, "-b", bottle])
        };
        if !args.is_empty() {
            cmd.arg("--").args(args);
        }

        cmd.spawn().map_err(|e| format!("启动 Bottles 失败: {e}"))
    }

    fn parse_bottles_json(raw: &str) -> Vec<String> {
        let parsed: Result<Value, _> = serde_json::from_str(raw);
        if let Ok(value) = parsed {
            if let Some(arr) = value.as_array() {
                return arr.iter().filter_map(Self::extract_bottle_name).collect();
            }
            if let Some(list) = value.get("bottles") {
                if let Some(arr) = list.as_array() {
                    return arr.iter().filter_map(Self::extract_bottle_name).collect();
                }
            }
        }
        Vec::new()
    }

    fn extract_bottle_name(value: &Value) -> Option<String> {
        if let Some(name) = value.as_str() {
            return Some(name.to_string());
        }
        if let Some(obj) = value.as_object() {
            for key in ["name", "Name", "bottle", "Bottle", "id", "Id"] {
                if let Some(v) = obj.get(key).and_then(|v| v.as_str()) {
                    return Some(v.to_string());
                }
            }
        }
        None
    }

    fn parse_bottles_text(raw: &str) -> Vec<String> {
        raw.lines()
            .map(|line| line.trim())
            .filter_map(|line| {
                if line.is_empty() {
                    return None;
                }
                if line.starts_with('-') {
                    return Some(line.trim_start_matches('-').trim().to_string());
                }
                if line.starts_with('*') {
                    return Some(line.trim_start_matches('*').trim().to_string());
                }
                if line.to_lowercase().contains("found") || line.contains("INFO") {
                    return None;
                }
                None
            })
            .collect()
    }

    async fn is_flatpak_bottles_installed() -> bool {
        let output = tokio::task::spawn_blocking(|| {
            StdCommand::new("flatpak")
                .args(["info", FLATPAK_APP_ID])
                .output()
        })
        .await;
        matches!(output, Ok(Ok(out)) if out.status.success())
    }

    async fn is_bottles_cli_available() -> bool {
        let output =
            tokio::task::spawn_blocking(|| StdCommand::new("which").arg("bottles-cli").output())
                .await;
        matches!(output, Ok(Ok(out)) if out.status.success())
    }

    fn is_flatpak_bottles_installed_sync() -> bool {
        let output = StdCommand::new("flatpak")
            .args(["info", FLATPAK_APP_ID])
            .output();
        matches!(output, Ok(out) if out.status.success())
    }

    fn is_bottles_cli_available_sync() -> bool {
        let output = StdCommand::new("which").arg("bottles-cli").output();
        matches!(output, Ok(out) if out.status.success())
    }
}
