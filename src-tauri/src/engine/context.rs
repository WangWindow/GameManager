//! 上下文 trait — 引擎配置可以调用的受控 API。
//!
//! 这些 trait 是软件能力的「安全门」：引擎配置文件只能声明要使用哪个能力，
//! 实际执行由 Rust 端的 trait 实现控制。

use std::path::{Path, PathBuf};

// ─── 检测上下文 ──────────────────────────────────────────

/// 检测规则求值时可用的文件系统查询能力。
///
/// 所有路径操作相对于正在检测的游戏目录。
pub trait DetectionContext {
    /// 检查游戏目录下是否存在某个相对路径文件
    fn file_exists(&self, relative_path: &str) -> bool;

    /// 检查游戏目录下是否存在某个相对路径目录
    fn dir_exists(&self, relative_path: &str) -> bool;

    /// 文件名 glob 匹配（例如 "RGSS*.dll"）
    /// 返回是否有至少一个匹配项
    fn glob_match(&self, pattern: &str) -> bool;

    /// 目录下是否有指定扩展名的文件（例如 "rpy", "exe"）
    fn has_extension(&self, ext: &str) -> bool;

    /// 获取当前正在检测的游戏目录路径
    fn game_dir(&self) -> &Path;
}

// ─── 启动上下文 ──────────────────────────────────────────

/// 启动策略可用的进程管理能力。
///
/// 所有进程启动必须通过此 trait 的 `spawn` 方法，保证沙箱和安全策略一致。
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
}

impl FsDetectionContext {
    pub fn new(game_dir: PathBuf) -> Self {
        Self { game_dir }
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
        if let Ok(entries) = std::fs::read_dir(&self.game_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if simple_glob_match(pattern, &name) {
                    return true;
                }
            }
        }
        false
    }

    fn has_extension(&self, ext: &str) -> bool {
        let ext = ext.trim_start_matches('.');
        if let Ok(entries) = std::fs::read_dir(&self.game_dir) {
            for entry in entries.flatten() {
                if entry
                    .path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case(ext))
                    == Some(true)
                {
                    return true;
                }
            }
        }
        false
    }

    fn game_dir(&self) -> &Path {
        &self.game_dir
    }
}

pub fn simple_glob_match(pattern: &str, name: &str) -> bool {
    let pattern = pattern.to_lowercase();
    let name = name.to_lowercase();
    glob_match_impl(&pattern, &name)
}

fn glob_match_impl(pattern: &str, name: &str) -> bool {
    if pattern.is_empty() {
        return name.is_empty();
    }

    let first = pattern.chars().next().unwrap();
    let rest = &pattern[first.len_utf8()..];

    match first {
        '*' => {
            // * matches zero or more characters
            if glob_match_impl(rest, name) {
                return true;
            }
            if name.is_empty() {
                return false;
            }
            let name_first = name.chars().next().unwrap();
            glob_match_impl(pattern, &name[name_first.len_utf8()..])
        }
        '?' => {
            if name.is_empty() {
                return false;
            }
            let name_first = name.chars().next().unwrap();
            glob_match_impl(rest, &name[name_first.len_utf8()..])
        }
        c => {
            if name.is_empty() {
                return false;
            }
            let name_first = name.chars().next().unwrap();
            if c == name_first {
                glob_match_impl(rest, &name[name_first.len_utf8()..])
            } else {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn glob_match_exact() {
        assert!(simple_glob_match("test.txt", "test.txt"));
        assert!(!simple_glob_match("test.txt", "other.txt"));
    }

    #[test]
    fn glob_match_star() {
        assert!(simple_glob_match("*.dll", "RGSS301.dll"));
        assert!(simple_glob_match("RGSS*.dll", "RGSS301.dll"));
        assert!(!simple_glob_match("*.exe", "RGSS301.dll"));
    }

    #[test]
    fn glob_match_question() {
        assert!(simple_glob_match("file?.txt", "file1.txt"));
        assert!(!simple_glob_match("file?.txt", "file12.txt"));
    }

    #[test]
    fn glob_match_complex() {
        assert!(simple_glob_match("*_Data", "MyGame_Data"));
        assert!(simple_glob_match("lib/python*", "lib/python3.11"));
        assert!(!simple_glob_match("*_Data", "MyGame_Data_Backup"));
    }

    #[test]
    fn fs_detection_context_basics() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = FsDetectionContext::new(dir.path().to_path_buf());

        // No files exist yet
        assert!(!ctx.file_exists("test.txt"));
        assert!(!ctx.dir_exists("subdir"));
        assert!(!ctx.has_extension("txt"));

        // Create a file
        fs::write(dir.path().join("test.txt"), b"hello").unwrap();
        assert!(ctx.file_exists("test.txt"));
        assert!(ctx.has_extension("txt"));

        // Create a directory
        fs::create_dir(dir.path().join("subdir")).unwrap();
        assert!(ctx.dir_exists("subdir"));
    }
}
