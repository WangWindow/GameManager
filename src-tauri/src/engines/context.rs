//! 上下文 trait — 引擎配置可以调用的受控 API。
//!
//! 这些 trait 是软件能力的「安全门」：引擎配置文件只能声明要使用哪个能力，
//! 实际执行由 Rust 端的 trait 实现控制。

use glob::Pattern;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

// ─── 检测上下文 ──────────────────────────────────────────

/// 检测规则求值时可用的文件系统查询能力。
///
/// 所有路径操作相对于正在检测的游戏目录。
#[allow(dead_code)]
pub trait DetectionContext {
    /// 检查游戏目录下是否存在某个相对路径文件
    fn file_exists(&self, relative_path: &str) -> bool;

    /// 检查游戏目录下是否存在某个相对路径目录
    fn dir_exists(&self, relative_path: &str) -> bool;

    /// 文件名 glob 匹配（例如 "RGSS*.dll"）
    /// 返回是否有至少一个匹配项
    fn glob_match(&self, pattern: &str) -> bool;

    /// 在游戏目录的有限深度子目录中进行 glob 匹配，适合 Unreal 等目录型引擎。
    fn glob_match_recursive(&self, pattern: &str, max_depth: u32) -> bool {
        let mut queue = vec![(self.game_dir().to_path_buf(), 0)];
        while let Some((dir, depth)) = queue.pop() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if simple_glob_match(pattern, &name) {
                        return true;
                    }
                    if depth < max_depth && entry.path().is_dir() {
                        queue.push((entry.path(), depth + 1));
                    }
                }
            }
        }
        false
    }

    /// 目录下是否有指定扩展名的文件（例如 "rpy", "exe"）
    fn has_extension(&self, ext: &str) -> bool;

    /// 当前目录是否包含带执行权限的非 Windows 文件。
    fn has_native_executable(&self) -> bool;

    /// 获取当前正在检测的游戏目录路径
    fn game_dir(&self) -> &Path;
}

// ─── 启动上下文 ──────────────────────────────────────────

/// 启动策略可用的进程管理能力。
///
/// 所有进程启动必须通过此 trait 的 `spawn` 方法，保证沙箱和安全策略一致。
#[allow(dead_code)]
pub trait LaunchContext {
    /// 获取已安装运行时的路径（如 NW.js SDK）
    fn get_runtime(&self, runtime_id: &str) -> Option<PathBuf>;

    /// 获取游戏沙箱容器目录
    fn container_dir(&self, profile_key: &str) -> PathBuf;

    /// 受控进程启动 — 所有引擎启动最终都得过这个方法
    fn spawn(
        &self,
        program: &str,
        args: &[String],
        working_dir: &Path,
        env: &[(String, String)],
    ) -> Result<std::process::Child, String>;
}

// ─── 资源上下文 ──────────────────────────────────────────

/// 封面和图标资源提取能力。
#[allow(dead_code)]
pub trait ResourceContext {
    /// 按优先级文件名列表查找封面图片
    fn find_cover(&self, game_dir: &Path, preferred_names: &[String]) -> Option<PathBuf>;

    /// 从可执行文件中提取图标（返回 PNG 字节）
    fn extract_exe_icon(&self, exe_path: &Path) -> Option<Vec<u8>>;
}

// ─── 默认实现（用于测试和开发） ──────────────────────────

/// 基于真实文件系统的 DetectionContext 实现
pub struct FsDetectionContext {
    game_dir: PathBuf,
    direct_entries: OnceLock<Vec<PathBuf>>,
    recursive_names: OnceLock<Vec<(String, u32)>>,
}

impl FsDetectionContext {
    pub fn new(game_dir: PathBuf) -> Self {
        Self {
            game_dir,
            direct_entries: OnceLock::new(),
            recursive_names: OnceLock::new(),
        }
    }

    fn direct_entries(&self) -> &[PathBuf] {
        self.direct_entries.get_or_init(|| {
            std::fs::read_dir(&self.game_dir)
                .map(|entries| entries.flatten().map(|entry| entry.path()).collect())
                .unwrap_or_default()
        })
    }

    fn recursive_names(&self) -> &[(String, u32)] {
        self.recursive_names.get_or_init(|| {
            let mut names = Vec::new();
            let mut queue = vec![(self.game_dir.clone(), 0)];
            while let Some((dir, depth)) = queue.pop() {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        names.push((entry.file_name().to_string_lossy().to_string(), depth));
                        if depth < 3 && entry.path().is_dir() {
                            queue.push((entry.path(), depth + 1));
                        }
                    }
                }
            }
            names
        })
    }
}

impl DetectionContext for FsDetectionContext {
    fn file_exists(&self, relative_path: &str) -> bool {
        self.game_dir.join(relative_path).is_file()
    }

    fn dir_exists(&self, relative_path: &str) -> bool {
        self.game_dir.join(relative_path).is_dir()
    }

    fn glob_match(&self, pattern: &str) -> bool {
        self.direct_entries().iter().any(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| simple_glob_match(pattern, name))
        })
    }

    fn glob_match_recursive(&self, pattern: &str, max_depth: u32) -> bool {
        self.recursive_names()
            .iter()
            .any(|(name, depth)| *depth <= max_depth && simple_glob_match(pattern, name))
    }

    fn has_extension(&self, ext: &str) -> bool {
        let ext = ext.trim_start_matches('.');
        self.direct_entries().iter().any(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case(ext))
        })
    }

    fn has_native_executable(&self) -> bool {
        self.direct_entries()
            .iter()
            .any(|path| crate::utils::path::is_linux_native_executable(path))
    }

    fn game_dir(&self) -> &Path {
        &self.game_dir
    }
}

pub fn simple_glob_match(pattern: &str, name: &str) -> bool {
    let pattern = pattern.to_lowercase();
    let name = name.to_lowercase();
    Pattern::new(&pattern).map_or(false, |p| p.matches(&name))
}
