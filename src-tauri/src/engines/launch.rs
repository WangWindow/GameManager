use std::path::{Path, PathBuf};
use std::process::Child;

use super::context::LaunchContext;
use super::detection::find_executable;
use super::profile::LaunchConfig;

#[allow(dead_code)]
pub trait LaunchStrategy: Send + Sync {
    fn strategy_name(&self) -> &str;

    fn launch(
        &self,
        game_path: &Path,
        config: &LaunchConfig,
        ctx: &dyn LaunchContext,
    ) -> Result<Child, String>;
}

pub struct NativeLauncher;

impl LaunchStrategy for NativeLauncher {
    fn strategy_name(&self) -> &str {
        "native"
    }

    fn launch(
        &self,
        game_path: &Path,
        config: &LaunchConfig,
        ctx: &dyn LaunchContext,
    ) -> Result<Child, String> {
        let exe = find_executable(game_path, &config.entry_patterns, &config.exclude_patterns)
            .ok_or_else(|| "找不到游戏可执行文件".to_string())?;

        let working_dir = exe.parent().unwrap_or(game_path);

        ctx.spawn(&exe.to_string_lossy(), &config.args, working_dir, &[])
    }
}

pub struct NwJsLauncher;

impl LaunchStrategy for NwJsLauncher {
    fn strategy_name(&self) -> &str {
        "nwjs"
    }

    fn launch(
        &self,
        game_path: &Path,
        config: &LaunchConfig,
        ctx: &dyn LaunchContext,
    ) -> Result<Child, String> {
        let runtime_id = &config.runtime_id;
        if runtime_id.is_empty() {
            return Err("NW.js 运行时 ID 未配置".into());
        }

        let nw_runtime_path = ctx
            .get_runtime(runtime_id)
            .ok_or_else(|| format!("NW.js 运行时 '{}' 未安装", runtime_id))?;

        let nw_binary = find_nw_binary(&nw_runtime_path)?;

        let mut args: Vec<String> = config.args.clone();
        args.push(game_path.to_string_lossy().to_string());

        ctx.spawn(&nw_binary.to_string_lossy(), &args, game_path, &[])
    }
}

#[allow(dead_code)]
fn find_nw_binary(runtime_dir: &Path) -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    let candidates = ["nw.exe"];
    #[cfg(not(target_os = "windows"))]
    let candidates = ["nw", "nwjs"];

    for name in &candidates {
        let candidate = runtime_dir.join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err(format!(
        "在 {} 中找不到 nw 可执行文件",
        runtime_dir.display()
    ))
}

pub struct ExternalLauncher;

impl LaunchStrategy for ExternalLauncher {
    fn strategy_name(&self) -> &str {
        "external"
    }

    fn launch(
        &self,
        game_path: &Path,
        config: &LaunchConfig,
        ctx: &dyn LaunchContext,
    ) -> Result<Child, String> {
        let program = &config.program;
        if program.is_empty() {
            return Err("external 策略缺少 program 配置".into());
        }

        let entry = find_executable(game_path, &config.entry_patterns, &config.exclude_patterns)
            .ok_or_else(|| "找不到游戏可执行文件".to_string())?;

        let mut args: Vec<String> = config.program_args_prefix.clone();

        let template = if config.args_template.is_empty() {
            "{exe}".to_string()
        } else {
            config.args_template.clone()
        };

        let resolved = template
            .replace("{exe}", &entry.to_string_lossy())
            .replace("{game_dir}", &game_path.to_string_lossy());

        for part in resolved.split_whitespace() {
            let trimmed = part.trim();
            if !trimmed.is_empty() {
                args.push(trimmed.to_string());
            }
        }

        args.extend(config.args.clone());

        ctx.spawn(program, &args, game_path, &[])
    }
}

pub fn build_strategy(name: &str) -> Result<Box<dyn LaunchStrategy>, String> {
    match name {
        "native" => Ok(Box::new(NativeLauncher)),
        "nwjs" => Ok(Box::new(NwJsLauncher)),
        "external" => Ok(Box::new(ExternalLauncher)),
        // Bottles 需要游戏级配置（bottle 名称和全局集成状态），实际执行由
        // launcher service 完成；这里保留可验证的插件策略占位实现。
        "bottles" => Ok(Box::new(NativeLauncher)),
        other => Err(format!("未知的启动策略: {}", other)),
    }
}
